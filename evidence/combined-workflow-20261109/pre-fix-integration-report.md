# Combined integration audit

Status: local audit implementation and bounded verification finished; independent root review pending. The strict full-model IFC probe remains failed and the label-overlap finding requires final triage. No push, upstream merge, release or combined replacement PR.

Base: `30527e323e9c2e59abff927e4b76fe3ce0f83e15`.
Branch/worktree: `audit/combined-factory-fixes` / sibling `upstream-combined-audit`.
Final head: `4056f2795e7b46bec20cfd56589e9b4058973a2d`.
All original worktrees and branches are preserved. The audit contains the exact source commits through local merge commits, followed by the explicitly described combined-only reconciliation. Intermediate mechanical merge commits are not independently claimed buildable.

## Included exact source heads

| Task | Exact head |
|---|---|
| 19 | `8421cbbb4fb3944ff4e844403300f0f5db815a6c` |
| 1 | `48407bc78171b4e2d95cbbf0f2005fb87cb6654a` |
| 7 | `1d4279ebce23bd15a249038e82da9afd4346acc0` |
| 21 | `79a897209390c6a6340f3ea8732d8b31fdb29472` |
| 2 | `7d5424d2d8e7cd6ac732097f8046d988023fa33f` |
| 16 | `85888aaf764df26777ceebf00218145efb8fbb58` |
| 14 | `c8683ecbebf28c90230f9ac8189de3bc355ef6ff` |
| 13 | `1bc2c67e855bb6d77b97c65e48fc19fa4122a0de` |
| 3 | `b9343587cd224e9a131d6953feef9a616a8730e4` |
| 4 | `61ab42bd42e2d30df91760e1879923e6b872629b` |
| 15 | `549d322b42b33feb212c08307b86a4a8fdf6d93b` |
| 18 | `71398cf01f2a444bc53412af58bcf124ce964d29` |
| 20 | `f32146429f4912ee84c7ec12327f34f861049523` |
| 22 | `8681249e5d60f5836ab4e52d3a3bfc07f6733e0d` |
| 24 | `18b91317cd357c2030fdff1f2b541cdd055df5bc` |
| 10 | `64bf0ec5d0c0afd711e70002dc3b68a214fb0714` |
| 9 | `79199fbb02e4187de0aefb9c21be4636ee4a94e2` |
| 11 | `9274a263c1e483859585bfd60e707d2d910ca0e5` |
| 12 | `be6881c9d72fa7973bc4d328867448bf60cc9b54` |
| 23 | `3219a10bfce22a70199a7c6caef5dd35e1882f1b` |
| 26 | `3293f093a9b827dcacb8c78d0cca4682cb449cbb` |
| 5 | `ca3d5e5bf3cbde1f3b0cff2d16173de967bf5d2f` |

External wholesale parent features PR284/289 and their acceptance branches are excluded. The task5 parent acceptance head b2ec5a4, task6 7ebeb79fa917a840d3b90b9548977909bd3b77c5 and task25 3317917 remain separate review records; their acceptance evidence is not combined-main evidence.

## Explicit merge and behavior resolutions

