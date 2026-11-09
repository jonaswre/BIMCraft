# Consolidated final-review fix wave — F1

F1 is corrected and verified within the requested rendering scope. No Critical finding existed. M1–M3 remain explicitly nonblocking recommendations. No push, PR edit, upstream merge or release was performed.

## Exact source history

| Tree | Reviewed old head | Final head |
|---|---|---|
| `upstream-11-property-lines`, `feat/property-boundaries` | `9274a263c1e483859585bfd60e707d2d910ca0e5` | `271643700503c0045d79ed71f4a9b6b9a4b3e9a8` |
| `upstream-combined-audit`, `audit/combined-factory-fixes` | `4056f2795e7b46bec20cfd56589e9b4058973a2d` | `626ad873a09d2217d350edbbcc11c1e3518a69e3` |

Production correction: `c117662cb80cb83206620db35d52af6ea036bfc8`. The task11 follow-ups are `c85c2f7827ff78eab6ef2e453df6c894d49e21a0` (generated catalog references only) and `271643700503c0045d79ed71f4a9b6b9a4b3e9a8` (test-helper lint only). Audit merged these exact commits with `--no-ff`, producing `a1528f3a2ab3f0e3654ba8d633ba71bbb2751767`, `c716a4948c73b15d4e218074932af7432a5cb3a8`, and `626ad873a09d2217d350edbbcc11c1e3518a69e3` respectively. No cherry-pick, reset, rebase or history rewrite occurred. Both original reviewed heads and the final exact task11 head are audit ancestors. Both source worktrees are clean; see `final-fix-artifacts/final-source-state.json`. All owned cargo/rustc/CLI processes exited before handoff.

## Root cause and bounded change

The property label independently centered its two text lines on the polygon centroid, while the room independently centered its three-line tag on its placement point. The combined synthetic plot and room both use `[9000,5000]`, so adding the property boundary overprinted the room name and area in a normal plan and its printed sheet.

`crates/core/src/render.rs` separates boundary strokes from property lettering. Plans collect the actual visible room text stroke bounds after annotation scaling, then draw property lettering after room tags are known. The centroid remains the default; only a conflicting property label moves upward, with a paper-scaled gap. Bounds are processed bottom to top in one pass, also avoiding an additional room encountered while moving upward. Hidden, other-level and cropped-out room tags do not reserve space. Creation order does not affect the result. The property outline, model coordinates, room anchor and room lettering are unchanged. This is property/room avoidance, not a general annotation engine.

`crates/core/tests/property_boundaries.rs` adds three public Session/render/sheet regressions, using real drawn stroke bounds and printed pixels. `docs/areas/property-boundaries.md` documents the behavior. Twenty translation catalog/template files receive only generated source-reference updates; stripping only `#:` lines makes every file identical to its prior content in each tree (`catalog-reference-only-*.json`). No translated message or template message changed.

## Red/green evidence and commands

All cargo commands used two build jobs and `RAYON_NUM_THREADS=2`; tests used `--test-threads=4`. Test/build commands supplied local, uncommitted `--config 'profile.dev.package.bimcraft-core.opt-level=0' --config 'profile.dev.debug=0'`. Task11 reused `.targets/issue-11`; audit used its exclusive `.targets/combined-audit`. Clippy's overrides followed the `clippy` subcommand. No build override entered a commit.

Before production edits, task11 ran:

- `cargo test -p bimcraft-core --test property_boundaries [local config] labels_remain_readable -- --test-threads=4`: **1 failed**, on real property/room ink overlap; `red-plan.log`.
- `cargo test -p bimcraft-core --test property_boundaries [local config] printed_sheet_preserves -- --test-threads=4`: **1 failed**, on a changed room-tag pixel in the printed sheet; `red-sheet.log`.

Passing affected verification:

| Execution | Result | Evidence |
|---|---|---|
| Task11 `cargo test -p bimcraft-core --test property_boundaries --test agent_docs [local config] -- --test-threads=4` | 12 property + 3 docs passed | `green-task11.log` |
| Audit `cargo test -p bimcraft-core --test property_boundaries --test combined_audit --test agent_docs [local config] -- --test-threads=4` | 12 property + 4 interaction + 3 docs passed | `green-combined.log` |
| Audit six named existing drawing/sheet lib tests | 6 passed | `green-drawing-sheet.log` |
| Final task11 and audit property targets after helper lint correction | 12 passed in each tree | `green-task11-final-helper.log`, `green-combined-final-helper.log` |
| `cargo fmt --check` | exit 0 | `fmt-final.log` |
| `cargo i18n check` | exit 0 in both trees | `i18n-check-task11.log`, `i18n-check.log` |
| Audit `cargo clippy -p bimcraft-core --all-targets [local config] -- -D warnings` | exit 0 | `clippy.log` |
| Audit `cargo build -p bimcraft-cli --no-default-features [local config]` | exit 0 | `cli-build.log` |

