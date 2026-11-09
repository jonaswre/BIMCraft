# Explicit roof overhang reproduction and verification

Tracks [IPedrax/BIMCraft issue #337](https://github.com/IPedrax/BIMCraft/issues/337).
All models and images here use arbitrary synthetic rectangles, unrelated to any customer project.

`repro.txt` reproduces the incorrect explicit-boundary outline/ridge on Windows portable 0.11.0. `baseline-output.jsonl` records that result.

After building the fix, run:

```text
python verify_cli.py PATH_TO_BIMCRAFT_CLI OUTPUT_DIRECTORY
```

The runner creates the roof, saves native and IFC output, renders a 3D PNG, reopens the native file and compares its full public roof record. It also attempts an impossible mass-roof update and checks the CLI error and unchanged saved file. The public core regression target additionally proves full in-memory update atomicity and unchanged undo/redo history, courtyard handling, clockwise outlines, sketch edge slopes and actual eave alignment picks, unchanged Edit Boundary, and successful mass updates preserving identity and pitch.

The PNG and IFC show a 10 x 20 m wall-line rectangle with 750 mm eaves, a 10 degree mono-pitch roof, and a base 6 m above Level 1. Expected eave outline is -750..10750 by -750..20750 mm. Expected ridge underside is 7895.52 mm. The native file stores the actual eave outline; no format migration is required.

Validation details are in `public-test-results.txt` and `cli-verification.json`; actual sanitized RED/GREEN, focused test and check logs are in `public-logs/`. Workspace-wide, desktop platform and full IFC coverage CI remain required; a focused local pass is not a claim that those CI jobs passed.
