# Open stair construction: synthetic evidence

Tracking: [issue 189](https://github.com/IPedrax/BIMCraft/issues/189#issuecomment-6740344165), continued in [discussion 220](https://github.com/IPedrax/BIMCraft/discussions/220#discussioncomment-19491738). This scoped contribution covers open construction; the wider component-stair proposal and multistorey issue 297 are separate.

`stair {"open":true}` and `modify {"open":true}` create thin tread plates, two side stringers per flight, and thin landing slabs. Open construction survives native and IFC save/reopen. Existing stairs and desktop placement remain solid by default. The native schema changes from 18 to 19, because the old reader drops the field and reinterprets geometry. This independent version-19 change must be reconciled when another schema-changing branch lands; it does not include unrelated steel-profile changes.

## Reproduce

Use a CLI built from this branch and a separate CLI built from upstream commit `30527e323e9c2e59abff927e4b76fe3ce0f83e15`. No executable is included here.

```text
cargo build -p bimcraft-cli --no-default-features
python check_cli_open.py PATH_TO_CLI
python verify_cli.py PATH_TO_CLI PATH_TO_FORMAT18_CLI
python verify_ifc.py PATH_TO_CLI
python verify_identity.py
```

`verify_cli.py` runs in this directory, reads the synthetic `baseline-solid.bimcraft`/IFC controls, creates the example from `open-stair.txt`, reopens both formats, tests the older-reader refusal and compares the closed stair's complete IFC DATA section (the generated timestamp is excluded). `verify_ifc.py` requires IfcOpenShell 0.8.5, NumPy and pytest. It validates schema and EXPRESS rules, builds every shape, checks GlobalId uniqueness, compares independently calculated volumes and native glTF mesh bounds/clearances, and repeats the journey after tessellating flights externally.

The sample is wholly synthetic: a raised level and a rotated, two-flight stair with one turning landing and two railings. `open-stair.png` is the core software 3D render; `open-construction-detail.png` is the reopened IFC rendered with railings hidden. These are not desktop or GPU frame captures. The automated mesh assertions inspect the mesh data supplied to the desktop renderer. Existing desktop placement, landing, railing and width-edit/undo workflows are exercised by egui_kittest.

## Expected geometry

Treads are 40 mm thick. Each flight has two side stringers, 15 mm wide and 250 mm deep vertically, sloping toward the next tread. Landings are 250 mm slabs. For this fixture each flight has seven tread solids and two stringer solids; its summed solid volume is 0.0931 m³, and the landing is 0.25 m³. IFC writes the supports inside the existing flight products and adds `BIMCraft_Stair.Open` (`IfcBoolean`) on the stair. The shape counts and identities of closed construction stay unchanged. Geometry dimensions are fixed in this first addition; no stair-type editor is added.

## Verification

The logs include the initial failing public Session and real CLI tests, passing focused regressions, the actual native-version gate, desktop control tests, independent IFC checks, and applicable build/lint/catalog checks. Local Cargo runs use two build jobs, four test threads, `RAYON_NUM_THREADS=2`, and temporary `profile.dev.package.bimcraft-core.opt-level=0` / `profile.dev.debug=0` overrides. These flags are not committed.

The full workspace/platform/default-feature CI suite remains a publication requirement; it is not claimed by the focused local runs. The local IFC CI fixtures (sample house and coverage script) were independently validated: 46 and 353 shapes respectively, zero problems.

Native reopen preserves all IFC role IDs. IFC import preserves native stair and railing GlobalIds; generated flight, landing and metadata IDs follow the existing model-uid derivation and regenerate after IFC import. The same flight/landing behavior was verified with the unchanged format-18 reader and closed fixture; this contribution does not change that derivation. See `identity-validation.json`.

Reviewed local source: `3293f093a9b827dcacb8c78d0cca4682cb449cbb` (base `30527e323e9c2e59abff927e4b76fe3ce0f83e15`); Windows 11 Enterprise.
