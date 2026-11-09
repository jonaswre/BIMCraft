"""Export public Session-test commands with the real CLI; independently validate every IFC."""
import argparse, json, math, re, subprocess
from pathlib import Path
import ifcopenshell, ifcopenshell.geom, ifcopenshell.validate
from ifcopenshell.util.element import get_psets
from ifcopenshell.util.shape import get_volume, get_bbox, get_vertices, get_top_area, get_side_area
from ifcopenshell.util.pset import get_template

p=argparse.ArgumentParser(); p.add_argument('--cli',required=True); p.add_argument('--test',required=True); p.add_argument('--out',required=True)
a=p.parse_args(); out=Path(a.out); out.mkdir(exist_ok=True); results=[]
source=Path(a.test).read_text(); template=get_template('IFC4'); settings=ifcopenshell.geom.settings(); settings.set(settings.USE_WORLD_COORDS,True)
for i,m in enumerate(re.finditer(r'exported\(\s*r#"(.*?)"#\s*,?\s*\)',source,re.S),1):
    prefix=source[:m.start()]; name=re.findall(r'fn (\w+)\(',prefix)[-1]; stem=f'{i:02d}'
    script=out/(stem+'.txt'); script.write_text(m.group(1).strip()+'\n')
    native=out/(stem+'.bimcraft'); ifc=out/(stem+'.ifc')
    for args in [['run','--script',str(script),'--save',str(native)],['run',str(native),'--save',str(ifc)]]:
        run=subprocess.run([a.cli,*args],capture_output=True,text=True)
        if run.returncode: raise RuntimeError(run.stderr)
    model=ifcopenshell.open(str(ifc)); log=ifcopenshell.validate.json_logger(); ifcopenshell.validate.validate(model,log,express_rules=True)
    assert not log.statements, [(x.get('message'),x.get('attribute')) for x in log.statements]
    ids=[x.GlobalId for x in model.by_type('IfcRoot')]; assert len(ids)==len(set(ids)), 'duplicate GlobalId'
    products=[]
    for e in model.by_type('IfcElement'):
        if e.is_a('IfcOpeningElement'): continue
        created=ifcopenshell.geom.create_shape(settings,e); shape=created.geometry
        verts=get_vertices(shape); bounds=[verts.min(axis=0).tolist(),verts.max(axis=0).tolist()]
        volume=abs(get_volume(shape)); top=get_side_area(shape,axis='Z',angle=89.999); qs=get_psets(e,qtos_only=True)
        for setname,values in qs.items():
            allowed={t.Name for t in template.get_by_name(setname).HasPropertyTemplates}
            assert set(values)-{'id'} <= allowed, (setname,values)
        if e.is_a('IfcFooting') or e.is_a('IfcWall'):
            q=qs['Qto_FootingBaseQuantities' if e.is_a('IfcFooting') else 'Qto_WallBaseQuantities']
            # Circular sections tessellate to a polygon: allow the documented sub-percent shortfall.
            assert math.isclose(q['NetVolume'],volume,rel_tol=0.002,abs_tol=1e-6),(stem,q,volume)
        if e.is_a('IfcCovering'):
            assert math.isclose(qs['Qto_CoveringBaseQuantities']['NetArea'],top,abs_tol=1e-6),(stem,qs,top)
        if e.is_a('IfcRoof'):
            assert math.isclose(qs['Qto_RoofBaseQuantities']['NetArea'],top,abs_tol=2e-6),(stem,qs,top)
        products.append({'class':e.is_a(),'tag':e.Tag,'quantities':qs,'mesh_volume_m3':volume,'mesh_top_area_m2':top,'bounds_m':bounds})
    results.append({'fixture':stem,'test':name,'schema':model.schema,'express_statements':len(log.statements),'unique_global_ids':len(ids),'products':products})
assert len(results)==11, f'expected eleven exported fixtures, got {len(results)}'
print(json.dumps({'ifcopenshell':ifcopenshell.version,'fixtures':results},indent=2))
