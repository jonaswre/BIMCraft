import json
import sys
from pathlib import Path
import ifcopenshell
import ifcopenshell.geom
import ifcopenshell.validate
import ifcopenshell.util.element

filename = sys.argv[1] if len(sys.argv) > 1 else 'industrial-gates.ifc'
model = ifcopenshell.open(filename)
logger = ifcopenshell.validate.json_logger()
ifcopenshell.validate.validate(model, logger, express_rules=True)
assert not logger.statements, logger.statements
roots = model.by_type('IfcRoot')
guids = [r.GlobalId for r in roots]
assert len(guids) == len(set(guids)), 'duplicate GlobalId'
settings = ifcopenshell.geom.settings()
settings.set(settings.USE_WORLD_COORDS, True)
rows = []
for door in model.by_type('IfcDoor'):
    shape = ifcopenshell.geom.create_shape(settings, door)
    coords = shape.geometry.verts
    lo = [min(coords[i::3]) for i in range(3)]
    hi = [max(coords[i::3]) for i in range(3)]
    size = [hi[i] - lo[i] for i in range(3)]
    kind = ifcopenshell.util.element.get_type(door)
    qto = ifcopenshell.util.element.get_psets(door)['Qto_DoorBaseQuantities']
    if door.Name in ('Sectional gate', 'Roller gate'):
        expected = {'Sectional gate':'USERDEFINED', 'Roller gate':'ROLLINGUP'}[door.Name]
        assert door.OperationType == expected
        assert kind.OperationType == expected
        if door.Name == 'Sectional gate':
            assert door.UserDefinedOperationType == 'SECTIONAL'
            assert kind.UserDefinedOperationType == 'SECTIONAL'
        assert all(abs(a-b) < 1e-8 for a,b in zip(size, [3.2,3.28,3.3] if door.Name == 'Sectional gate' else [3.2,0.45,3.65])), size
        assert abs(qto['Area'] - 9.6) < 1e-8, qto
        assert abs(qto['Width'] - 3000) < 1e-8, qto
        assert abs(qto['Height'] - 3200) < 1e-8, qto
    rows.append(dict(name=door.Name, global_id=door.GlobalId, operation=door.OperationType,
                     type_operation=kind.OperationType, bounds_m=[lo,hi], size_m=size,
                     area_m2=qto['Area']))
assert {r['operation'] for r in rows} >= {'USERDEFINED','ROLLINGUP','SINGLE_SWING_LEFT'}
assert len(model.by_type('IfcWindow')) == 1
print(json.dumps(dict(file=filename,ifcopenshell_version=ifcopenshell.version, schema=model.schema,
                     schema_express_errors=logger.statements, roots=len(roots), unique_global_ids=len(set(guids)),
                     doors=rows), indent=2))