| Intersection | Resolution and resulting semantics |
|---|---|
| IFC exporter tasks7/16/11/26 | Union GUID-role constants: grid59/60, wall intent61/62, property63..66, stair67/68. Existing steel51/52/57/58 unchanged; no new role. All publishers/readers retained. |
| IFC exporter tasks7/12 | Keep complete grid exporter and independent prism-volume helper; task12 quantities use task9 profile-aware member quantities where applicable. |
| Opening tasks21/18/22, commands.rs | Preserve21 ordered cut-eligible host lookup; if none, apply18 unique raised fallback only on default level. Explicit-level lookup remains exclusive and cut-first. Extract shared raised lookup without changing ambiguity guard. Apply22 requested-level elevation after host choice. Root approved this composition. |
| Opening test21 versus22 | Original explicit upper-wall/Foundation-level success assumed level ignored. Preserve it as below-host refusal+rollback. Add a tall Foundation host spanning Level2 and assert explicit host identity and absolute3000mm sill. This intentionally updates the older expectation to task22 behavior; no assertion silently removed. |
| Sheet tasks3/15 | Preserve task3 title-aware free placement and task15 default3D scale fitting; combine command documentation, including returned viewport ID/position/scale. |
| Room tasks19/24, model.rs | Keep task24 WallSegment/end-cap eligibility and task19 filtered centre-line geometry. Curved walls still lack straight end-cap eligibility. |
| Native tasks9/26 | Combined FORMAT20: steel19 then open stairs20. Existing pre19 migration keeps solid legacy construction;19->20 preserves an already present open marker and defaults absent fields only. Original published19 histories remain unchanged. |
| Legacy georeference task5 | Clean merge. Field-marker conversion is after future-format refusal and before lenient unknown-field scanning; no extra format bump. |
| CI coverage script | Append all grid, steel, property and open-stair contributions; preserve original ID-sensitive commands. |
| UI module registrations | Union sheet_fit, industrial_doors, open_stairs; existing structure/control tests retained. |
| CLAUDE module map | Preserve doors, profiles and site module references plus all original links. |
| Generated catalogs | During merges retain current conflicted catalog; audit all input semantic changes, then regenerate once from combined source. Only tasks9/10/11 add untranslated messages. Complete normalized catalog entries from base and all semantic contributors are retained in all20 catalogs; i18n check passes. |

Detailed git merge outputs are `combined-artifacts/merge-*.log`; catalog audits are `catalog-input-audit.json` and `catalog-preservation.json`.

## Integration-only regressions and source changes

`crates/core/tests/combined_audit.rs` adds four public-interface user journeys: format20 and preservation of both19 shapes/reference maps; implicit cut-host priority/default raised success/ambiguity; property boundary Level edit across warmed old/new plans, save/reopen, undo and reopen; sectional/roller section and elevation rendering, visible/pickable section pixels, native reopen, and visible removal.

Initial red run: 0/2 passed. Failures specifically prove the still19 format and missing implicit raised fallback. Green after reconciliation:4/4. A test-helper compile typo (`Image.ids` instead of public pixel picking) was corrected before green; source behavior was not changed for it. The later dedicated run exposed the21/22 test contract described above, then all21 dedicated targets passed144 tests. No tests were disabled, skipped or weakened to obtain a result.

## Verification and exact resource bounds

Every Cargo command uses exclusive `.targets/combined-audit`, `CARGO_BUILD_JOBS=2`, `RAYON_NUM_THREADS=2`, testthreads4, sequential Cargo processes. Local overrides `--config profile.dev.package.bimcraft-core.opt-level=0 --config profile.dev.debug=0` precede test/build and follow clippy. No overrides enter committed configuration.

| Check | Result | Evidence |
|---|---|---|
| Dedicated core integration targets,21 targets |144 passed,0 failed|core-integrations-final.log|
| Core library raised room/plan duplication/profiles/bridge filters |37 passed,0 failed|core-lib-focused.log|
| Focused desktop UI |34 passed,0 failed|ui-focused.log|
| CLI no-default-features test target |INCOMPLETE: dev-dependency reader code generation stopped within resource scope; no test result claimed|cli-tests.log,cli-tests-stopped-processes.json|
| Actual CLI build |passed|cli-build.log|
| Actual mixed CLI/native/IFC workflow |passed for represented feature identities and native complete fields|cli-runtime-final.log|
| Independent IFC0.8.5 schema/EXPRESS/shape/GUID and scoped geometry |passed,7 files|ifc-feature-runtime.log,ifc-validation.json|
| Strict same-identity full-model IFC geometry roundtrip |FAILED: existing IfcSpace height/volume loss|strict-ifc-roundtrip.log,strict-ifc-roundtrip-failures.json|
| Baseline18 room-loss reproduction |same2.7m->2.5m defect independently reproduced|baseline-room-cli.log,baseline-room-shapes.log|
| fmt / i18n / docs / affected-package Clippy |passed; Clippy core/UI/RVT/CLI all-targets no-default-features, warnings denied|fmt.log,i18n-check.log,clippy-final.log|
| Combined CI coverage script and independent IFC validation |365 products,361 shapes,0 problems|ci-coverage-cli.log,ci-coverage-ifc.log|

