import json
import sys
from pathlib import Path
import ifcopenshell
import ifcopenshell.geom
import ifcopenshell.util.element

path = Path(sys.argv[1])
model = ifcopenshell.open(str(path))
spaces = model.by_type('IfcSpace')
assert len(spaces) == 1, len(spaces)
quantities = ifcopenshell.util.element.get_psets(spaces[0], qtos_only=True)['Qto_SpaceBaseQuantities']
assert abs(quantities['GrossFloorArea'] - 154.44) < 1e-6, quantities
assert abs(quantities['NetFloorArea'] - 148.96) < 1e-6, quantities
shape = ifcopenshell.geom.create_shape(ifcopenshell.geom.settings(), spaces[0])
assert len(shape.geometry.verts) > 0
assert len(shape.geometry.faces) > 0
print(json.dumps({'spaces': len(spaces), 'gross_floor_area_m2': quantities['GrossFloorArea'],
                  'net_floor_area_m2': quantities['NetFloorArea'],
                  'room_mesh_vertices': len(shape.geometry.verts) // 3,
                  'room_mesh_faces': len(shape.geometry.faces) // 3}, indent=2))
