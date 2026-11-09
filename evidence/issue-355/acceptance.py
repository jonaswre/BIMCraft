"""Synthetic PR284 acceptance; run with --cli PATH from this artifact folder."""
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
ROOT = Path.cwd()
transcript = []


def cli(*arguments, success=True):
    process = subprocess.run([args.cli, *arguments], capture_output=True, text=True)
    transcript.append({"arguments": arguments, "exit": process.returncode,
                       "stdout": process.stdout, "stderr": process.stderr})
    if success:
        assert process.returncode == 0, process.stderr
    return [json.loads(line) for line in process.stdout.splitlines() if line.startswith("{")]


def script(name, commands):
    Path(name).write_text("\n".join(commands) + "\n", encoding="utf-8")
    return name


create = script("create.txt", [
    'wall {"points":[[0,0],[8000,0]],"height":3000}',
    'crs {"code":32632}',
    'location {"name":"Internal","east":500000000,"north":5000000000,"elevation":42000,"angle":-30,"scale":0.9996}',
    'coordinates {}',
    'save {"path":"synthetic.bimcraft"}',
])
created = cli("run", "--script", create)
reopened = cli("run", "synthetic.bimcraft", "--cmd", "coordinates {}")
assert created[-2] == reopened[-1], "native coordinates or identities changed"
report = {"ifcopenshell": ifcopenshell.version, "native_reopen": "pass", "origins": {}}

# The legacy dialect has clockwise rotation and no scale. This is only a
# synthetic georeference field payload, not a claim about other native schemas.
legacy = json.loads(Path("synthetic.bimcraft").read_text())
legacy.pop("coordinates", None)
legacy["georeference"] = {"crs": "EPSG:32632", "eastings": 500000000,
                          "northings": 5000000000, "orthogonal_height": 42000,
                          "rotation": 30}
Path("legacy-georef.bimcraft").write_text(json.dumps(legacy, indent=2) + "\n")
legacy_result = cli("run", "legacy-georef.bimcraft", "--cmd", "coordinates {}")
report["legacy_open"] = legacy_result

# Never lower the native format to pretend a newer file is compatible.
newer = dict(legacy, format=18)
Path("newer-format.bimcraft").write_text(json.dumps(newer, indent=2) + "\n")
cli("run", "newer-format.bimcraft", success=False)
assert transcript[-1]["exit"] != 0 and "newer BIMCraft" in transcript[-1]["stderr"]
report["newer_format_refused"] = True

a = math.radians(-30)


def reference(point):
    x, y, z = point
    return [500000 + 0.9996 * (x * math.cos(a) - y * math.sin(a)) / 1000,
            5000000 + 0.9996 * (x * math.sin(a) + y * math.cos(a)) / 1000,
            42 + z / 1000]


for origin in ("internal", "base", "survey", "shared"):
    output = f"{origin}.ifc"
    cli("run", "synthetic.bimcraft", "--cmd",
        f'save {{"path":"{output}","ifc_origin":"{origin}"}}')
    cli("run", output, "--cmd", "coordinates {}", "--save", f"{origin}-reopened.bimcraft")
    model = ifcopenshell.open(output)
    logger = ifcopenshell.validate.json_logger()
    ifcopenshell.validate.validate(model, logger, express_rules=True)
    ids = [entity.GlobalId for entity in model.by_type("IfcRoot")]
    assert len(ids) == len(set(ids)), "duplicate IFC GlobalId"
    settings = ifcopenshell.geom.settings()
    settings.set("use-world-coords", True)
    settings.set("convert-back-units", True)
    walls = model.by_type("IfcWall")
    assert len(walls) == 1
    shape = ifcopenshell.geom.create_shape(settings, walls[0])
    vertices = list(zip(*[iter(shape.geometry.verts)] * 3))
    assert len(vertices) >= 8
    conversion = model.by_type("IfcMapConversion")[0]
    def mapped(p):
        return list(ifcopenshell.util.geolocation.xyz2enh(*p,
            conversion.Eastings, conversion.Northings, conversion.OrthogonalHeight,
            conversion.XAxisAbscissa, conversion.XAxisOrdinate, conversion.Scale))
    expected_top = reference([0, 0, 3000])[2]
    actual_top = max(mapped(p)[2] for p in vertices)
    points = [[0, 0, 0], [8000, 0, 0], [8000, 2000, 3000]]
    comparisons = []
    if origin != "shared":
        for point in points:
            actual, expected = mapped(point), reference(point)
            comparisons.append({"internal_mm": point, "ifc_map": actual,
                "native_map": expected, "error_mm": [(x-y)*1000 for x,y in zip(actual,expected)]})
    report["origins"][origin] = {
        "schema_messages": logger.statements, "unique_global_ids": len(ids),
        "wall_vertices": len(vertices), "map_scale": conversion.Scale,
        "top_map_m": actual_top, "expected_top_map_m": expected_top,
        "top_error_mm": (actual_top-expected_top)*1000,
        "numeric_reference": comparisons,
    }

Path("cli-transcript.json").write_text(json.dumps(transcript, indent=2) + "\n")
Path("acceptance-result.json").write_text(json.dumps(report, indent=2, default=str) + "\n")
print(json.dumps(report, indent=2, default=str))