Dedicated command:
```text
cargo <overrides> test -p bimcraft-core --test combined_audit --test agent_docs --test drawing_lettering --test ifc_grids --test ifc_supplementary_quantities --test ifc_wall_authoring --test industrial_doors --test legacy_georeference --test open_stairs --test opening_elevation --test opening_level --test property_boundaries --test raised_wall_openings --test roof_overhang --test scan_framing --test script_references --test separator_wall_end --test sheet_fit --test sheet_title_fitting --test steel_profiles --test viewport_placement -- --test-threads=4
cargo <overrides> test -p bimcraft-core --lib -- raised_wall_room_tests:: plan_view_tests:: profile_tests:: bridge::tests:: --test-threads=4
cargo <overrides> test -p bimcraft-ui-egui --lib -- tests::structure:: tests::industrial_doors:: tests::open_stairs:: tests::sheet_fit:: tests::control:: --test-threads=4
cargo <overrides> test -p bimcraft-cli --no-default-features -- --test-threads=4  # stopped during dev-dependency codegen, INCOMPLETE
cargo clippy <overrides> -p bimcraft-core -p bimcraft-ui-egui -p bimcraft-rvt -p bimcraft-cli --no-default-features --all-targets -- -D warnings
```

## Actual output and independence

`mixed.txt` authors invented steel/profile, two industrial gates, boundary, open stair, grids, raised enclosing walls, upper window, room, projected georeference, section/elevation and sheet. CLI save/reopen retains all native element/type/library/georeference data. IFC reopens with no skipped/generic elements; authored wall/opening/steel/stair/boundary GUIDs retain identity. Native auxiliary identities remain stable. IFC-generated stair/metadata roles may regenerate as already documented by individual contributions; no claim is made that every auxiliary IFC ID survives import.

IfcOpenShell0.8.5 independently validates all7 files, every represented shape, and unique IfcRoot GlobalIds. Mixed original/native-reopened/IFC-reopened each contain106 roots,26 products,21 represented shapes. Both original19 fixtures are byte-preserved copies, not relabelled combined fixtures. Original old readers export them and combined reader exports them with exactly matching represented product GUIDs, bounds and mesh volumes (1e-9 relative/1e-8 absolute). Both actual19 executables reject combined20 before lenient decode. Reader hashes are in CLI log; binaries are excluded from publication.

CRS EPSG:32632, invented map origin500000m/5500000m, orthogonal height42m, rotation23degrees and scale0.9996 survive IFC import. IFC MapConversion.Scale is0.0009996 because project lengths are millimetres and projected CRS units metres. The first verifier incorrectly expected0.9996 in IFC; correcting that unit expectation is recorded in ifc-runtime-final.log. A separate earlier verifier lifetime error used a geometry object after dropping its IfcOpenShell owner; fixed by retaining the owner. Neither required production changes.

Viewed mixed.png, section.png, elevation.png, sheet.png and sheet-3d.png: gate panels and upper window visible, frame and open stair visible in section, and raised wall extent represented. Synthetic default Level2(3000) and Upper(3400) labels overlap in the section/elevation; this pre-existing annotation-layout limitation is not fixed here. These are core software-rendered outputs, not GPU captures or a desktop screenshot.

## Explicit failure boundary and final-review triage

