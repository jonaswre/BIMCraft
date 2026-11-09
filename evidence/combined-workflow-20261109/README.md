# Combined BIMCraft workflow audit

Final source: [`626ad873a09d2217d350edbbcc11c1e3518a69e3`](https://github.com/jonaswre/BIMCraft/tree/626ad873a09d2217d350edbbcc11c1e3518a69e3), branch `audit/combined-factory-fixes`. Base: `30527e323e9c2e59abff927e4b76fe3ce0f83e15`. All 22 exact final main-target contribution heads are ancestors; see [final-included-heads.json](final-included-heads.json). [final-combined-source.patch](final-combined-source.patch) reproduces the final source tree from the base.

This is a reproducible fork audit, not a combined replacement PR, upstream merge or release. The external coordinate/site implementations and their three dependent acceptance PRs are excluded. The individual upstream PRs still need rebasing and the documented native-version/behavior reconciliation before they can land together.

## Verification and review

The original combined audit at `4056f2795e7b46bec20cfd56589e9b4058973a2d` passed 144 dedicated integration tests, 37 focused core tests and 34 desktop tests. A final overlapping subset passed 17 tests. Actual CLI/native/IFC journeys, both original independent format19 fixtures and both actual older-reader refusals passed. IfcOpenShell0.8.5 checked seven IFCs for schema/EXPRESS, unique GlobalIds and actual shape construction. The merged coverage IFC separately contains 365 products and 361 represented shapes, with zero validation problems. Formatting, i18n and affected-package Clippy passed. Full combined workspace/default-reader/platform CI was not run.

The [initial whole-delivery review](initial-whole-delivery-review.md) found one Important interaction: the new property label overlapped an existing room label. The [fix report](property-label-fix-report.md) and [scoped independent re-review](final-fix-review.md) close that finding without new Critical/Important breakage. Property text moves only when it conflicts with visible room text. Model geometry, room lettering and the entire native document remain unchanged. The corrected source is also published in existing [PR352](https://github.com/IPedrax/BIMCraft/pull/352), head `271643700503c0045d79ed71f4a9b6b9a4b3e9a8`.

After that bounded rendering change, 12 property tests, four combined interactions, three docs tests and six existing drawing/sheet tests passed. All 12 property tests passed again in each final source tree after the test-helper lint correction. Formatting, i18n and core all-target Clippy passed; the actual CLI was rebuilt and exercised. The earlier 144/37/34 suites were not repeated or attributed to the amended head. Both catalog refreshes change only source references in all 20 catalogs.

## Before and after evidence

- [Original failing sheet](combined-artifacts/sheet.png) and [strict failed geometry probe](combined-artifacts/public-logs/strict-ifc-roundtrip.log) remain unchanged.
- [Corrected plan](property-label-fix/after-plan.png), [sheet](property-label-fix/after-sheet.png) and [actual PDF](property-label-fix/after-sheet.pdf) show both readable labels. [PDF raster](property-label-fix/after-sheet-pdf.png) is from that real output.
- [Plan RED](property-label-fix/red-plan.log) and [sheet RED](property-label-fix/red-sheet.log) reproduce the defect before the correction; final green logs and intermediate fixture/lint failures remain alongside them.
- [Native invariance](property-label-fix/after-invariance.json), [pixel comparison](property-label-fix/visible-output-check.json) and [original evidence integrity](property-label-fix/original-evidence-integrity.json) distinguish a text-only fix from moving/hiding model data.
- Root replayed the verifier from this exact public folder layout in a separate copy: [replay log](public-package-replay.log). Plan and sheet PNGs match the corrected recorded outputs byte for byte.

`combined-artifacts/` preserves the complete explicitly curated pre-correction evidence, including the original layout reproduction, checksums and portable independent IFC validators. `property-label-fix/` preserves the separately checksummed correction evidence. No customer models, real site data, credentials or executables are included. These are actual software-rendered outputs, not GPU/window screenshots.

## Reproduction

Build `bimcraft-cli` from the exact final source with `cargo build -p bimcraft-cli --no-default-features`. In a copy of this evidence package, run `python property-label-fix/verify_cli.py ABSOLUTE_CLI_PATH after`. The script opens the supplied synthetic native model, renders the plan/sheet/PDF, saves, and checks every persisted field. Use an absolute path to the built executable.

With IfcOpenShell0.8.5 and NumPy installed, `python combined-artifacts/verify_ifc.py` repeats the scoped IFC validation. `python combined-artifacts/verify_ifc_full_model.py` remains a strict **FAILED** check and exits1 on the room-height discrepancy below; it is not an xfail or a passing expected-failure result. Original fixture/reader provenance and instructions are in [the original README](combined-artifacts/README.md).

## Explicit limits

- Whole-model IFC geometry equivalence is **not established**. An IfcSpace retains identity but changes height2700mm to2500mm and volume485.9055912863348m3 to449.91258452438404m3 after IFC import. A pristine base18 minimal room reproduces the same loss. The strict failure and baseline evidence are retained. The passing feature validator excludes IfcSpace geometry equivalence, while checking its schema, identity uniqueness and shape construction.
- The combined CLI unit-test target is **INCOMPLETE**: stopped during unconditional unrelated DWG/Revit dev-dependency compilation. Actual CLI build/runtime proof is separate. No source test was disabled or weakened.
- Steel and stairs independently advance18 to19 in their main PRs. This audit reconciles steel19 then stairs20, preserves both original19 shapes and is refused by both older19 readers. The two independent PR histories still need coordination upstream.
- Parent coordinate/site branches were not combined. PR284's native horizontal-only versus IFC4 isotropic Z-scale discrepancy remains reported. Terrain IFC vertex retriangulation does not establish constraint/boundary persistence.
- Nearby baseline level labels can still overlap. The correction is property/room text avoidance, not a general annotation engine. Original single-sheet overflow is retained under the documented placement policy; S-02 separately demonstrates printable 3D output.
- Three nonblocking recommendations remain: tighten the planar public verifier tolerance, keep the spawned-CLI context probe as a permanent regression, and assert final reopened CRS/map conversion permanently. Existing executed probes and stricter independent checks are described in the reviews.
- No final-head combined hosted CI, GPU/platform/default-reader certification, engineering design certification or upstream merge/release approval is claimed. Individual PR CI is tracked in the delivery inventory.

Latest-target snapshot: upstream main advanced to `6227d0144881e077cec3274de1144968fd053502` during delivery. The [separate merge audit](../upstream-delivery-20261109/current-main-merge-audit.json) finds18of22main contributions need conflict resolution there. This audit proves the pinned30527-based composition; it does not incorporate the93changed upstream files. PR352 current-head hosted CI is blocked by those merge conflicts.
