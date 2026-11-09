"""Hand-calculated bounds and quantities, compared with independent IFC measurements."""
import json, math, sys
from pathlib import Path
p=Path(sys.argv[1]); data=json.loads(p.read_text()); checks=[]
def target(n,cls):
    fixture=data['fixtures'][n-1]
    return next(x for x in fixture['products'] if x['class']==cls)
def bounds(n,cls,expected):
    actual=target(n,cls)['bounds_m']
    assert all(math.isclose(x,y,abs_tol=1e-7) for a,b in zip(actual,expected) for x,y in zip(a,b)),(n,actual,expected)
    checks.append({'fixture':f'{n:02d}','class':cls,'expected_bounds_m':expected,'actual_bounds_m':actual})
def value(n,cls,setname,name,expected):
    actual=target(n,cls)['quantities'][setname][name]
    assert math.isclose(actual,expected,abs_tol=0.00000051),(n,name,actual,expected)
    checks.append({'fixture':f'{n:02d}','quantity':name,'expected':expected,'actual':actual})
bounds(1,'IfcCovering',[[0,0,2.6],[4,3,2.62]])
c=math.cos(math.pi/6); s=math.sin(math.pi/6)
bounds(5,'IfcFooting',[[-(1.2*c+s)/2,-(1.2*s+c)/2,-0.6],[(1.2*c+s)/2,(1.2*s+c)/2,-0.2]])
bounds(7,'IfcFooting',[[0,-0.3,-0.3],[4,0.3,0]])
bounds(9,'IfcWall',[[0,-0.1,0],[4,0.1,6]])
bounds(10,'IfcWall',[[0,0.4,0],[4,0.6,4]])
bounds(11,'IfcWall',[[0,-0.1,0],[4,0.1,4-0.2*math.sqrt(1.0625)]])
for n,v in [(2,12/math.cos(math.pi/6)),(3,16*math.sqrt(1.0025)),(4,11),(8,12*math.sqrt(1.25))]:
    value(n,'IfcRoof','Qto_RoofBaseQuantities','GrossArea',v)
value(1,'IfcCovering','Qto_CoveringBaseQuantities','NetArea',10)
value(5,'IfcFooting','Qto_FootingBaseQuantities','NetVolume',0.48)
value(6,'IfcFooting','Qto_FootingBaseQuantities','NetVolume',math.pi*0.1)
value(7,'IfcFooting','Qto_FootingBaseQuantities','NetVolume',0.72)
value(9,'IfcWall','Qto_WallBaseQuantities','NetVolume',4.05)
value(10,'IfcWall','Qto_WallBaseQuantities','NetVolume',2.4)
print(json.dumps({'checks':checks,'passed':len(checks)},indent=2))
