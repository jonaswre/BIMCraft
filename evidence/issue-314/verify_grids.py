"""Synthetic command -> IFC -> independent IfcOpenShell read -> command reopen, with schema/shape validation."""
import json
from pathlib import Path
import subprocess
import sys

import ifcopenshell
import ifcopenshell.util.placement
import numpy as np

assert ifcopenshell.version == "0.8.5", ifcopenshell.version
repo, cli = Path(sys.argv[1]), Path(sys.argv[2])
out = Path(__file__).parent


def execute(name, args):
    result = subprocess.run([str(cli), *map(str, args)], cwd=repo, text=True, capture_output=True)
    (out / f"{name}.stdout").write_text(result.stdout, encoding="utf-8")
    (out / f"{name}.stderr").write_text(result.stderr, encoding="utf-8")
    assert result.returncode == 0, result.stderr
    return [json.loads(line) for line in result.stdout.splitlines() if line.startswith("{")]


def points(curve, placement, sense=True):
    matrix = ifcopenshell.util.placement.get_local_placement(placement)
    coords = [p.Coordinates for p in curve.Points]
    a, b = coords[0], coords[-1]
    values = [(matrix @ np.array([p[0], p[1], 0.0, 1.0]))[:2].tolist() for p in (a, b)]
    return values if sense else values[::-1]


fixtures = {
    "oblique": [("U'1", [-500, 100], [2500, 4100]), ("U2", [3500, 4100], [500, 100]), ("V", [0, 2000], [4000, -1000])],
    "parallel": [(str(i + 1), [x, 0], [x, 3000]) for i, x in enumerate((0, 1000, 2000))],
    "single": [("Solo", [100, 200], [3100, 4200])],
    "empty": [],
}
results = []
files = []
for name, axes in fixtures.items():
    script = out / f"{name}.txt"
    script.write_text("\n".join("grid " + json.dumps(dict(name=n, start=a, end=b)) for n, a, b in axes), encoding="utf-8")
    path = out / f"{name}.ifc"
    files.append(path)
    execute(f"{name}-save", ["run", "--script", script, "--save", out / f"{name}.bimcraft", "--save", path])
    f = ifcopenshell.open(str(path))
    actual = []
    grid_products = f.by_type("IfcGrid")
    if len(axes) >= 2:
        assert len(grid_products) == 1
        grid = grid_products[0]
        assert grid.UAxes and grid.VAxes and not grid.WAxes
        assert set(grid.UAxes).isdisjoint(grid.VAxes)
        assert len(f.by_type("IfcGridAxis")) == len(axes)
        for axis in (*grid.UAxes, *grid.VAxes):
            assert len(axis.PartOfU) + len(axis.PartOfV) + len(axis.PartOfW) == 1
            assert axis.AxisCurve.Dim == 2
            actual.append((axis.AxisTag, *points(axis.AxisCurve, grid.ObjectPlacement, axis.SameSense)))
    elif axes:
        assert not grid_products and not f.by_type("IfcGridAxis")
        annotations = [a for a in f.by_type("IfcAnnotation") if a.ObjectType == "BIMCraft_GridAxis"]
        assert len(annotations) == 1
        annotation = annotations[0]
        actual.append((annotation.Name, *points(annotation.Representation.Representations[0].Items[0], annotation.ObjectPlacement)))
    else:
        assert not grid_products and not f.by_type("IfcGridAxis") and not f.by_type("IfcAnnotation")
    assert sorted(actual) == sorted(axes), (name, actual)
    reopened = execute(f"{name}-open", ["run", path, "--cmd", 'elements {"category":"grid"}'])
    native = next(reply["elements"] for reply in reopened if "elements" in reply and "count" in reply)
    back = [(g["name"], g["start"], g["end"]) for g in native]
    assert sorted(back) == sorted(axes), (name, back)
    results.append(dict(fixture=path.name, independently_read=actual, reopened=back))

for name, args in (("house", ["--sample"]), ("coverage", ["--script", repo / "ci/ifc-coverage.txt"])):
    path = out / f"{name}.ifc"
    execute(f"{name}-save", ["run", *args, "--save", path])
    files.append(path)
    if name == "house":
        replies = execute("house-open", ["run", path, "--cmd", 'elements {"category":"grid"}'])
        assert next(r["count"] for r in replies if "count" in r and "elements" in r) > 0

validation = subprocess.run([sys.executable, str(repo / "ci/validate_ifc.py"), *map(str, files)], cwd=repo, capture_output=True, text=True)
(out / "ifc-validation.log").write_text(validation.stdout + validation.stderr, encoding="utf-8")
assert validation.returncode == 0, validation.stdout + validation.stderr
(out / "independent-grid-results.json").write_text(json.dumps(dict(ifcopenshell=ifcopenshell.version, fixtures=results), indent=2), encoding="utf-8")
print(validation.stdout)
print("Independent axis labels/endpoints and CLI reopen agree for all four grid fixtures.")
