"""Run synthetic CLI save/reopen, output, and failed-update journeys.
Usage: python verify_cli.py /path/to/bimcraft-cli [output-directory]
"""
import json
import math
from pathlib import Path
import subprocess
import sys

cli = str(Path(sys.argv[1]).resolve())
out = Path(sys.argv[2] if len(sys.argv) > 2 else ".").resolve()
out.mkdir(parents=True, exist_ok=True)

def invoke(name, *args, success=True):
    result = subprocess.run([cli, *map(str, args)], capture_output=True, text=True)
    (out / f"{name}.stdout").write_text(result.stdout.replace(json.dumps(str(out))[1:-1], ".").replace(str(out), "."), encoding="utf-8")
    (out / f"{name}.stderr").write_text(result.stderr, encoding="utf-8")
    assert (result.returncode == 0) == success, (name, result.returncode, result.stderr)
    return [json.loads(line) for line in result.stdout.splitlines() if line.strip()]

boundary = [[0,0],[10000,0],[10000,20000],[0,20000]]
params = dict(level="Level 1", boundary=boundary, slope=10, gables=[0,1,2], overhang=750, offset=6000)
model = out / "synthetic-roof.bimcraft"
rows = invoke("cli-create", "run", "--cmd", "roof " + json.dumps(params), "--save", model,
              "--save", out / "synthetic-roof.ifc", "--cmd", "elements {\"category\":\"roof\"}",
              "--cmd", "render " + json.dumps(dict(view="{3D}",path=str(out / "synthetic-roof-3d.png"),width=1000,height=750)))
roof = next(row["elements"][0] for row in rows if "elements" in row)
assert roof["outline"] == [[-750.,-750.],[10750.,-750.],[10750.,20750.],[-750.,20750.]], roof
assert abs(roof["ridge"] - (6000 + 10750 * math.tan(math.radians(10)))) < .02, roof
reopened = invoke("cli-reopen", "run", model, "--cmd", "elements {\"category\":\"roof\"}")[-1]["elements"][0]
assert reopened == roof

mass = dict(boundary=[[0,0],[8000,0],[8000,6000],[0,6000]], height=3000)
rows = invoke("cli-mass", "run", "--cmd", "mass " + json.dumps(mass))
mass_id = rows[0]["id"]
# Fresh models start with the same numeric ids; references in persisted native files remain GUIDs.
mass_model = out / "synthetic-mass-roof.bimcraft"
rows = invoke("cli-mass-roof", "run", "--cmd", "mass " + json.dumps(mass), "--cmd",
              "roof_by_face " + json.dumps(dict(mass=mass_id, overhang=-400)), "--save", mass_model)
original = mass_model.read_bytes()
small = dict(id=mass_id, boundary=[[0,0],[600,0],[600,600],[0,600]])
rows = invoke("cli-invalid-update", "run", mass_model, "--cmd", "modify " + json.dumps(small),
              "--cmd", "update_by_face " + json.dumps(dict(mass=mass_id)), "--save", mass_model, success=False)
assert "overhang" in (out / "cli-invalid-update.stderr").read_text(encoding="utf-8")
assert mass_model.read_bytes() == original
invoke("cli-mass-reopen", "run", mass_model, "--cmd", "elements {\"category\":\"roof\"}")
summary = dict(result="passed", ridge_mm=roof["ridge"], outline_mm=roof["outline"],
               native_reopen_equal=True, failed_update_preserved_saved_file=True)
(out / "cli-verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
