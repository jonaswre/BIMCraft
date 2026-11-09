"""Synthetic steel-profile public command journey and independent IFC mesh/quantity checks.
Usage: python verify_steel.py REPO CLI
Requires IfcOpenShell 0.8.5. Outputs remain beside this script.
"""
import json
import math
from pathlib import Path
import subprocess
import sys

import ifcopenshell
import ifcopenshell.geom
import ifcopenshell.util.element
import ifcopenshell.util.shape
import ifcopenshell.validate

assert ifcopenshell.version == "0.8.5", ifcopenshell.version
repo, cli = map(lambda p: Path(p).resolve(), sys.argv[1:3])
out = Path(__file__).resolve().parent

def execute(name, args, check=True):
    r = subprocess.run([str(cli), *map(str, args)], cwd=out, text=True, encoding="utf-8", capture_output=True)
    (out / (name + ".stdout")).write_text(r.stdout, encoding="utf-8")
    (out / (name + ".stderr")).write_text(r.stderr, encoding="utf-8")
    if check:
        assert r.returncode == 0, (name, r.stdout, r.stderr)
    return [json.loads(line) for line in r.stdout.splitlines() if line.startswith("{")], r.returncode

commands = []
def cmd(command, **p):
    commands.append(command + " " + json.dumps(p, separators=(",", ":")))

fillet = 1 - math.pi / 4
expected = {
    "HEB 300": (2 * 300 * 19 + (300-38)*11 + 4*fillet*27**2, "IfcIShapeProfileDef"),
    "IPE 300": (2*150*10.7+(300-21.4)*7.1+4*fillet*15**2, "IfcIShapeProfileDef"),
    "Channel": (2*80*11+(200-22)*6+2*fillet*13**2, "IfcUShapeProfileDef"),
    "Angle": (10*(100+150-10)+fillet*12**2, "IfcLShapeProfileDef"),
    "RHS": (200*100-(200-16)*(100-16), "IfcRectangleHollowProfileDef"),
    "CHS": (math.pi/4*(168.3**2-(168.3-12)**2), "IfcCircleHollowProfileDef"),
    "Mixed I": (2*150*10+(300-20)*7, "IfcIShapeProfileDef"),
    "Mixed thick I": (2*150*15+(300-30)*9, "IfcIShapeProfileDef"),
    "Mixed U": (2*150*10+(300-20)*7, "IfcUShapeProfileDef"),
    "Mixed RHS": (150*300-(150-16)*(300-16), "IfcRectangleHollowProfileDef"),
    "Solid control": (150*300, "IfcRectangleProfileDef"),
}
cmd("structural_column", at=[0,0], type="HEB 300", height=3000, angle=30)
for x, name, shape, sizes in [
    (800,"Channel","U",dict(width=80,depth=200,web=6,flange=11,radius=13)),
    (1600,"Angle","L",dict(width=100,depth=150,thickness=10,radius=12)),
    (2400,"RHS","RHS",dict(width=200,depth=100,thickness=8)),
    (3200,"CHS","CHS",dict(width=168.3,thickness=6)),
]:
    cmd("type", category="structuralcolumn", name=name, shape=shape, **sizes)
    cmd("structural_column", at=[x,0], type=name, height=3000)
    cmd("type", category="beam", name=name, shape=shape, material="Metal", **sizes)
for y, name in [(1200,"IPE 300"),(2400,"Channel"),(3600,"Angle"),(4800,"RHS"),(6000,"CHS")]:
    cmd("beam", points=[[0,y],[6000,y]], type=name, offset=3500)
cmd("beam", points=[[0,7200],[3000,7200]], type="IPE 300", offset=1000,end_offset=5000,justify="centre")
for y,(name,shape,plates) in enumerate([
    ("Mixed I","I",dict(web=7,flange=10)),
    ("Mixed thick I","I",dict(web=9,flange=15)),
    ("Mixed U","U",dict(web=7,flange=10)),
    ("Mixed RHS","RHS",dict(thickness=8)),
]):
    cmd("type", category="beam",name=name,shape=shape,width=150,depth=300,material="Metal",**plates)
    cmd("beam", points=[[8000,y*1200],[14000,y*1200]],type=name,offset=3500,justify="bottom" if y==1 else "top")
