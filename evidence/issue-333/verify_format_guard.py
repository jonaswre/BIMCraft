"""Run a format-18 CLI against the steel format-19 native file.
Usage: python verify_format_guard.py LEGACY_CLI steel.bimcraft
The older binary must fail before dropping unknown profile geometry.
"""
from pathlib import Path
import subprocess
import sys

legacy, native = map(lambda s: Path(s).resolve(), sys.argv[1:3])
r = subprocess.run([str(legacy), "run", native.name], cwd=native.parent, capture_output=True, text=True, encoding="utf-8")
print("Legacy reader exit:", r.returncode)
print(r.stdout, end="")
print(r.stderr, end="")
assert r.returncode != 0
assert "file format 19" in r.stderr and "reads up to 18" in r.stderr
assert "update BIMCraft" in r.stderr
print("PASS: older reader refuses format 19 before geometry can be reinterpreted.")
