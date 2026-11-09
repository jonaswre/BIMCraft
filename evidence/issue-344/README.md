# Sectional and roller gate evidence

This folder contains synthetic fixtures for IPedrax/BIMCraft issue #344. Every model and coordinate here was generated solely for this test; no customer or Autodesk files are included.

The command entry point is the existing CLI/MCP `type` or `family` command with `operation: "sectional"` or `"roller"`. Existing Family Editor Operation choices use the same operation list. Project-owned gate types generate closed equipment geometry; an authored family retains its own forms and symbols.

Run `bimcraft-cli run --script journey.txt` in this folder, then reopen `industrial-gates.bimcraft` or `industrial-gates.ifc`. The fixture includes both industrial gate operations plus an ordinary swinging door and a sliding window. `validate_ifc.py` requires IfcOpenShell 0.8.5 and checks IFC4/EXPRESS, GlobalId uniqueness, operation/type metadata, actual tessellated bounds and door quantities.

Sectional gates use IFC4 `USERDEFINED` and `UserDefinedOperationType: SECTIONAL`; roller gates use `ROLLINGUP`. Opening Width/Height/Area describe the clear opening (3000 mm by 3200 mm, 9.6 m2), while representation bounds also include guides, tracks or the coil box.

Native files remain format 18. The unchanged-base format-18 reader refuses the new enum variants before returning a model (`baseline18-verification.json`, with long-lived MCP state-preservation checks); it does not silently redraw them as swing doors. Existing files keep unspecified operations and their GUIDs. Future native formats remain refused.

The public log copies retain actual diagnostic output with local build paths replaced by neutral placeholders. RED logs intentionally contain the expected pre-fix failures. Full default-feature workspace/platform CI remains pending and is not established by focused local tests.


Local Cargo runs used `CARGO_BUILD_JOBS=2`, `RAYON_NUM_THREADS=2`, and `--test-threads=4`. Core build overrides were `--config profile.dev.package.bimcraft-core.opt-level=0 --config profile.dev.debug=0`; no profile change is committed. The standalone CLI was built with `--no-default-features`, excluding only the optional external DWG/RVT readers. Full default-feature workspace and platform CI remains required.

The format-18 baseline reader is identified by SHA-256 in `baseline18-verification.json`. Its core and CLI product source match delivery base `30527e323e9c2e59abff927e4b76fe3ce0f83e15`. The earlier portable 0.11.0 save inherited format 17; initial misleading format-18 fixture names are explicitly corrected in that verification record. The final compatibility probes use actual format-18 fixtures and the verified format-18 reader.

The permanent desktop test exercises Manage > Edit Family > Operation > Apply > native save/open > displayed operation > Cancel for both new operations. Six optional look/profile screenshots were attempted but not captured: enabling egui_kittest/wgpu exposes missing render methods in the existing upload-only test mocks; the bounded temporary workaround then required an unavailable image dependency. Both compiler logs are retained. All temporary source edits were restored; no dependency or unrelated harness change is committed.

`legacy-ifc-comparison.json` compares the identical existing swinging-door/Sliding Window fixture with the unchanged-base format-18 CLI and final CLI. Both show the same pre-existing IFC reimport normalization: DOUBLE_PANEL_VERTICAL becomes SINGLE_PANEL and the window material association is lost. In the industrial fixture, both gate Metal associations remain; this window normalization accounts for the root count changing from 49 to 48. It is outside this feature's implementation scope.
