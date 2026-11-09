"""IFC placement/unit/body variants and the documented sample import journey."""
from pathlib import Path
import json, math, subprocess, sys
import ifcopenshell, ifcopenshell.geom, ifcopenshell.util.unit, ifcopenshell.util.shape, ifcopenshell.util.element, ifcopenshell.validate
from ifcopenshell.api.geometry import add_mesh_representation

ART=Path(__file__).parent
CLI=Path(sys.argv[1])
settings=ifcopenshell.geom.settings()
settings.set('use-world-coords',True)

def run(name,*args):
    r=subprocess.run([str(CLI),*map(str,args)],capture_output=True,text=True,encoding="utf-8")
    (ART/(name+'.stdout')).write_text(r.stdout,encoding='utf-8')
    (ART/(name+'.stderr')).write_text(r.stderr,encoding='utf-8')
    assert r.returncode==0,(name,r.stderr)
    return [json.loads(x) for x in r.stdout.splitlines() if x.strip()]

def shape_map(f):
    out={}
    for p in f.by_type('IfcProduct'):
        if p.Representation is None:continue
        shape=ifcopenshell.geom.create_shape(settings,p)
        verts=ifcopenshell.util.shape.get_vertices(shape.geometry)
        key=('opening-of:'+p.HasFillings[0].RelatedBuildingElement.GlobalId) if p.is_a('IfcOpeningElement') and p.HasFillings else p.GlobalId
        out[key]={'raw_guid':p.GlobalId,'class':p.is_a(),'volume_m3':ifcopenshell.util.shape.get_volume(shape.geometry),'min_m':verts.min(axis=0).tolist(),'max_m':verts.max(axis=0).tolist()}
    return out

def validate(path):
    f=ifcopenshell.open(str(path))
    log=ifcopenshell.validate.json_logger()
    ifcopenshell.validate.validate(f,log,express_rules=True)
    assert not log.statements,(path.name,log.statements)
    return f

original=ifcopenshell.open(str(ART/'original.ifc'))
angle=math.radians(31)
f=ifcopenshell.open(str(ART/'original.ifc'))
place=f.by_type('IfcBuilding')[0].ObjectPlacement.RelativePlacement
place.Location.Coordinates=(1200.,-800.,0.)
place.Axis=f.createIfcDirection((0.,0.,1.))
place.RefDirection=f.createIfcDirection((math.cos(angle),math.sin(angle),0.))
f.write(str(ART/'transformed.ifc'))

metres=ifcopenshell.util.unit.convert_file_length_units(original,'METRE')
metres.write(str(ART/'metres.ifc'))

mesh=ifcopenshell.open(str(ART/'original.ifc'))
local=ifcopenshell.geom.settings()
local.set('disable-opening-subtractions',True)
for product in mesh.by_type('IfcProduct'):
    if product.Representation is None: continue
    bodies=[r for r in product.Representation.Representations if r.RepresentationIdentifier=='Body']
    if not bodies: continue
    shape=ifcopenshell.geom.create_shape(local,product)
    vertices=ifcopenshell.util.shape.get_vertices(shape.geometry)
    faces=ifcopenshell.util.shape.get_faces(shape.geometry)
    rep=add_mesh_representation(mesh,context=bodies[0].ContextOfItems,vertices=[vertices.tolist()],faces=[faces.tolist()])
    assert len(bodies)==1
    bodies[0].Items=rep.Items
    bodies[0].RepresentationType=rep.RepresentationType
    mesh.remove(rep)
mesh.write(str(ART/'meshed.ifc'))

report={'ifcopenshell_version':ifcopenshell.version}
for name in ['transformed','metres','meshed']:
    source=validate(ART/(name+'.ifc'))
    results=run('cli-'+name,'run',ART/(name+'.ifc'),'--save',ART/(name+'-reopened.ifc'),'--cmd','elements {"category":"wall"}')
    assert results[0]['skipped']==[],(name,results[0])
    assert results[-1]['count']==5
    reopened=validate(ART/(name+'-reopened.ifc'))
    before,after=shape_map(source),shape_map(reopened)
    assert before.keys()==after.keys(),(name,before.keys()^after.keys())
    for guid,p in before.items():
        q=after[guid]
        # Curved body tessellation may differ; compare the exact IFC curves through their bounds and volume tolerance.
        assert abs(p['volume_m3']-q['volume_m3'])<0.01,(name,p,q)
        assert all(abs(a-b)<0.002 for key in ['min_m','max_m'] for a,b in zip(p[key],q[key])),(name,p,q)
    old={p.GlobalId:ifcopenshell.util.element.get_psets(p)['BIMCraft_Wall'] for p in source.by_type('IfcWall')}
    for p in reopened.by_type('IfcWall'):
        pset=ifcopenshell.util.element.get_psets(p)['BIMCraft_Wall']
        assert {k:v for k,v in old[p.GlobalId].items() if k!='id'}=={k:v for k,v in pset.items() if k!='id'}
    # Metre conversion retains the original grid-to-ground scale, independent of coordinate units.
    assert abs(reopened.by_type('IfcMapConversion')[0].Scale-0.0009996)<1e-12
    report[name]={'schema_statements':0,'products_with_shapes':len(before),'walls':results[-1],'source_shapes':before,'reopened_shapes':after}
    print(name,': zero schema statements; all',len(before),'shapes and all five authored wall identities preserved')

run('sample-export','run','--sample','--save',ART/'sample.ifc')
source=validate(ART/'sample.ifc')
opened=run('sample-reopen','run',ART/'sample.ifc','--save',ART/'sample-reopened.ifc','--cmd','render '+json.dumps({'view':'{3D}','path':str(ART/'sample-reopened-3d.png')}))
assert opened[0]['skipped']==[],opened[0]
reopened=validate(ART/'sample-reopened.ifc')
report['sample']={'schema_statements':0,'source_shapes':len(shape_map(source)),'reopened_shapes':len(shape_map(reopened)),'open':opened[0]}
print('sample: zero schema statements; source and reopened shapes built; no skipped elements')
(ART/'ifc-variants-inspection.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
