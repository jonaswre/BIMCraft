"""Synthetic CLI/save/open and independent IFC output inspection for issue 321."""
from pathlib import Path
import json, math, subprocess, sys
import ifcopenshell, ifcopenshell.geom, ifcopenshell.util.element, ifcopenshell.util.shape, ifcopenshell.validate

ART = Path(__file__).parent
CLI = Path(sys.argv[1])

def run(name, *args):
    result = subprocess.run([str(CLI), *map(str,args)], capture_output=True, text=True, encoding="utf-8")
    (ART / (name+'.stdout')).write_text(result.stdout, encoding='utf-8')
    (ART / (name+'.stderr')).write_text(result.stderr, encoding='utf-8')
    assert result.returncode == 0, (name, result.returncode, result.stderr)
    return [json.loads(line) for line in result.stdout.splitlines() if line.strip()]

def elements(path, category):
    return run('elements-'+path.stem+'-'+category,'run',path,'--cmd','elements '+json.dumps({'category':category}))[-1]

commands = [
 'type '+json.dumps({'category':'door','name':'VerificationDoor','width':900,'height':2100}),
 'type '+json.dumps({'category':'window','name':'VerificationWindow','width':1200,'height':1200,'sill':900}),
 'type '+json.dumps({'category':'wall','name':'Layered','layers':[{'function':'finish','material':'Brick','thickness':40},{'function':'structure','material':'Concrete','thickness':180},{'function':'finish','material':'Plaster','thickness':20}]}),
 'wall '+json.dumps({'points':[[0,0],[5000,0]],'type':'Layered','height':2800,'location':'exterior-finish'}),
 'door '+json.dumps({'at':[1000,0],'type':'VerificationDoor'}),
 'window '+json.dumps({'at':[3500,0],'type':'VerificationWindow'}),
 'floor '+json.dumps({'boundary':[[7000,-1000],[13000,-1000],[13000,2000],[7000,2000]],'offset':400}),
 'floor '+json.dumps({'level':'Level 2','boundary':[[7000,-1000],[13000,-1000],[13000,2000],[7000,2000]]}),
 'wall '+json.dumps({'points':[[7500,0],[12500,0]],'type':'Layered','height':2800,'location':'interior-core','attach_base':100005,'attach_top':100006}),
 'roof '+json.dumps({'boundary':[[15000,-500],[21000,-500],[21000,3500],[15000,3500]],'gable':True}),
 'wall '+json.dumps({'points':[[15500,0],[20500,0]],'type':'Layered','height':2800,'location':'core-centre','attach_top':100008}),
 'wall '+json.dumps({'points':[[0,7000],[5000,7000]],'type':'Layered','height':2800,'location':'interior-finish','arc':45}),
 'wall '+json.dumps({'points':[[12500,7000],[7500,7000]],'type':'Layered','height':2800,'location':'exterior-core','arc':-60}),
 'georeference '+json.dumps({'crs':'EPSG:25832','eastings':123000,'northings':456000,'height':7000,'rotation':17,'scale':0.9996}),
]
(ART/'wall-intent.txt').write_text('\n'.join(commands)+'\n',encoding='utf-8')
run('cli-create','run','--script',ART/'wall-intent.txt','--save',ART/'original.ifc','--save',ART/'original.bimcraft')
opened=run('cli-ifc-reopen','run',ART/'original.ifc','--save',ART/'reopened.bimcraft','--save',ART/'roundtrip.ifc','--cmd','render '+json.dumps({'view':'{3D}','path':str(ART/'reopened-3d.png')}))
assert opened[0]['skipped'] == [], opened[0]
run('cli-native-reopen','run',ART/'reopened.bimcraft','--save',ART/'native-reopened.ifc')
# Compare command-visible authoring choices and GUID reference targets.
original = elements(ART/'original.bimcraft','wall')
reopened = elements(ART/'reopened.bimcraft','wall')
(ART/'elements-comparison.json').write_text(json.dumps({'original':original,'reopened':reopened},indent=2),encoding='utf-8')
assert original['count']==reopened['count']==5
before={p['guid']:p for p in original['elements']}
after={p['guid']:p for p in reopened['elements']}
assert before.keys()==after.keys()
for guid, p in before.items():
    q=after[guid]
    for field in ['location','start','end','arc','attach_top','attach_base']:
        if field in ['start','end']:
            assert all(abs(a-b)<0.002 for a,b in zip(p[field],q[field])), (guid,field,p,q)
        elif field=='arc':
            assert abs(p.get(field,0)-q.get(field,0))<1e-5, (guid,field,p,q)
        else:
            assert p.get(field)==q.get(field), (guid,field,p.get(field),q.get(field))