cmd("type", category="beam",name="Solid control",width=150,depth=300,material="Metal")
cmd("beam", points=[[8000,4800],[14000,4800]],type="Solid control",offset=3500)
cmd("section",start=[3000,700],end=[3000,1700],name="Steel section")
cmd("elements",category="beam")
cmd("elements",category="structuralcolumn")
cmd("schedule",category="beam",fields=["Type","Profile","Section Area","Weight per Metre","Weight"])
cmd("schedule",category="structuralcolumn",fields=["Type","Profile","Section Area","Weight per Metre","Weight"])
(out/"steel.txt").write_text("\n".join(commands)+"\n", encoding="utf-8")
original,_=execute("steel-create",["run","--script","steel.txt","--save","steel.bimcraft","--save","steel.ifc"])
original_elements=[e for reply in original if "elements" in reply for e in reply["elements"] if isinstance(e,dict) and "width" in e]
by_id={str(e["id"]):e for e in original_elements}
query=['elements {"category":"beam"}','elements {"category":"structuralcolumn"}']
reopened,_=execute("native-reopen",["run","steel.bimcraft",*sum((["--cmd",q] for q in query),[]),"--save","steel-native-resaved.ifc"])
back=[e for reply in reopened if "elements" in reply for e in reply["elements"] if isinstance(e,dict) and "width" in e]
assert len(back)==len(original_elements)
assert sorted(original_elements,key=lambda e:e["id"])==sorted(back,key=lambda e:e["id"])
execute("render-plan",["run","steel.bimcraft","--cmd",'crop {"view":"Level 1","region":[[-400,-500],[3600,500]]}',"--cmd",'scale {"view":"Level 1","scale":20}',"--cmd",'render {"view":"Level 1","width":1600,"height":500,"path":"steel-plan.png"}'])
execute("render-3d",["run","steel.bimcraft","--cmd",'render {"view":"{3D}","width":1400,"height":900,"path":"steel-3d.png"}'])
execute("render-section",["run","steel.bimcraft","--cmd",'crop {"view":"Steel section","region":[[250,3100],[750,3700]]}',"--cmd",'scale {"view":"Steel section","scale":10}',"--cmd",'render {"view":"Steel section","width":600,"height":700,"path":"steel-section.png"}'])
perimeters_mm = {
    "HEB 300": 4*300+2*300-2*11+4*(math.pi/2-2)*27,
    "IPE 300": 4*150+2*300-2*7.1+4*(math.pi/2-2)*15,
    "Channel": 4*80+2*200-2*6+2*(math.pi/2-2)*13,
    "Angle": 2*(100+150)+(math.pi/2-2)*12,
    "RHS": 4*(200+100)-8*8,
    "CHS": math.pi*(2*168.3-12),
    "Mixed I": 4*150+2*300-2*7,
    "Mixed thick I": 4*150+2*300-2*9,
    "Mixed U": 4*150+2*300-2*7,
    "Mixed RHS": 4*(150+300)-8*8,
    "Solid control": 2*(150+300),
}
# Check the actual schedule replies as well as per-element quantities.
for reply in original:
    if "rows" not in reply:
        continue
    for name, label, area_cm2, kg_m, kg in reply["rows"]:
        if name == "Solid control":
            continue
        area = expected[name][0] / 1e6
        assert abs(area_cm2-area*1e4)<=0.00501
        assert abs(kg_m-area*7850)<=0.00501
        candidates=[e for e in original_elements if e["type"]==name]
        assert any(abs(kg-e["weight_kg"])<=0.00501 for e in candidates)
