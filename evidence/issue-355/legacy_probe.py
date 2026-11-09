"""Synthetic current-main georeference reproduction; no general file converter."""
import argparse
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument("--cli", required=True)
args = parser.parse_args()
transcript = []


def cli(*arguments):
    process = subprocess.run([args.cli, *arguments], capture_output=True, text=True)
    transcript.append({"arguments": arguments, "exit": process.returncode,
                       "stdout": process.stdout, "stderr": process.stderr})
    assert process.returncode == 0, process.stderr


cli("run", "--cmd", 'wall {"points":[[0,0],[8000,0]],"height":3000}',
    "--save", "main-seed.bimcraft")
seed = json.loads(Path("main-seed.bimcraft").read_text())
seed["georeference"] = {"crs": "EPSG:32632", "eastings": 500000000,
                        "northings": 5000000000, "orthogonal_height": 42000,
                        "rotation": 30}
Path("main-legacy.bimcraft").write_text(json.dumps(seed, indent=2) + "\n")
cli("run", "main-legacy.bimcraft", "--save", "main-legacy-reopened.bimcraft",
    "--save", "main-legacy.ifc")
result = json.loads(Path("main-legacy-reopened.bimcraft").read_text())["georeference"]
Path("main-legacy-transcript.json").write_text(json.dumps(transcript, indent=2) + "\n")
Path("main-legacy-result.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps(result, indent=2))
assert result["height"] == 42000, f'height lost: {result["height"]}, expected 42000'
assert result["rotation"] == -30, f'wrong rotation: {result["rotation"]}, expected -30'
assert result["scale"] == 1, "legacy scale default changed"
print("PASS: legacy height and rotation preserved")
