# Straight offset-wall-end separator evidence

Synthetic case for [IPedrax/BIMCraft#341](https://github.com/IPedrax/BIMCraft/issues/341), based on upstream `30527e323e9c2e59abff927e4b76fe3ce0f83e15`, with local fix commit `13ee0085603f1ffa8c45168dd06f905f7b220400`. No customer model or site data is included.

Latest correction: commit `18b91317cd357c2030fdff1f2b541cdd055df5bc` restricts the new cap connection to straight walls and prevents a curved-chord false closure. See [round-1 proof and current checks](round-1/README.md). The original artifacts and checks below are explicitly from commit `13ee0085603f1ffa8c45168dd06f905f7b220400`; current checks are in `round-1/checks.json`.

`repro.txt` creates a 9.6 m by 6.4 m centreline enclosure and a partial partition drawn on its exterior finish face. A horizontal separator continues that finish face at the end cap. On official Windows CLI 0.11.0, the default centreline room measurement gives South the entire 61.44 m² enclosure and leaves North redundant. `before-centre.png`, `before.bimcraft`, `before.ifc`, and the release output record that result.

For straight walls, the fix adds a short derived connection across the wall end cap. It keeps the separator at `[4200,3200]`–`[9500,3200]` (5300 mm long), rather than tilting the separator towards the wall axis. The existing forward-extension rule for crossing separators is unchanged.

| Room | Before centre m² | After centre / gross m² | Finish / net m² |
|---|---:|---:|---:|
| South | 61.44 | 30.30 | 28.32 |
| North | 0, redundant | 31.14 | 29.14 |

Gross areas add to 61.44 m². South's gross area is `4.2*3.1 + 5.4*3.2 = 30.30 m²`; the cap connector creates the short step between those two runs. Net areas add to 57.46 m², the interior enclosure less the partition body. Both settings retain two independent rooms.

To repeat the CLI journey with a built executable:

```powershell
./run-journey.ps1 -Cli /path/to/bimcraft-cli.exe
python ./verify-ifc.py
```

Use IfcOpenShell **0.8.5** with pytest for the independent verification script. `journey.txt` is the original synthetic model-building script, using fresh-project ids. The runner writes native and IFC files, renders both room measurements, and opens native and IFC output in new CLI sessions. `verify-ifc.py` checks both quantities, EXPRESS validation, independently tessellated space volume/height, unchanged separator geometry, and both reopened room lists. `ci/validate_ifc.py after.ifc after-finish.ifc` from the source repository additionally builds every represented product.

Source checks use sequential Cargo commands with `CARGO_BUILD_JOBS=2`, `RAYON_NUM_THREADS=2`, `--test-threads=4`, and local-only overrides `--config 'profile.dev.package.bimcraft-core.opt-level=0' --config 'profile.dev.debug=0'`. The standalone CLI uses `--no-default-features`, which omits the optional DWG/RVT readers. The full default-feature workspace and platform matrix remain required CI follow-through; this package does not claim they ran locally.

The regression target is `cargo test -p bimcraft-core --test separator_wall_end -- --test-threads=4`. It checks reversed and rotated geometry, real gaps, nearby wall-side lines, non-room-bounding walls, native/IFC reopen and quantities, invalid input rollback, slab behavior, and actual rendered room fills. The original RED run recorded four failures and one negative-control pass. An intermediate run exposed an unrelated pre-existing behavior: `room_separator` accepts unknown JSON keys. The invalid-input case was corrected to use an out-of-range coordinate, retaining the intermediate failure and a separate official-release confirmation. No existing tests were weakened or disabled.

An additional focused geometry regression rejects internal arc-chord junctions as cap candidates; only actual wall ends carry cap metadata. Its separate RED/GREEN logs accompany the public Session suite. The broader 62-test room run preceded this guard; final focused separator/crossing/curved-wall results are recorded separately.

The native schema and stored separator are unchanged; no format migration is needed. Previously saved dynamic room boundaries are recalculated on reopen. Frozen room boundaries retain their existing semantics. Parallel end-cap joining applies only to straight walls and is limited to parallel outward continuations within the wall half-thickness and the existing 1 mm geometric tolerance. It does not bridge a real gap or attach along a wall side.

The public log copies replace local checkout/target paths with placeholders. Raw private logs and issue-search snapshots are excluded from `public-evidence-files.json`.


Final local results: public Session target 5/5; separator/crossing/arc-cap target 4/4; existing curved-wall test 1/1; documentation-map target 3/3; formatting, affected-package clippy and both i18n commands passed. The broader pre-cap-guard room run passed 62/62, including its tower case. Catalog message text and source references have no Git blob differences; only Windows newline rewrites were produced by extraction, yielding no catalog commit changes.

The standalone CLI build and creation/save/reopen/render journey passed. Both generated IFC files passed IfcOpenShell 0.8.5 EXPRESS validation and the repository's all-product shape check: 12 products, 8 represented products built, 0 problems per file. Both rooms have the expected independently tessellated volume and height. Shared footprint boundaries measure 9.7 m gross and 5.3 m net, with no sampled overlap. The original failing `before.bimcraft` also opens with both dynamic rooms correctly enclosed and no fields left out.

A first external verification attempt had a script error: the IFC import summary's `elements` is a count dictionary, while element-query replies contain arrays. The script now distinguishes the two valid output shapes. The initial error is retained as `independent-ifc-first-run.stderr`; the corrected script completed successfully. This was a verification harness error, not a product failure.

`checks.json` records commands, exit results, distinctions between pre-guard and final-source runs, toolchain, local build flags, binary hash, and required CI follow-through. No push or PR was performed by the implementer.



Known IFC reopen limit: the earlier output retains the tested areas and GUIDs, but imported IFC rooms use the existing 2500 mm default height instead of the native 2700 mm height. This change does not implement full height-reference round-trip preservation.
