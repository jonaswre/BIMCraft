# Distant scan automatic framing: issue #324

Reproduced on BIMCraft 0.11.0, base `30527e323e9c2e59abff927e4b76fe3ce0f83e15`, Windows 11. Run `repro.txt` with the adjacent synthetic.xyz. Four synthetic walls and a section are rendered before and after importing a distant four-point scan. `reproduction.png` shows the walls shrinking in section, parallel and perspective views. The original public Session regression fails for all three views (`red.log`, `red-controls.log`).

Fix head `1bc2c67e855bb6d77b97c65e48fc19fa4122a0de` fits built geometry first, falling back to scans when no built geometry is visible. `green-output.png` shows the readable building and separate explicitly aimed scan views. All three automatic building PNGs are byte-identical before import, after import and after native save/open (`green-metrics.json`). The four public Session tests also cover scan-only and hidden-building fallback, empty scenes, explicit cameras/crops/level extents/section box and failed scan import rollback. These are CPU-rendered command outputs, not desktop/GPU screenshots.

Completed focused runs: 4 new journey tests, 21 cloud, 62 rendering, 7 spatial-cache, 10 component-preparation and 4 primitive-fragment tests passed (108 total; rendering has four unchanged opt-in ignored probes). Formatting, core all-target Clippy and translations passed. Catalogs remained identical after generation.

Additional view3d run is **INCOMPLETE**: 72 passed, one existing ignored benchmark and two unfinished unchanged randomized update stress cases, no observed failures. Direct scan/camera/perspective/picking/section/tower cases completed. The coordinator stopped only that owned broad process after 9m43s; its abnormal exit and full raw log are retained in view3d-tests.log, view3d-partial-summary.txt and view3d-interruption.json. No source or CI test was disabled, weakened or marked acceptable. This is not a suite pass. Full workspace/platform/IFC CI remains required on the PR head.

Local Cargo uses uncommitted opt-level=0/debug=0 overrides, two build jobs and four test threads. No format/migration, GUID, georeference or command-permission contract changes. All evidence is synthetic.
