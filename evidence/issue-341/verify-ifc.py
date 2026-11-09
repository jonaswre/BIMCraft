"""Independent synthetic room/IFC checks. Requires IfcOpenShell 0.8.5 and pytest."""
import json
from pathlib import Path

import ifcopenshell
import ifcopenshell.geom
import ifcopenshell.util.element
import ifcopenshell.util.shape
import ifcopenshell.validate

assert ifcopenshell.version == "0.8.5", ifcopenshell.version
root = Path(__file__).resolve().parent
expected = {"South": (30.30, 28.32), "North": (31.14, 29.14)}
summary = {"ifcopenshell": ifcopenshell.version, "files": []}


def edges(ring):
    return zip(ring, ring[1:] + ring[:1])


def shared_boundary(a, b):
    length = 0.0
    for p, q in edges(a):
        for r, s in edges(b):
            for fixed, along in [(0, 1), (1, 0)]:
                if abs(p[fixed] - q[fixed]) < 1e-7 and abs(r[fixed] - s[fixed]) < 1e-7 and abs(p[fixed] - r[fixed]) < 1e-7:
                    length += max(0.0, min(max(p[along], q[along]), max(r[along], s[along]))
                                  - max(min(p[along], q[along]), min(r[along], s[along])))
    return length


def inside(point, ring):
    x, y = point
    result = False
    for (ax, ay), (bx, by) in edges(ring):
        if (ay > y) != (by > y) and x < ax + (y - ay) * (bx - ax) / (by - ay):
            result = not result
    return result


for name, measure in [("after.ifc", 0), ("after-finish.ifc", 1)]:
    model = ifcopenshell.open(str(root / name))
    logger = ifcopenshell.validate.json_logger()
    ifcopenshell.validate.validate(model, logger, express_rules=True)
    assert not logger.statements, logger.statements
    spaces = model.by_type("IfcSpace")
    assert len(spaces) == 2
    rows = []
    footprints = {}
    settings = ifcopenshell.geom.settings()
    for space in sorted(spaces, key=lambda s: s.LongName):
        gross, net = expected[space.LongName]
        qto = ifcopenshell.util.element.get_psets(space, qtos_only=True)["Qto_SpaceBaseQuantities"]
        assert abs(qto["GrossFloorArea"] - gross) < 1e-8
        assert abs(qto["NetFloorArea"] - net) < 1e-8
        shape = ifcopenshell.geom.create_shape(settings, space)
        volume = ifcopenshell.util.shape.get_volume(shape.geometry)
        area = (gross, net)[measure]
        assert abs(volume - area * 2.7) < 1e-7, (space.LongName, volume, area)
        vertices = ifcopenshell.util.shape.get_vertices(shape.geometry)
        lower = vertices.min(axis=0).tolist()
        upper = vertices.max(axis=0).tolist()
        assert abs(lower[2]) < 1e-8 and abs(upper[2] - 2.7) < 1e-8
        rows.append({"room": space.LongName, "gross_area_m2": gross, "net_area_m2": net,
                     "shape_volume_m3": volume, "bounds_m": [lower, upper]})
        solids = [item for rep in space.Representation.Representations for item in rep.Items
                  if item.is_a("IfcExtrudedAreaSolid")]
        assert len(solids) == 1
        footprints[space.LongName] = [tuple(p.Coordinates[:2]) for p in solids[0].SweptArea.OuterCurve.Points][:-1]
    shared = shared_boundary(footprints["South"], footprints["North"])
    assert abs(shared - (9700.0, 5300.0)[measure]) < 1e-6, shared
    overlap = 0
    for x in [50.0, 1000.0, 4150.0, 4250.0, 7000.0, 9550.0]:
        for y in [50.0, 1600.0, 3050.0, 3150.0, 4800.0, 6350.0]:
            covers = sum(inside((x, y), ring) for ring in footprints.values())
            overlap += covers > 1
            if measure == 0:
                assert covers == 1, (x, y, covers)
    assert overlap == 0, overlap
    summary["files"].append({"file": name, "express_problems": len(logger.statements), "rooms": rows,
                             "shared_boundary_m": shared / 1000.0, "sampled_overlaps": overlap})
# Native and IFC reopen must show two enclosed rooms at both measures, with the original drawn separator.
for name in ["after-native-reopen.jsonl", "after-ifc-reopen.jsonl"]:
    outputs = [json.loads(line) for line in (root / name).read_text(encoding="utf-8-sig").splitlines()]
    results = [v["elements"] for v in outputs if "elements" in v and v.get("count") == 2
               and v["elements"] and v["elements"][0].get("category") == "room"]
    assert len(results) == 2, (name, results)
    for measure, rooms in enumerate(results):
        assert len(rooms) == 2
        for room in rooms:
            assert room["enclosed"] and "shares_space_with" not in room
            assert abs(room["area_m2"] - expected[room["name"]][measure]) < 1e-8
    for output in outputs:
        # IFC open returns an element-count dictionary; only element-query arrays contain geometry rows.
        if not isinstance(output.get("elements"), list):
            continue
        for element in output.get("elements", []):
            if element.get("category") == "roomseparator":
                assert element["start"] == [4200.0, 3200.0]
                assert element["end"] == [9500.0, 3200.0]
                assert element["length"] == 5300.0
original = [json.loads(line) for line in (root / "after-open-before-native.jsonl").read_text(encoding="utf-8-sig").splitlines()]
assert "left_out" not in original[0]
for room in original[-1]["elements"]:
    assert room["enclosed"] and "shares_space_with" not in room
    assert abs(room["area_m2"] - expected[room["name"]][0]) < 1e-8
summary["original_native_reopen"] = "both rooms enclosed; no fields left out"
print(json.dumps(summary, indent=2))
