"""Reproduce the synthetic placement workflow, then prove native reopen preserves its output."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument("--cli", required=True, help="Path to the built bimcraft-cli executable")
parser.add_argument("--out", default=".", help="Folder containing reproduce.txt; outputs are written here")
args = parser.parse_args()
cli = str(Path(args.cli).resolve())
out = Path(args.out).resolve()

def run(label, arguments):
    completed = subprocess.run([cli, *arguments], cwd=out, capture_output=True, text=True, encoding="utf-8")
    (out / (label + ".log")).write_text(completed.stdout + completed.stderr, encoding="utf-8")
    if completed.returncode:
        raise RuntimeError(label + " failed; see its log")
    return [json.loads(line) for line in completed.stdout.splitlines() if line.strip()]

def render(sheet, path):
    return ["--cmd", "render " + json.dumps({"view": sheet, "path": path})]

created = run("cli-after", ["run", "--script", "reproduce.txt", "--save", "after.bimcraft",
    *render("A101", "after-titles.png"), *render("A102", "after-overflow.png"),
    *render("A101", "after-titles.pdf"), *render("A102", "after-overflow.pdf")])
reopened = run("cli-reopen", ["run", "after.bimcraft", "--save", "reopened.bimcraft",
    *render("A101", "reopened-titles.png"), *render("A102", "reopened-overflow.png")])
assert all("left_out" not in reply for reply in reopened), "native reopen lost fields"

def viewports(filename):
    model = json.loads((out / filename).read_text(encoding="utf-8"))
    return {key: value for key, value in model["elements"].items() if value["category"] == "viewport"}

assert viewports("after.bimcraft") == viewports("reopened.bimcraft"), "viewport GUIDs, positions or scales changed on reopen"
positions = [v["at"] for v in viewports("after.bimcraft").values()]
assert len(set(map(tuple, positions))) == len(positions), "automatic viewports stacked"
summary = {"native_viewports_preserved": True, "positions": positions, "pngs": {}}
for name in ["titles", "overflow"]:
    actual = (out / ("after-" + name + ".png")).read_bytes()
    assert actual == (out / ("reopened-" + name + ".png")).read_bytes(), "PNG changed on native reopen"
    summary["pngs"][name] = {"native_reopen_identical": True, "sha256": hashlib.sha256(actual).hexdigest()}
for name in ["titles", "overflow"]:
    assert (out / ("after-" + name + ".pdf")).read_bytes().startswith(b"%PDF"), "sheet PDF was not generated"
summary["pdf_generated"] = True
(out / "cli-verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary, indent=2))