The six selected lib regressions cover existing room tag text, annotation scale, primitive ownership, crop clipping, reflected ceiling plans, and sheet placement/PDF printing. Their exact names are preserved in the output log. The three new regressions cover centered labels at three scales and differing name widths; full sheet creation → viewport placement → property creation → native save/open → unchanged original room pixels → PDF print at scales 100/200; and multiple room tags with property-first creation, visibility changes, distant tags and another level.

Intermediate failures are retained, not relabelled as passes: an initial sheet anchor assertion accidentally mixed old/new frame widths, one room-separator fixture used the wrong command shape, and moving a room to Level 2 needed a valid top constraint. Those test fixtures were corrected without weakening the overlap or room-pixel assertions. The first i18n check reported stale generated references; normal regeneration resolved it. The first Clippy run reported only a collapsible test-helper condition; collapsing it preserved all assertions. See the separately named failure logs.

The 144/37/34 unchanged suites, hostile sweep, tower/default-reader compilation, full workspace/platform suite and unrelated UI suites were not repeated, per the bounded brief. No test was disabled or weakened. The earlier affected tests ran on the production-fix merge; subsequent commits only refreshed catalog references and the equivalent test-helper condition, with final-head property runtime and all-target Clippy repeated after that helper change.

## Actual CLI and visible result

`final-fix-artifacts/synthetic-mixed.bimcraft` is a byte-identical copy of the existing `combined-artifacts/mixed.bimcraft`, SHA-256 `bb8f5ded0e8d4d974baa7f10ef1a36c424f930de8c69f429562697b805b21468`. The existing reviewed CLI captured separate `before-*` outputs. The CLI rebuilt from final audit head opened that exact native model and rendered the same Level 1 plan (1200×800), S-01 sheet PNG (2100×1485) and one-page sheet PDF into `after-*`. `verify_cli.py CLI before|after` records the bounded commands and asserts exact equality of every persisted native field after reopen/render/save. Both CLI binary hashes are recorded without shipping either executable or recording its local path.

I inspected `before-plan.png`, `after-plan.png`, `after-sheet.png` and a PyMuPDF raster of the actual corrected PDF (`after-sheet-pdf.png`). The corrected plan and printed PDF clearly show “SYNTHETIC BOUNDARY / 308.00 M²” above “SYNTHETIC ASSEMBLY / 1 / 179.97 M²”. Both property name and area remain useful; the room name, number and area remain in place. The only changed plan pixels lie in `[527,324,672,390]`, and sheet pixels in `[993,556,1158,631]`; all pixels outside fixed central label neighborhoods remain identical. The original failing sheet is unmodified; `before-sheet.png` is byte-identical to it. See `visible-output-check.json` and `original-evidence-integrity.json`.

Native geometry and identity invariance is stronger than a screen comparison: before and after CLI each prove equality of the entire native document, including room and boundary GUIDs, coordinates, room area inputs, types, library and georeference. Public tests also compare element output before/after rendering and native reopened elements. Existing invalid-input, read-only, native-version and IFC property-boundary tests pass. This render-only patch changes no native or IFC production code, so independent IFC geometry acceptance was not repeated.

## Evidence scope and triage

New evidence is confined to `final-fix-artifacts`, with an explicit `public-evidence-files.json` and hashes. All input names/coordinates are invented; no executable binary, private input or private path is allowlisted. Test/compiler paths in new logs are normalized. All 61 original checksummed combined evidence files match their recorded hashes, including preserved failures. Existing `combined-artifacts` and its original sheet were never written.

- M1 remains nonblocking: broad external verifier tolerance is not precision proof for arbitrary planar solids. Scope remains limited by the prior explicit-value and tighter combined checks; no harness expansion here.
- M2 remains nonblocking: existing real-TCP permanent coverage and external spawned-CLI evidence remain complementary. Permanent spawned-CLI regression is future work; the incomplete combined CLI test target remains explicitly incomplete.
- M3 remains nonblocking: permanent final IFC-reopen EPSG/map-conversion assertions remain future work. Existing six-case follow-up is bounded evidence, not acceptance of the whole coordinate parent.

Baseline IfcSpace height/volume loss remains **FAILED** and unmodified. Full-model IFC fidelity, parent coordinate scale fidelity, terrain IFC constraints, general annotation layout, desktop GPU/window behavior, final-head hosted CI and upstream integration readiness are not certified by this wave. Independent format/version reconciliation before landing combined feature branches remains required. The scoped F1 correction is ready for the requested independent re-review; publication and any approved PR352 update remain with the root agent.
