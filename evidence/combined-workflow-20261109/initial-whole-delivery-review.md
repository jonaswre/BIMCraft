# Independent final whole-delivery review

Reviewed base `30527e323e9c2e59abff927e4b76fe3ce0f83e15` to audit head `4056f2795e7b46bec20cfd56589e9b4058973a2d`, against `final-review-brief.md` and the code-reviewer template. This is the final interaction review after individual task reviews, not a replacement review of every task. Source paths below refer to `upstream-combined-audit`; artifact paths refer to this ledger. Source/index/refs were read-only. Only this report was written. No suites were repeated.

## Strengths

- The opening composition is coherent: omitted level searches ordered cut hosts before trying a unique raised host on the default level; explicit level remains exclusive; explicit wall selects identity while an explicitly different level sets elevation (`commands.rs:7413`, `:7430`). The below-host refusal is retained alongside the new valid tall-host success. The reconciliation is a real contract change with explicit coverage, not a hidden exception or removed assertion.
- The combined native format advances to 20, preserving the independent steel19 and stair19 shapes while preventing either actual older reader from silently reinterpreting combined files (`model.rs:119`, `:3537`). The two original fixtures, actual reader refusals and independent geometry comparisons provide stronger evidence than relabelling a current fixture alone. Legacy georeference normalization occurs after future-version refusal and before lenient unknown-field removal.
- IFC roles 59 through 68 are disjoint and preserve earlier reservations. Profile-aware quantities, wall-authoring metadata, grid exchange, property curves and open-stair construction remain present together. Borrowed recognized metadata avoids reintroducing the previously reviewed fan-out problems. Physical wall joins remain separate from room-boundary filtering.
- Sheet placement includes title width/height and keeps overflow backgrounds beyond paper; default 3D scale respects an assigned view/template scale (`sheet.rs:85`, `:130`, `:146`). Profile geometry participates in bounds keys, hollow plan filling and section geometry, while profiled tubes bypass the old solid-tube instancing path. Scan framing and built geometry use the same fitting-point policy.
- Script references still enter normal command validation and read-only/file-permission handling. Bridge preflight uses the same named-line parser. Desktop context is authenticated and reads live UI-thread state without modifying session history or selection.
- Captured verification includes actual CLI/native/IFC journeys, independent IfcOpenShell shape construction, negative migration probes and meaningful UI/public-command tests. The failed full-model probe and the original layout reproduction are preserved, rather than being silently relabelled as passing. Catalog preservation has a separate normalized audit covering all 20 catalogs; generated source-reference churn is not mistaken for translation loss.

## Issues

### Critical

None found within the reviewed contributions.

### Important

**F1 — The new property auto-label makes an ordinary mixed plan and printed sheet unreadable.**

- Location: `crates/core/src/render.rs:1270-1274`, interacting with the room tag at `:2975-2979`; property data at `crates/core/src/model.rs:224` and creation at `crates/core/src/commands.rs:6845` have no separate label position.
- Evidence: I viewed `combined-artifacts/sheet.png` and inspected `mixed.txt`. The rectangular plot centroid and the room point are both `[9000,5000]`. The new property name and area are drawn directly over the existing room name and area. The image visibly confuses two different areas. This is a natural single-building/plot arrangement, not merely an exotic malformed input.
- Impact: adding the new boundary degrades an existing room drawing and its printable output. The current automatic label has no independent placement control. Moving the room point can work around some instances, but does not establish a correct default combined journey. Calling this an annotation limitation, or moving the synthetic fixture to hide it, does not close the defect.
- Required correction: provide a bounded way for these labels to coexist, such as deterministic avoidance of room-tag bounds or a usable independent property-label placement mechanism. No global annotation engine is required. Add a public render/sheet regression with the coincident-center case, demonstrate it fails before the fix, and inspect the corrected plan and sheet while preserving the original failed evidence. Verify the boundary geometry, room area and identity are unchanged.

### Minor

**M1 — The public quantity verifier gives planar walls/footings an unnecessarily loose mesh tolerance.**

`task-12-artifacts/validate_ifc.py:34` applies `rel_tol=0.002` to all walls and footings, although its explanation concerns circular tessellation. Scope that tolerance to the circular case and retain a tighter planar comparison when maintaining the harness. This does not block the present contribution: separate explicit-value Rust regressions, hand-calculated measurements, and the much tighter combined geometry comparisons cover the current planar results. Artifact scope clarification is sufficient for current bounded claims; the broad tolerance should not be advertised as precision proof for arbitrary planar solids.

**M2 — Preserve the spawned-CLI desktop-context journey as a permanent regression when practical.**

`crates/ui-egui/src/tests/control.rs:381` permanently tests the real TCP/live-UI journey; the actual CLI MCP-to-TCP-to-egui proof remains in `task-23-artifacts/cli-probe.log` and its reproducible external probe. The production MCP dispatch is small and the executed external proof addresses the current integration risk. This is a future coverage improvement, not a blocker. The incomplete combined CLI test target must remain explicitly incomplete.

**M3 — Assert final IFC-reopen CRS and map conversion permanently.**

`task-25-artifacts/accept_georef.py:101-104` checks reopened latitude/longitude/true north/elevation, but omits final CRS EPSG and map conversion. `task-25-artifacts/root-crs-reopen-followup.json` records six actual cases with matching EPSG, including Lambert 2154, and mapping errors consistent with the frozen PROJ oracle. No current failure is demonstrated. Keep the permanent-assertion recommendation, without treating this narrow parent acceptance as approval of the complete parent feature.

## Deferred-item and boundary triage

