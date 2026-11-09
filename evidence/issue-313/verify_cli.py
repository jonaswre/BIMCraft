"""Synthetic CLI journeys. Run with a CLI executable and output directory."""
import json
from pathlib import Path
import subprocess
import sys

from PIL import Image, ImageDraw

exe = Path(sys.argv[1]).resolve()
out = Path(sys.argv[2]).resolve()
out.mkdir(parents=True, exist_ok=True)
box = 'wall {"points":[[0,0],[8000,0],[8000,6000],[0,6000]],"closed":true}'


def journey(name, commands):
    script = out / f"{name}.txt"
    script.write_text("\n".join(commands) + "\n", encoding="utf-8")
    result = subprocess.run([str(exe), "run", "--script", script.name], cwd=out, capture_output=True, text=True)
    (out / f"{name}.stdout").write_text(result.stdout, encoding="utf-8")
    (out / f"{name}.stderr").write_text(result.stderr, encoding="utf-8")
    assert result.returncode == 0, result.stderr
    return [json.loads(line) for line in result.stdout.splitlines()]


def render(view, path):
    return "render " + json.dumps(dict(view=view, path=path, width=400, height=300))


def cuts(name):
    with Image.open(out / name) as image:
        image = image.convert("RGB")
        return sum(pixel == (168, 168, 168) for pixel in image.crop((0, 0, 400, 240)).getdata())


graphics = journey("graphics", [box,
    'duplicate_view {"view":"Level 1","name":"No walls"}',
    'graphics {"view":"No walls","hide":["wall"]}',
    render("Level 1", "parent.png"), render("No walls", "duplicate.png"),
    render(100006, "duplicate-by-id.png"), render("Level 1", "parent-after.png")])
crop = journey("crop", [box,
    'duplicate_view {"view":"Level 1","name":"Interior"}',
    'crop {"view":"Interior","region":[[1000,1000],[3000,2000]]}',
    render("Interior", "crop.png"), render("Level 1", "crop-parent.png")])
range_replies = journey("range", [
    'wall {"points":[[0,0],[4000,0]],"height":900}',
    'duplicate_view {"view":"Level 1","name":"Low cut"}',
    'modify {"id":100003,"view_range":{"cut":500}}',
    render("Level 1", "range-parent.png"), render(100003, "range-copy.png")])

counts = {name: cuts(name) for name in ["parent.png", "duplicate.png", "duplicate-by-id.png", "parent-after.png",
    "crop.png", "crop-parent.png", "range-parent.png", "range-copy.png"]}
summary = dict(cut_pixels_above_title=counts, graphics_render_names=[r["view"] for r in graphics[-4:]],
    crop_render_names=[r["view"] for r in crop[-2:]], range_render_names=[r["view"] for r in range_replies[-2:]])
(out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary, indent=2))

montage = Image.new("RGB", (800, 660), "#eeeeee")
draw = ImageDraw.Draw(montage)
for i, name in enumerate(["parent.png", "duplicate.png", "range-parent.png", "range-copy.png"]):
    x, y = (i % 2) * 400, (i // 2) * 330
    draw.text((x + 8, y + 5), name, fill="black")
    with Image.open(out / name) as image:
        montage.paste(image.convert("RGB"), (x, y + 25))
montage.save(out / "montage.png")

assert counts["parent.png"] > 1000
assert counts["duplicate.png"] == counts["duplicate-by-id.png"] == 0
assert (out / "parent.png").read_bytes() == (out / "parent-after.png").read_bytes()
assert summary["graphics_render_names"] == ["Level 1", "No walls", "No walls", "Level 1"]
assert counts["crop.png"] == 0 and counts["crop-parent.png"] > 1000
assert counts["range-parent.png"] == 0 and counts["range-copy.png"] > 1000
assert summary["crop_render_names"] == ["Interior", "Level 1"]
assert summary["range_render_names"] == ["Level 1", "Low cut"]
print("All CLI journeys passed.")
