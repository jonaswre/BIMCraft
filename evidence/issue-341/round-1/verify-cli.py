"""Compare actual standalone CLI outputs for the synthetic curved-cap gap regression."""
import json
from math import sqrt
from pathlib import Path

root = Path(__file__).resolve().parent
expected = {
    "red-cli.jsonl": ([39.63, 21.81], [True, True]),
    "official-cli.jsonl": ([61.44, 0.0], [True, False]),
    "green-cli.jsonl": ([61.44, 0.0], [True, False]),
}
results = {}
for name, (areas, enclosed) in expected.items():
    rows = [json.loads(line) for line in (root / name).read_text(encoding="utf-8-sig").splitlines()]
    rooms = rows[-1]["elements"]
    assert [room["name"] for room in rooms] == ["South", "North"]
    assert [room["area_m2"] for room in rooms] == areas
    assert [room["enclosed"] for room in rooms] == enclosed
    if not enclosed[1]:
        assert rooms[1]["shares_space_with"] == rooms[0]["id"]
    results[name] = [{key: room[key] for key in ("name", "area_m2", "enclosed")} for room in rooms]
gap = ((4201.813986211087 - 4200) + (3201.7686006042572 - 3200)) / sqrt(2)
assert gap > 1.0
print(json.dumps({"actual_cap_tangent_gap_mm": gap, "cli_results": results}, indent=2))
