import json
import pathlib
import subprocess
import sys

cli = sys.argv[1]
root = pathlib.Path(__file__).resolve().parent

def run(*args):
    p = subprocess.run([cli, *args], cwd=root, text=True, capture_output=True, check=True)
    return [json.loads(line) for line in p.stdout.splitlines() if line.strip()]

created = run("run", "--script", "survey.txt")
assert created[2]["heights"] == [100, 900, None], created[2]
assert created[3]["totals"] == {"cut_m3": 6.4, "fill_m3": 6.4, "net_m3": 0.0}, created[3]
reopened = run("run", "survey.bimcraft", "--cmd", 'cut_fill {}', "--cmd", 'height_at {"points":[[1000,5000],[9000,5000]]}')
assert reopened[1]["totals"] == created[3]["totals"], reopened
assert reopened[2]["heights"] == [100, 900], reopened
ifc = run("run", "survey.ifc", "--cmd", 'cut_fill {}', "--cmd", 'elements {"category":"toposurface"}', "--cmd", 'elements {"category":"pad"}')
assert ifc[0]["skipped"] == [], ifc
assert ifc[1]["totals"] == created[3]["totals"], ifc
assert ifc[2]["count"] == 1 and ifc[3]["count"] == 1, ifc
(root / "cli-results.json").write_text(json.dumps({"created": created, "native_reopen": reopened, "ifc_reopen": ifc}, indent=2), encoding="utf-8")
print("PASS: CLI create, inspect, schedule, save, native reopen, IFC reopen and render")
