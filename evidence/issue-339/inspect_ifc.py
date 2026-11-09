"""Independent IFC 0.8.5 check; usage: python inspect_ifc.py red|green."""
import json, pathlib, sys
import ifcopenshell, ifcopenshell.geom, ifcopenshell.validate
import ifcopenshell.util.shape
root = pathlib.Path(__file__).parent.resolve()
phase = sys.argv[1]
assert ifcopenshell.version == '0.8.5', ifcopenshell.version
results = {'ifcopenshell_version': ifcopenshell.version}
for name in [phase+'.ifc', phase+'-reopened.ifc']:
    f = ifcopenshell.open(str(root/name))
    settings = ifcopenshell.geom.settings()
    settings.set(settings.USE_WORLD_COORDS, True)
    products = []
    for p in f.by_type('IfcProduct'):
        if not p.Representation: continue
        shape = ifcopenshell.geom.create_shape(settings, p)
        vertices = shape.geometry.verts
        lo = [min(vertices[i::3]) for i in range(3)]
        hi = [max(vertices[i::3]) for i in range(3)]
        products.append(dict(type=p.is_a(), bounds_min_m=lo, bounds_max_m=hi,
                             volume_m3=ifcopenshell.util.shape.get_volume(shape.geometry)))
    logger = ifcopenshell.validate.json_logger()
    ifcopenshell.validate.validate(f, logger, express_rules=True)
    results[name] = dict(products=products, validation_statements=len(logger.statements))
    window = next(p for p in products if p['type']=='IfcWindow')
    assert abs(window['bounds_min_m'][2] - (0.9 if phase=='red' else 4.3)) < 1e-6, window
    assert abs(window['bounds_max_m'][2] - (2.4 if phase=='red' else 5.8)) < 1e-6, window
    void = next(p for p in products if p['type']=='IfcOpeningElement')
    assert abs(void['bounds_min_m'][2] - window['bounds_min_m'][2]) < 1e-6, void
    wall = next(p for p in products if p['type']=='IfcWall')
    assert abs(wall['volume_m3'] - 11.16) < 1e-6, wall  # 8*.2*7.2 - 1.2*.2*1.5
    assert not logger.statements, logger.statements
(root/(phase+'-ifc-check.json')).write_text(json.dumps(results, indent=2), encoding='utf-8')
print(json.dumps(results, indent=2))
