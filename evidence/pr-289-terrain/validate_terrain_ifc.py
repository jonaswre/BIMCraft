import collections
import json
import math
import pathlib
import ifcopenshell
import ifcopenshell.geom
import ifcopenshell.validate

root = pathlib.Path(__file__).resolve().parent
f = ifcopenshell.open(str(root / "survey.ifc"))
logger = ifcopenshell.validate.json_logger()
ifcopenshell.validate.validate(f, logger, express_rules=True)
assert not logger.statements, logger.statements
ids = [e.GlobalId for e in f.by_type("IfcRoot")]
assert len(ids) == len(set(ids)), "GlobalIds must be unique"
settings = ifcopenshell.geom.settings()
settings.set(settings.USE_WORLD_COORDS, True)
shapes = []
for product in f.by_type("IfcProduct"):
    if not product.Representation:
        continue
    shape = ifcopenshell.geom.create_shape(settings, product)
    v = shape.geometry.verts
    vertices = [v[i:i+3] for i in range(0, len(v), 3)]
    bounds = [[min(p[d] for p in vertices) for d in range(3)], [max(p[d] for p in vertices) for d in range(3)]]
    faces = shape.geometry.faces
    area = volume = 0.0
    for i in range(0, len(faces), 3):
        a,b,c = [vertices[faces[i+j]] for j in range(3)]
        u = [b[j]-a[j] for j in range(3)]
        w = [c[j]-a[j] for j in range(3)]
        cross = [u[1]*w[2]-u[2]*w[1],u[2]*w[0]-u[0]*w[2],u[0]*w[1]-u[1]*w[0]]
        area += math.sqrt(sum(x*x for x in cross))/2
        volume += (a[0]*(b[1]*c[2]-b[2]*c[1]) + a[1]*(b[2]*c[0]-b[0]*c[2]) + a[2]*(b[0]*c[1]-b[1]*c[0]))/6
    if product.is_a("IfcGeographicElement"):
        assert product.PredefinedType == "TERRAIN"
        assert bounds == [[0.0,0.0,0.0],[10.0,10.0,1.0]], bounds
        assert abs(area - 100*math.sqrt(1.01)) < 1e-7, area
        assert product.ContainedInStructure[0].RelatingStructure.is_a("IfcSite")
    elif product.is_a("IfcSlab"):
        assert product.ObjectType == "Pad" and product.PredefinedType == "BASESLAB"
        assert all(abs(a-b)<1e-8 for actual,expected in zip(bounds,[[1,1,.3],[9,9,.5]]) for a,b in zip(actual,expected)), bounds
        assert abs(abs(volume)-12.8) < 1e-7, volume
    else:
        raise AssertionError(product.is_a())
    shapes.append({"class": product.is_a(), "name": product.Name, "bounds_m": bounds, "surface_area_m2": area, "signed_volume_m3": volume})
assert len(shapes) == 2, shapes
result = {"ifcopenshell": ifcopenshell.version, "schema": f.schema, "express_errors": len(logger.statements), "global_ids": len(ids), "unique_global_ids": len(set(ids)), "shapes": shapes}
(root / "independent-ifc-results.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
print(json.dumps(result, indent=2))
