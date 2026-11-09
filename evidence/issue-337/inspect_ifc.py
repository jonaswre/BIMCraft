"""Inspect synthetic mono-pitch roof IFC with IfcOpenShell 0.8.5.
Usage: python inspect_ifc.py synthetic-roof.ifc
"""
import json
from pathlib import Path
import sys

import ifcopenshell
import ifcopenshell.geom
import ifcopenshell.validate

path = Path(sys.argv[1])
f = ifcopenshell.open(str(path))
logger = ifcopenshell.validate.json_logger()
ifcopenshell.validate.validate(f, logger, express_rules=True)
assert not logger.statements, logger.statements
roofs = f.by_type("IfcRoof")
assert len(roofs) == 1
settings = ifcopenshell.geom.settings()
shape = ifcopenshell.geom.create_shape(settings, roofs[0])
coords = shape.geometry.verts
points = [coords[i:i+3] for i in range(0, len(coords), 3)]
low = [min(p[i] for p in points) for i in range(3)]
high = [max(p[i] for p in points) for i in range(3)]
for got, want in zip(low[:2] + high[:2], [-.75, -.75, 10.75, 20.75]):
    assert abs(got - want) < 1e-6, (low, high)
result = dict(ifcopenshell_version=ifcopenshell.version, express_problems=len(logger.statements),
              roof_count=len(roofs), mesh_vertices=len(points), bbox_min_m=low, bbox_max_m=high)
print(json.dumps(result, indent=2))
