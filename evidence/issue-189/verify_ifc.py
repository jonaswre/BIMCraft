"""Independent IFC4/EXPRESS, shapes, identity, quantities and native mesh comparison."""
import json, math, pathlib, struct, subprocess, sys
import numpy as np
import ifcopenshell, ifcopenshell.geom, ifcopenshell.validate
from ifcopenshell.util.element import get_psets
root=pathlib.Path(__file__).resolve().parent
world=ifcopenshell.geom.settings(); world.set("use-world-coords",True)
local=ifcopenshell.geom.settings()
def geometry(product, settings=world):
    built=ifcopenshell.geom.create_shape(settings,product)
    shape=built.geometry
    vertices=np.array(shape.verts).reshape(-1,3)
    faces=np.array(shape.faces,dtype=np.int64).reshape(-1,3)
    return vertices[faces]
def volume(t):
    return abs(np.einsum('ij,ij->i',t[:,0],np.cross(t[:,1],t[:,2])).sum()/6)
def heights(tris,point):
    z=[]
    for p in tris:
        x,y=point
        d=(p[1,1]-p[2,1])*(p[0,0]-p[2,0])+(p[2,0]-p[1,0])*(p[0,1]-p[2,1])
        if abs(d)<1e-12: continue
        a=((p[1,1]-p[2,1])*(x-p[2,0])+(p[2,0]-p[1,0])*(y-p[2,1]))/d
        b=((p[2,1]-p[0,1])*(x-p[2,0])+(p[0,0]-p[2,0])*(y-p[2,1]))/d
        c=1-a-b
        if min(a,b,c)>=-1e-6: z.append(a*p[0,2]+b*p[1,2]+c*p[2,2])
    assert z
    return np.array([min(z),max(z)])
def glb(path):
    data=path.read_bytes(); length=struct.unpack_from('<I',data,12)[0]
    doc=json.loads(data[20:20+length]); binary=data[28+length:]
    node=next(n for n in doc['nodes'] if n.get('extras',{}).get('category')=='stair')
    def accessor(index):
        a=doc['accessors'][index]; v=doc['bufferViews'][a['bufferView']]
        shape={'SCALAR':1,'VEC3':3}[a['type']]
        dt={5126:'<f4',5125:'<u4',5123:'<u2'}[a['componentType']]
        return np.frombuffer(binary,dtype=dt,count=a['count']*shape,offset=v.get('byteOffset',0)+a.get('byteOffset',0)).reshape(-1,shape)
    triangles=[]
    for primitive in doc['meshes'][node['mesh']]['primitives']:
        pos=accessor(primitive['attributes']['POSITION'])
        pos=np.stack([pos[:,0],-pos[:,2],pos[:,1]],axis=1)
        indices=accessor(primitive['indices']).reshape(-1,3) if 'indices' in primitive else np.arange(len(pos)).reshape(-1,3)
        triangles.extend(pos[indices])
    return np.array(triangles)
def check(name,glb_name):
    f=ifcopenshell.open(str(root/name)); log=ifcopenshell.validate.json_logger()
    ifcopenshell.validate.validate(f,log,express_rules=True)
    assert not log.statements,log.statements
    ids=[x.GlobalId for x in f.by_type('IfcRoot')]; assert len(ids)==len(set(ids))
    stair=f.by_type('IfcStair')[0]
    assert get_psets(stair)['BIMCraft_Stair']['Open'] is True
    flights=f.by_type('IfcStairFlight'); pads=[x for x in f.by_type('IfcSlab') if x.PredefinedType=='LANDING']
    assert len(flights)==2 and len(pads)==1
    assert len(f.by_type('IfcRailing'))==2
    tris=[]
    for product in flights+pads:
        t=geometry(product); tris.extend(t)
        expected=.0931 if product.is_a('IfcStairFlight') else .25
        assert abs(volume(t)-expected)<2e-7,(product.is_a(),volume(t),expected)
        if product.is_a('IfcStairFlight') and name!='tessellated-open.ifc':
            assert len(product.Representation.Representations[0].Items)==9, '7 treads plus 2 supports per flight'
    tris=np.array(tris); native=glb(root/glb_name)
    assert np.allclose(tris.reshape(-1,3).min(0),native.reshape(-1,3).min(0),atol=2e-6)
    assert np.allclose(tris.reshape(-1,3).max(0),native.reshape(-1,3).max(0),atol=2e-6)
    angle=math.radians(37); c,s=math.cos(angle),math.sin(angle)
    probes=[]
    for x,y,expected in [(980,0,[1.680,1.720]),(980,494,[1.520,1.770]),(980,-494,[1.520,1.770]),(2460,0,[2.190,2.440])]:
        point=[(.2+(x*c-y*s)/1000),(.3+(x*s+y*c)/1000)]
        found=heights(tris,point); rendered=heights(native,point)
        assert np.allclose(found,expected,atol=2e-6),(name,found,expected)
        assert np.allclose(found,rendered,atol=2e-6),(name,found,rendered)
        probes.append(found.tolist())
    shapes=0
    for product in f.by_type('IfcProduct'):
        if product.Representation: geometry(product); shapes+=1
    result={'file':name,'schema':'IFC4','express_problems':0,'unique_globalids':len(ids),'product_shapes':shapes,
        'flights':2,'landings':1,'railings':2,'flight_volume_m3':.0931,'landing_volume_m3':.25,'sample_z_m':probes,
        'native_mesh_bounds_and_clearance_match':True}
    print(json.dumps(result)); return result
results=[check(n,g) for n,g in [('open-stair.ifc','open-stair.glb'),('reopened-bimcraft.ifc','reopened-bimcraft.glb'),('reopened-ifc.ifc','reopened-ifc.glb')]]
# Preserve the external placement/metadata but replace each flight's B-reps with an independently tessellated body.
f=ifcopenshell.open(str(root/'open-stair.ifc'))
for flight in f.by_type('IfcStairFlight'):
    t=geometry(flight,local); points=t.reshape(-1,3)*1000
    coordinates=f.create_entity('IfcCartesianPointList3D',CoordList=points.tolist())
    indices=(np.arange(len(points)).reshape(-1,3)+1).tolist()
    item=f.create_entity('IfcTriangulatedFaceSet',Coordinates=coordinates,Closed=True,CoordIndex=indices)
    old=flight.Representation.Representations[0]
    rep=f.create_entity('IfcShapeRepresentation',ContextOfItems=old.ContextOfItems,RepresentationIdentifier='Body',RepresentationType='Tessellation',Items=[item])
    flight.Representation.Representations=[rep]
    f.remove(old)
f.write(str(root/'tessellated-open.ifc'))
results.append(check('tessellated-open.ifc','open-stair.glb'))
p=subprocess.run([sys.argv[1],'run','tessellated-open.ifc','--save','reopened-tessellated.ifc','--save','reopened-tessellated.glb'],cwd=root,text=True,capture_output=True)
print(p.stdout); assert p.returncode==0,p.stderr
results.append(check('reopened-tessellated.ifc','reopened-tessellated.glb'))
(root/'ifc-validation.json').write_text(json.dumps({'ifcopenshell_version':ifcopenshell.version,'checks':results},indent=2))
print('PASS: IFC schema/EXPRESS, all shapes, unique IDs, volumes, native mesh bounds, under-stair clearances and tessellated IFC reopen')
