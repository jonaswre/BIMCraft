"""Synthetic F1 before/after CLI journey. Usage: verify_cli.py CLI before|after.
The CLI path is runtime-only and is never written to evidence.
"""
import hashlib, json, os, pathlib, shutil, subprocess, sys
root = pathlib.Path(__file__).resolve().parent
cli, phase = sys.argv[1:]
assert phase in ("before", "after")
original = root.parent / "combined-artifacts" / "mixed.bimcraft"
fixture = root / "synthetic-mixed.bimcraft"
if not fixture.exists():
    shutil.copyfile(original, fixture)
assert fixture.read_bytes() == original.read_bytes()
args = ["run", fixture.name,
        "--cmd", 'render ' + json.dumps({"view":"Level 1","path":f"{phase}-plan.png","width":1200,"height":800}),
        "--cmd", 'render ' + json.dumps({"view":"S-01","path":f"{phase}-sheet.png"}),
        "--cmd", 'render ' + json.dumps({"view":"S-01","path":f"{phase}-sheet.pdf"}),
        "--save", f"{phase}-reopened.bimcraft"]
result = subprocess.run([cli, *args], cwd=root, text=True, encoding="utf-8", capture_output=True, env={**os.environ, "RAYON_NUM_THREADS":"2"})
log = "COMMAND bimcraft-cli " + " ".join(args) + "\n" + result.stdout + result.stderr
(root / f"{phase}-cli.log").write_text(log, encoding="utf-8")
print(log, end="")
assert result.returncode == 0, result.returncode
source = json.loads(fixture.read_text(encoding="utf-8-sig"))
reopened = json.loads((root / f"{phase}-reopened.bimcraft").read_text(encoding="utf-8-sig"))
assert source == reopened, "render/native reopen must not change any persisted value"
summary = {"phase":phase,"native_all_fields_equal":True,"format":source["format"],
    "input_sha256":hashlib.sha256(fixture.read_bytes()).hexdigest(),
    "cli_sha256":hashlib.sha256(pathlib.Path(cli).read_bytes()).hexdigest(),
    "rooms_and_properties":{k:v for k,v in source["elements"].items() if v["category"] in ("room","propertyline")}}
(root / f"{phase}-invariance.json").write_text(json.dumps(summary,indent=2)+"\n", encoding="utf-8")
print("PASS exact native equality; unchanged room identity, point, boundary and area inputs")