| Item | Disposition |
|---|---|
| Task12 mesh tolerance | Nonblocking M1, for the reasons above. |
| Task10 dedicated section/elevation journey | Closed. `crates/core/tests/combined_audit.rs:81` exercises both operations, actual section pixels/picking, native reopen and equality of both rendered outputs, followed by a visible elevation change on deletion. Captured integration log passes it. I also viewed `section.png` and `elevation.png`; the mixed section is an additional illustration, not a substitute for the dedicated gate-crossing test. |
| Task11 warm old/new plan, Level edit, undo, reopen | Closed for that lifecycle concern. `combined_audit.rs:51` warms both plans, changes level, saves/opens a fresh Session, undoes, saves/opens again, compares the old rendered image and checks identity. Captured log passes it. This does not close the separate F1 readability issue. |
| Task23 permanent spawned CLI | Nonblocking M2; current external runtime proof and committed real-TCP coverage are complementary. |
| Task25 permanent final CRS checks | Nonblocking M3; all six follow-up cases match, including the distinct Lambert CRS. External-parent evidence only. |
| Strict full-model IFC probe | Remains **FAILED**. The same IfcSpace GUID changes max Z 2.7m to 2.5m and volume 485.9055912863348 to 449.91258452438404m3. `baseline-room-shapes.log` independently reproduces 2.7m to 2.5m on original baseline18. This is real data fidelity loss, but not introduced by these contributions. The report and artifact README explicitly distinguish it from the passing feature validator, which excludes IfcSpace geometry equivalence. Those bounded claims are adequate; do not claim full-model IFC geometry preservation. |
| Independent format19 histories | Combined20 reconciliation is sound, but the two published independent feature PRs still need coordinated rebase/version reconciliation before both land. Local audit success is not evidence they merge unchanged. |
| Coordinate parent PR284 scale | The 45m to 44.9988m native/IFC discrepancy at scale0.9996 remains unresolved and reported. It blocks any broad parent fidelity claim, not this narrow migration contribution. |
| Terrain parent | Native breakline persistence has the noncocircular control; IFC retriangulation/constraint and boundary loss remain outside the accepted native persistence scope. No combined terrain/coordinate compatibility conclusion is warranted. |
| Original single-sheet overflow | Documented task3 policy: when full, put the next viewport beyond paper, without erasing the printed border. The original command transcript and explicitly labelled reproduction remain available. S-02 makes the final 3D output visible; it does not retroactively turn S-01 overflow into an on-paper success. No additional defect is established against that stated policy. |
| Nearby level-label overlap | Visible in the inspected section/elevation; acknowledged baseline annotation behavior, distinct from the new property/room interaction. No global annotation claim is approved. |

## Verification scope and limitations

I inspected the combined report, final source-review package/stat/history, interaction production diffs and relevant test bodies, delivery manifest, CI snapshot, catalog audits, strict/baseline IFC outputs, validator logic, captured red/green and final verification logs, and the three rendered images noted above. I confirmed the audit worktree is clean and HEAD matches the reviewed hash. This is an interaction-focused review in passes, relying on the completed individual task reviews for their independent feature audits; it is not a fresh line-by-line review of every test/catalog or of external parent branches.

Captured results support 144 dedicated integration tests, 37 focused core tests and 34 focused UI tests; the final focused rerun passes 17 tests. Affected-package Clippy completes, with formatting/i18n/docs checks recorded passing. Actual CLI build/runtime and seven-file independent IFC verification are separate from the stopped CLI test compilation. No test suite was run by this reviewer because the named questions were answerable from source and preserved evidence.

`ci-latest.json` is a point-in-time snapshot at 2026-11-09T14:20:42Z: 21 main-target PRs have all five recorded checks successful; PR356 has IFC/scripts successful and Linux/Windows/macOS still running. The three dependent PRs have no check results and await maintainer workflow approval according to the delivery records. This is not a live refreshed CI result. No full combined workspace/default-reader/platform CI or desktop GPU/OS-window proof is established. No tests were observed disabled or weakened to conceal a regression; the changed opening and attached-wall quantity expectations have explicit behavioral justification.

Publication evidence is synthetic and explicitly allowlisted; inspected source/fixture content introduces no customer geometry or credentials. I did not independently rescan every published binary artifact or revalidate every artifact checksum, and do not claim a separate comprehensive privacy/security audit.

## Declined to judge

- Full correctness of external coordinate PR284 and site PR289: their production implementations are excluded from this combined audit; only the narrow dependent acceptance results are in scope.
- Fixing baseline IfcSpace height loss: independently preexisting and outside these changes; the failed probe remains a mandatory limitation on claims.
- General level-label collision avoidance: baseline behavior outside the scoped fixes; F1 is explicitly retained because the new feature introduces it.
- Native terrain constraints surviving IFC retriangulation: outside the bounded native acceptance; documented loss remains unresolved.
- GPU/platform/default-reader behavior not exercised by the captured combined checks: no evidence to certify it here.
- Engineering certification of section catalog design capacities or open-stair structural safety: these contributions model geometry and quantities, not a certified structural design calculation.

## Assessment

**Published contributions:** suitable to remain published for upstream review, with honest evidence and limitations. Completion of the whole-delivery user journey requires F1 correction and a scoped re-review. M1-M3 are recommendations, not required additions to the consolidated fix wave.

**Ready for upstream integration: With fixes, not merge-ready as a set.** Address F1, carry the explicit merge/version resolutions into the eventual rebased branches, and obtain applicable final-head CI/maintainer approvals. The audit provides bounded integration evidence, not permission to merge or release; no upstream merge/release is authorized. The baseline room failure need not be fixed to accept these scoped contributions, but full-model IFC fidelity must remain unclaimed.
