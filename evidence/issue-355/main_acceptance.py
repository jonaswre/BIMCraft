"""Synthetic legacy migration through the actual CLI and independent IFC reader."""
import argparse
import json
import math
from pathlib import Path
import subprocess

import ifcopenshell
import ifcopenshell.geom
import ifcopenshell.util.geolocation
import ifcopenshell.validate

parser = argparse.ArgumentParser()
parser.add_argument("--cli", required=True)
args = parser.parse_args()
transcript, results = [], []


def cli(*arguments):
    p = subprocess.run([args.cli, *arguments], capture_output=True, text=True)
    transcript.append({"arguments": arguments, "exit": p.returncode, "stdout": p.stdout, "stderr": p.stderr})
    assert p.returncode == 0, p.stderr


cli("run", "--cmd", 'wall {"points":[[0,0],[8000,0]],"height":3000}', "--save", "migration-seed.bimcraft")
seed = json.loads(Path("migration-seed.bimcraft").read_text())
for name, scale in (("default", 1.0), ("scaled", 0.9996)):
    base = f"migration-{name}"
    legacy = dict(seed)
    legacy["georeference"] = {"crs": "EPSG:32632", "eastings": 500000000,
        "northings": 5000000000, "orthogonal_height": 42000, "rotation": 30}
    if name == "scaled":
        legacy["georeference"]["scale"] = scale
    Path(f"{base}-legacy.bimcraft").write_text(json.dumps(legacy, indent=2) + "\n")
    cli("run", f"{base}-legacy.bimcraft", "--save", f"{base}.bimcraft", "--save", f"{base}.ifc")
    cli("run", f"{base}.bimcraft", "--save", f"{base}-native-reopened.bimcraft")
    cli("run", f"{base}.ifc", "--save", f"{base}-ifc-reopened.bimcraft")
    g = json.loads(Path(f"{base}-native-reopened.bimcraft").read_text())["georeference"]
    assert g["height"] == 42000 and g["rotation"] == -30 and g["scale"] == scale, g
    imported = json.loads(Path(f"{base}-ifc-reopened.bimcraft").read_text())["georeference"]
    for key in ("height", "rotation", "scale"):
        assert abs(imported[key] - g[key]) < 1e-7, (key, imported[key], g[key])
    model = ifcopenshell.open(f"{base}.ifc")
    logger = ifcopenshell.validate.json_logger()
    ifcopenshell.validate.validate(model, logger, express_rules=True)
    assert not logger.statements, logger.statements
    ids = [e.GlobalId for e in model.by_type("IfcRoot")]
    assert len(ids) == len(set(ids))
    conversion = model.by_type("IfcMapConversion")[0]
    assert abs(conversion.Scale - scale/1000) < 1e-15
    assert model.by_type("IfcProjectedCRS")[0].Name == "EPSG:32632"
    settings = ifcopenshell.geom.settings()
    settings.set("use-world-coords", True)
    settings.set("convert-back-units", True)
    walls = model.by_type("IfcWall")
    assert len(walls) == 1
    shape = ifcopenshell.geom.create_shape(settings, walls[0])
    vertices = list(zip(*[iter(shape.geometry.verts)]*3))
    bounds = [[min(p[i] for p in vertices) for i in range(3)], [max(p[i] for p in vertices) for i in range(3)]]
    assert abs(bounds[1][0] - bounds[0][0] - 8000) < 1e-6, bounds
    assert abs(bounds[1][2] - bounds[0][2] - 3000) < 1e-6, bounds
    angle = math.radians(30)
    checks = []
    # Independent clockwise legacy formula, not the implementation's CCW helper.
    for point in [*vertices, (8000, 2000, 3000)]:
        x,y,z = point
        expected = [500000 + scale*(x*math.cos(angle)+y*math.sin(angle))/1000,
            5000000 + scale*(-x*math.sin(angle)+y*math.cos(angle))/1000,
            42 + scale*z/1000]
        actual = ifcopenshell.util.geolocation.xyz2enh(x,y,z, conversion.Eastings,
            conversion.Northings, conversion.OrthogonalHeight, conversion.XAxisAbscissa,
            conversion.XAxisOrdinate, conversion.Scale)
        error = max(abs(a-b)*1000 for a,b in zip(actual,expected))
        assert error < 0.001, (point, actual, expected)
        checks.append({"point_mm": point, "map_m": list(actual), "maximum_error_mm": error})
    results.append({"case": name, "native": g, "ifc_reopened": imported,
        "ifc_schema_messages": logger.statements, "unique_global_ids": len(ids),
        "wall_bounds_mm": bounds, "wall_vertices": len(vertices), "numeric_checks": checks})

Path("main-cli-transcript.json").write_text(json.dumps(transcript, indent=2) + "\n")
report = {"ifcopenshell": ifcopenshell.version, "cases": results}
Path("main-acceptance-result.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report, indent=2))
