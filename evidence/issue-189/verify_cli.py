"""Synthetic CLI save/reopen and old-reader gate check. Pass new CLI, old format18 CLI."""
import json
import pathlib
import subprocess
import sys
root = pathlib.Path(__file__).resolve().parent
cli, old = sys.argv[1:3]
def run(exe, *args, ok=True):
    p = subprocess.run([exe, *args], cwd=root, text=True, capture_output=True)
    print("COMMAND", *args)
    print(p.stdout, end="")
    if p.stderr: print(p.stderr, end="")
    if ok: assert p.returncode == 0, p.returncode
    return p
run(cli, "run", "--script", "open-stair.txt", "--save", "open-stair.bimcraft", "--save", "open-stair.ifc", "--save", "open-stair.glb", "--cmd", 'render {"view":"{3D}","path":"open-stair.png","azimuth":135,"elevation":25,"width":1200,"height":900}')
native=json.loads((root/"open-stair.bimcraft").read_text())
assert native["format"] == 19
stair=next(e for e in native["elements"].values() if e["category"]=="stair")
assert stair["open"] is True
for ext in ["bimcraft", "ifc"]:
    result=run(cli, "run", f"open-stair.{ext}", "--cmd", 'elements {"category":"stair"}', "--save", f"reopened-{ext}.bimcraft", "--save", f"reopened-{ext}.ifc", "--save", f"reopened-{ext}.glb")
    rows=[json.loads(line) for line in result.stdout.splitlines()]
    got=next(v["elements"][0] for v in rows if isinstance(v.get("elements"), list))
    assert got["open"] is True and got["guid"]==stair["guid"]
    assert got["risers"]==16 and got["landings"][0]["turn"]==90
refusal=run(old,"run","open-stair.bimcraft",ok=False)
assert refusal.returncode != 0 and "file format 19" in refusal.stderr + refusal.stdout
run(cli,"run","baseline-solid.bimcraft","--save","closed-preserved.ifc")
run(old,"run","baseline-solid.ifc","--save","baseline-ifc-reopened.ifc")
assert (root/"closed-preserved.ifc").read_text().split("DATA;",1)[1]==(root/"baseline-solid.ifc").read_text().split("DATA;",1)[1], "closed IFC DATA must remain byte-identical (header timestamp varies)"
print("PASS: native/IFC reopen retains open construction and GUID; format18 refuses new file; closed IFC DATA byte-identical")