print('CLI-visible wall locations, axes, arcs and GUID attachments preserved.')


settings=ifcopenshell.geom.settings()
settings.set('use-world-coords', True)
summary={}
for name in ['original','roundtrip','native-reopened']:
    f=ifcopenshell.open(str(ART/(name+'.ifc')))
    logger=ifcopenshell.validate.json_logger()
    ifcopenshell.validate.validate(f,logger,express_rules=True)
    assert not logger.statements, (name,logger.statements)
    global_ids=[x.GlobalId for x in f.by_type('IfcRoot')]
    assert len(global_ids)==len(set(global_ids)), (name,'duplicate GlobalIds')
    products=[]
    for product in f.by_type('IfcProduct'):
        if product.Representation is None: continue
        shape=ifcopenshell.geom.create_shape(settings,product)
        verts=ifcopenshell.util.shape.get_vertices(shape.geometry)
        bbox=[verts.min(axis=0).tolist(), verts.max(axis=0).tolist()]
        volume=ifcopenshell.util.shape.get_volume(shape.geometry)
        key=('opening-of:'+product.HasFillings[0].RelatedBuildingElement.GlobalId) if product.is_a('IfcOpeningElement') and product.HasFillings else product.GlobalId
        products.append({'key':key,'opening_host':product.VoidsElements[0].RelatingBuildingElement.GlobalId if product.is_a('IfcOpeningElement') else None,'guid':product.GlobalId,'class':product.is_a(),'volume_m3':volume,'bbox_m':bbox,'pset':ifcopenshell.util.element.get_psets(product).get('BIMCraft_Wall')})
    summary[name]={'statements':len(logger.statements),'products_with_shapes':len(products),'shapes':products}
    if name=='original':
        # First wall: gross 5*0.24*2.8, less a 0.9*2.1 door and a 1.2*1.2 window.
        first=f.by_type('IfcWall')[0]
        shape=ifcopenshell.geom.create_shape(settings,first)
        volume=ifcopenshell.util.shape.get_volume(shape.geometry)
        expected=5*.24*2.8 - (.9*2.1+1.2*1.2)*.24
        assert abs(volume-expected) < 1e-6, (volume,expected)
summary['ifcopenshell_version']=ifcopenshell.version
for name in ['roundtrip','native-reopened']:
    before={p['key']:p for p in summary['original']['shapes']}
    after={p['key']:p for p in summary[name]['shapes']}
    assert before.keys()==after.keys(), (name,before.keys()^after.keys())
    for guid, p in before.items():
        q=after[guid]
        assert p['opening_host']==q['opening_host'], (name,p,q)
        assert abs(p['volume_m3']-q['volume_m3']) < 1e-4, (name,p,q)
        assert all(abs(a-b)<0.001 for x,y in zip(p['bbox_m'],q['bbox_m']) for a,b in zip(x,y)), (name,p,q)
        if p['class']=='IfcWall':
            assert {k:v for k,v in p['pset'].items() if k!='id'}=={k:v for k,v in q['pset'].items() if k!='id'}, (p,q)
(ART/'ifc-inspection.json').write_text(json.dumps(summary,indent=2),encoding='utf-8')
print('IFC schema, shapes, wall intent, authored element GUIDs, volumes and bounds preserved across IFC/native reopen.')