f=ifcopenshell.open(str(out/"steel.ifc"))
settings=ifcopenshell.geom.settings()
settings.set("mesher-linear-deflection", 0.00001)  # 0.01 mm, independently resolved curved edges
settings.set("mesher-angular-deflection", 0.05)
results=[]
for product in f.by_type("IfcBeam")+f.by_type("IfcColumn"):
    area_mm2,profile_class=expected[product.ObjectType]
    ty=ifcopenshell.util.element.get_type(product)
    usage=ifcopenshell.util.element.get_material(product)
    material=ifcopenshell.util.element.get_material(ty)
    assert usage.is_a("IfcMaterialProfileSetUsage") and material.is_a("IfcMaterialProfileSet")
    assert usage.ForProfileSet==material
    profile=material.MaterialProfiles[0].Profile
    assert profile.is_a(profile_class),(product.Name,profile)
    body=product.Representation.Representations[0].Items[0]
    assert body.SweptArea==profile,"the body and type must share their actual section"
    q=ifcopenshell.util.element.get_psets(product,qtos_only=True)
    q=next(v for k,v in q.items() if k.startswith("Qto_"))
    shape=ifcopenshell.geom.create_shape(settings,product)
    volume=ifcopenshell.util.shape.get_volume(shape.geometry)
    length=q["Length"]/1000
    area=area_mm2/1e6
    assert math.isclose(q["CrossSectionArea"],area,abs_tol=5.1e-7)
    assert math.isclose(q["GrossVolume"],area*length,abs_tol=5.1e-7)
    assert abs(volume-area*length)/(area*length)<0.0001,(product.Name,volume,area*length)
    lateral = ifcopenshell.util.shape.get_area(shape.geometry) - 2*area
    expected_lateral = perimeters_mm[product.ObjectType]/1000 * length
    assert abs(q["OuterSurfaceArea"]-expected_lateral)<=5.1e-7
    assert abs(lateral-expected_lateral)/expected_lateral<0.0001,(product.Name,lateral,expected_lateral)
    element=by_id[product.Tag]
    if product.ObjectType!="Solid control":
        assert abs(element["section_area_cm2"]-area_mm2/100)<=0.00501
        assert abs(element["weight_kg_m"]-area*7850)<=0.00501
        assert abs(element["weight_kg"]-area*length*7850)<=0.00501
    results.append(dict(type=product.ObjectType,ifc_class=product.is_a(),profile=profile_class,
        area_m2=area,quantity_area_m2=q["CrossSectionArea"],length_m=length,
        quantity_volume_m3=q["GrossVolume"],independent_mesh_volume_m3=volume,
        mesh_error_fraction=abs(volume-area*length)/(area*length),quantity_lateral_m2=q["OuterSurfaceArea"],
        independent_mesh_lateral_m2=lateral,cardinal_point=usage.CardinalPoint))
all_guids=[r.GlobalId for r in f.by_type("IfcRoot")]
assert len(all_guids)==len(set(all_guids)),"all GlobalIds must be unique"
again=ifcopenshell.open(str(out/"steel-native-resaved.ifc"))
assert sorted((e.is_a(),e.GlobalId) for e in f.by_type("IfcRoot"))==sorted((e.is_a(),e.GlobalId) for e in again.by_type("IfcRoot"))
reimported,_=execute("ifc-reopen",["run","steel.ifc",*sum((["--cmd",q] for q in query),[]),"--save","steel-ifc-reopened.bimcraft","--save","steel-ifc-reexported.ifc"])
returned=[e for reply in reimported if "elements" in reply for e in reply["elements"] if isinstance(e,dict) and "width" in e]
assert len(returned)==len(original_elements),(len(returned),len(original_elements))
for e in returned:
    original_e=by_id.get(str(e["id"]))
    # Import can allocate different ids; matching typed geometry is the public invariant.
    matches=[a for a in original_elements if a.get("type")==e.get("type") and a["category"]==e["category"] and abs(a.get("length",a.get("height"))-e.get("length",e.get("height")))<0.1]
    assert matches,(e,original_elements)
    assert any(abs(a.get("section_area_cm2",0)-e.get("section_area_cm2",0))<0.02 for a in matches),e
files=["steel.ifc","steel-native-resaved.ifc","steel-ifc-reexported.ifc"]
validation=subprocess.run([sys.executable,str(repo/"ci/validate_ifc.py"),*map(str,files)],capture_output=True,text=True,encoding="utf-8",cwd=out)
(out/"ifc-validation.log").write_text(validation.stdout+validation.stderr,encoding="utf-8")
assert validation.returncode==0,validation.stdout+validation.stderr
(out/"independent-steel-results.json").write_text(json.dumps(dict(ifcopenshell=ifcopenshell.version,mesh_linear_deflection_m=0.00001,mesh_angular_deflection=0.05,mesh_relative_tolerance=0.0001,global_ids=len(all_guids),results=results),indent=2),encoding="utf-8")
print(validation.stdout)
print(f"Checked {len(results)} member meshes, shared type profiles, real areas/volumes/mass, native reopen and IFC reimport/reexport.")