The strict full-model IFC roundtrip probe remains FAILED, exit1. The room retains its GUID/plan, but IfcSpace height2700mm becomes2500mm and mesh volume485.9055912863348m3 becomes449.91258452438404m3. A pristine baseline18 executable reproduces height2700->2500 with only a rectangular room. `verify_ifc_full_model.py` is a strict comparison of independent geometry results; it is neither an xfail nor an expected-pass test. The passing feature validator excludes IfcSpace from IFC geometry equivalence while still validating its schema/EXPRESS, identity uniqueness and actual shape construction. Full mixed-model IFC fidelity is not established. Root ruled this out of scope for the22 contributions, with final review to assess whether that limitation is load-bearing.

- Task10 section/elevation runtime gap: addressed by combined persistent regression and viewed mixed output.
- Task11 Level reassignment warm-plan/undo/reopen gap: addressed by combined persistent regression.
- Task12 blanket0.2percent independent mesh-volume verifier tolerance: retained for root triage. No production or original verifier change; combined19 comparison uses much tighter tolerance and targeted combined solids use2e-5 relative/1e-5 absolute.
- Task23 permanent actual externalCLI MCP regression recommendation: remains separate. Included focused control/bridge/CLI tests run here; no large reader-runtime build.
- Task25 finalIFC-reopen CRS/mapconversion assertion recommendation: external-parent-only and excluded from this integration. Root reports six actualparentCLI cases passed the frozenPROJoracle in task-25-artifacts/root-crs-reopen-followup.json. This is not combined-main proof.

The final focused recheck of combined_audit, agent_docs and opening_level passed17 tests. This rerun followed test-module wrapping and report/documentation edits; the144/37/34 suites were not repeatedly rerun. The last source edit after actual CLI runtime proof was documentation-only (`///` on an existing blank line), with no runtime logic or source line-number change; final Clippy checked the final source. The first Clippy pass exposed an empty line in the merged native-history doc comment; replacing the blank line with `///` keeps source-reference line numbers stable and changes no runtime behavior. Final Clippy result is reported above.

The standalone CLI test target unconditionally activates DWG/Revit dev dependencies despite `--no-default-features`. After over3minutes of unrelated reader code generation, only the audit cargo process and verified target child compilers were stopped, per root resource ruling. Actual CLI build/runtime and focused real UI/bridge tests are separate passed evidence; no combined CLI-test pass is claimed.

New visual interaction for final review: the property-boundary name/area (`crates/core/src/render.rs:1270-1274`, polygon centroid) overlaps the room label (`render.rs:2975-2979`, Room.point) when both are[9000,5000]. This is potentially a new feature interaction, not classified as baseline/out-of-scope. No label or model position was changed to conceal it; sheet.png still shows it. Final review must classify its severity and may require a consolidated fix wave. Readability is not claimed passing.

The first mixed layout placed both plan and3D onS-01: actual command results in cli-runtime-2.log record viewports at[215,168.5] and[542.5,168.5]. Task3's documented overflow policy put the second view off paper. Final mixed.txt gives3D its ownS-02, visibly inspected. original-layout-reproduction.txt/.bimcraft/.png/.log faithfully re-render the original commands/layout and retain the label overlap and off-paper policy result. The initial PNG itself was overwritten by the later portable-fixture rerun; this is explicitly reproduction evidence, not an original-byte screenshot claim.

No full combined workspace/default-reader-feature or cross-platform CI claim: intentionally bounded dedicated targets/affected packages, no hostile/tower stress sweep, no combined remote CI. Individual branch CI results are separate evidence.

## Handoff

Public synthetic-only files and sanitized logs are explicitly allowlisted in `combined-artifacts/public-evidence-files.json`; README draft is `combined-artifacts/README.md`. No customer/private models, real-site coordinates, tokens or executables are included. Final head `4056f2795e7b46bec20cfd56589e9b4058973a2d` has a clean worktree. All owned Cargo/compiler/CLI/Python verification processes have exited. Root-review package is ready; review must triage the explicit room-loss and property-label findings and the incomplete CLI-test target. No source, test or artifact work remains running.
