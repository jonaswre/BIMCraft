"""Regression oracle: IFC map elevations must match native shared elevations."""
import json
from pathlib import Path

result = json.loads(Path("acceptance-result.json").read_text())
for origin, data in result["origins"].items():
    error = data["top_error_mm"]
    assert abs(error) < 0.001, f"{origin}: IFC top differs from native shared elevation by {error:.6f} mm"
print("PASS: all IFC map elevations match native shared elevations")
