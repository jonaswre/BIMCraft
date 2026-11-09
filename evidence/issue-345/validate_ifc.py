"""Independent IFC4 validation. Requires IfcOpenShell 0.8.5. Run after exercise.py."""
import json, math
from pathlib import Path
import ifcopenshell, ifcopenshell.geom, ifcopenshell.validate, ifcopenshell.util.element, ifcopenshell.util.placement
OUT=Path(__file__).resolve().parent
f=ifcopenshell.open(str(OUT/'plots.ifc'))
logger=ifcopenshell.validate.json_logger()
ifcopenshell.validate.validate(f,logger,express_rules=True)
assert not logger.statements, logger.statements
roots=f.by_type('IfcRoot'); ids=[e.GlobalId for e in roots]
assert len(ids)==len(set(ids)), 'duplicate GlobalId'
site=f.by_type('IfcSite')[0]
rep=next(r for r in site.Representation.Representations if r.RepresentationIdentifier=='FootPrint')
assert rep.RepresentationType=='Curve2D'
curves=rep.Items
assert len(curves)==2
areas=[]; perimeters=[]
for curve in curves:
    assert curve.is_a('IfcPolyline')
    pts=[tuple(p.Coordinates) for p in curve.Points]
    assert pts[0]==pts[-1]
    assert all(len(p)==2 for p in pts)
    areas.append(abs(sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(pts,pts[1:])))/2e6)
    perimeters.append(sum(math.dist(a,b) for a,b in zip(pts,pts[1:]))/1000)
assert sorted(areas)==[16.0,24.0], areas
assert perimeters==[20.0,20.0], perimeters
psets=ifcopenshell.util.element.get_psets(site)
assert psets['Pset_SiteCommon']['TotalArea']==40.0
assert list(psets['BIMCraft_PropertyLines']['Names'])==['Synthetic Plot','Concave synthetic plot']
assert len(set(psets['BIMCraft_PropertyLines']['Guids']))==2
settings=ifcopenshell.geom.settings()
settings.set('dimensionality', ifcopenshell.ifcopenshell_wrapper.CURVES_SURFACES_AND_SOLIDS)
settings.set('use-world-coords', True)
shape=ifcopenshell.geom.create_shape(settings,site)
verts=list(zip(*[iter(shape.geometry.verts)]*3))
assert verts and shape.geometry.edges and not shape.geometry.faces
bounds=[min(p[i] for p in verts) for i in range(3)]+[max(p[i] for p in verts) for i in range(3)]
assert all(abs(a-b)<1e-6 for a,b in zip(bounds,[0,0,0,14,4,0])), bounds
# A foreign-tool variant: remove all BIMCraft metadata, switch coordinates/placement to metres,
# and rotate/translate the site placement.
for rel in list(f.by_type('IfcRelDefinesByProperties')):
    if site in rel.RelatedObjects and rel.RelatingPropertyDefinition.Name=='BIMCraft_PropertyLines':
        pset=rel.RelatingPropertyDefinition; f.remove(rel); f.remove(pset)
for point in f.by_type('IfcCartesianPoint'):
    point.Coordinates=tuple(c/1000 for c in point.Coordinates)
for unit in f.by_type('IfcSIUnit'):
    if unit.UnitType=='LENGTHUNIT': unit.Prefix=None
site.ObjectPlacement.RelativePlacement.Location.Coordinates=(10.0,20.0,0.0)
site.ObjectPlacement.RelativePlacement.RefDirection=f.createIfcDirection((0.0,1.0,0.0))
site.ObjectPlacement.RelativePlacement.Axis=f.createIfcDirection((0.0,0.0,1.0))
# Storey elevations are informational length attributes too.
for storey in f.by_type('IfcBuildingStorey'): storey.Elevation=storey.Elevation/1000
f.write(str(OUT/'foreign-metre-rotated.ifc'))
logger=ifcopenshell.validate.json_logger(); ifcopenshell.validate.validate(f,logger,express_rules=True)
assert not logger.statements,logger.statements
shape=ifcopenshell.geom.create_shape(settings,site)
verts=list(zip(*[iter(shape.geometry.verts)]*3))
foreign_bounds=[min(p[i] for p in verts) for i in range(3)]+[max(p[i] for p in verts) for i in range(3)]
assert all(abs(a-b)<1e-6 for a,b in zip(foreign_bounds,[6,20,0,10,34,0])),foreign_bounds
report={'ifcopenshell':ifcopenshell.version,'schema':f.schema,'express_statements':0,'unique_global_ids':len(ids), 'areas_m2':areas,'perimeters_m':perimeters,'site_total_area_m2':40,'curve_shape_bounds_m':bounds,'foreign_curve_shape_bounds_m':foreign_bounds,'shape_edges':len(shape.geometry.edges)//2,'curve_faces':0}
(OUT/'ifc-validation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
print(json.dumps(report,indent=2))
