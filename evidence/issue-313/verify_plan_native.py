"""Actual CLI: configure duplicated views, save native project, reopen and compare rendered output."""
from pathlib import Path
import json
import subprocess
import sys

cli, out = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
out.mkdir(parents=True, exist_ok=True)

def execute(name, commands, project=None, save=None):
    script = out / f"{name}.txt"
    script.write_text("\n".join(commands) + "\n", encoding="utf-8")
    args = [str(cli), "run"] + ([project] if project else []) + ["--script", script.name]
    if save:
        args += ["--save", save]
    result = subprocess.run(args, cwd=out, capture_output=True, text=True)
    (out / f"{name}.stdout").write_text(result.stdout, encoding="utf-8")
    (out / f"{name}.stderr").write_text(result.stderr, encoding="utf-8")
    assert result.returncode == 0, result.stderr
    return [json.loads(line) for line in result.stdout.splitlines()]

setup = execute("native-setup", [
    'wall {"points":[[0,0],[8000,0],[8000,6000],[0,6000]],"closed":true}',
    'wall {"points":[[0,8000],[4000,8000]],"height":900}',
    'duplicate_view {"view":"Level 1","name":"No walls"}',
    'graphics {"view":"No walls","hide":["wall"]}',
    'duplicate_view {"view":"Level 1","name":"Interior"}',
    'crop {"view":"Interior","region":[[1000,1000],[3000,2000]]}',
    'duplicate_view {"view":"Level 1","name":"Low cut"}',
], save="configured.bimcraft")
low_cut = setup[-2]["id"]
views = ["Level 1", "No walls", "Interior", "Low cut"]
def renders(suffix):
    return ["render " + json.dumps(dict(view=view, width=400, height=300, path=f"native-{i}-{suffix}.png")) for i, view in enumerate(views)]
before = execute("native-before", ['modify ' + json.dumps(dict(id=low_cut, view_range=dict(cut=500)))] + renders("before"),
                 project="configured.bimcraft", save="saved.bimcraft")
after = execute("native-after", renders("after"), project="saved.bimcraft")
assert [r["view"] for r in before[-5:-1]] == views, before
assert [r["view"] for r in after[-4:]] == views, after
for i, view in enumerate(views):
    assert (out / f"native-{i}-before.png").read_bytes() == (out / f"native-{i}-after.png").read_bytes(), view
print("Native save/reopen preserves all four rendered plans, view names, hidden walls, crop and cut range.")
