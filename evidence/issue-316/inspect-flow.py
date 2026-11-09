import json
from pathlib import Path

import ifcopenshell
import ifcopenshell.geom
from PIL import Image, ImageChops

root = Path(__file__).resolve().parent
created = [json.loads(line) for line in (root / 'source-flow.stdout').read_text(encoding='utf-8-sig').splitlines()]
reopened = [json.loads(line) for line in (root / 'source-reopen.stdout').read_text(encoding='utf-8-sig').splitlines()]
created_elements = next(item for item in created if 'elements' in item)
reopened_elements = next(item for item in reopened if 'elements' in item)
assert created_elements == reopened_elements, 'native reopen changed query output'
assert reopened[0].get('left_out', []) == [], reopened[0]
print('Native save/reopen: identical elements, GUIDs and host references; no left_out fields')

before = Image.open(root / 'before.png').convert('RGBA')
after = Image.open(root / 'ground-plan.png').convert('RGBA')
back = Image.open(root / 'reopened-plan.png').convert('RGBA')
assert list(after.getdata()) == list(back.getdata()), 'reopened plan pixels differ'
changed = sum(a != b for a, b in zip(before.getdata(), after.getdata()))
assert changed > 100, changed
print(f'Plan: {after.width}x{after.height}, {changed} changed pixels after openings; reopened pixels identical')

model = ifcopenshell.open(str(root / 'openings.ifc'))
walls = model.by_type('IfcWall')
doors = model.by_type('IfcDoor')
windows = model.by_type('IfcWindow')
assert (len(walls), len(doors), len(windows)) == (1, 1, 1)
settings = ifcopenshell.geom.settings()
settings.set(settings.USE_WORLD_COORDS, True)
for element in [*doors, *windows]:
    assert len(element.FillsVoids) == 1
    opening = element.FillsVoids[0].RelatingOpeningElement
    assert len(opening.VoidsElements) == 1
    assert opening.VoidsElements[0].RelatingBuildingElement == walls[0]
    shape = ifcopenshell.geom.create_shape(settings, element)
    verts = shape.geometry.verts
    assert len(verts) >= 24
    print(f'{element.is_a()}: valid filled opening/void/host relations; geometry {len(verts) // 3} vertices, z {min(verts[2::3]):g}..{max(verts[2::3]):g} m')
shape = ifcopenshell.geom.create_shape(settings, walls[0])
assert len(shape.geometry.verts) >= 24
print(f'IfcWall: boolean geometry generated, {len(shape.geometry.verts) // 3} vertices')
