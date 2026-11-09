# Delivery decisions

These are delivery decisions, not approval to merge upstream. Original logs, intermediate failures and worktrees remain preserved. Final observed results supersede earlier resource-management instructions.

1. Open-stair work was scheduled before selection context to overlap independent geometry checks. Scope and review gates stayed unchanged.
2. Open-stair construction updates existing issue189 and Discussion220, following the maintainer's tracking direction. PR354 references them without claiming to close an already-closed issue. IFC roles67/68 are reserved for its metadata.
3. The property-boundary task's unrelated whole-command stress sweep was stopped after more than13 minutes. Its log remains incomplete. Focused property, rollback, permissions, UI and cache checks passed; no test was disabled.
4. Coordinate and terrain acceptance contributions target the exact existing contributor branches. They do not duplicate those whole features on main or establish compatibility between the parents.
5. Each issue used an independent agent and isolated source worktree, as requested.
6. Existing coordinate/site work received acceptance coverage and any measured defect report. Already-working behavior was not presented as a new bug.
7. Focused local checks precede publication. Full configured platform/IFC CI is tracked separately and is never inferred from a local build.
8. CLI/MCP is the primary authoring entry point for these contributions. Changes to existing desktop behavior receive separate UI workflow tests; these PRs do not claim new desktop authoring palettes.
9. A lone IFC grid axis uses IfcAnnotation because IfcGrid requires nonempty exclusive U/V axes. No unprovided axis is invented.
10. The raised-room task retained a full-suite timing failure and its unchanged isolated test result. A repeated whole suite was not used solely to replace the earlier result with green.
11. Local Cargo work uses exclusive targets, at most2 build jobs and4 test threads. Broad unrelated stress sweeps are left to configured CI. Resource limits never authorize weakening tests.
12. Incomplete unrelated local suites remain incomplete. Repeating expensive runs solely to replace an incomplete record was avoided; focused behavior and real CLI/native evidence remain separate.
13. An accidentally broad opening-related filter was stopped with151 tests passed and one unrelated long-running test. It is not reported as152 passed. New host/output regressions and relevant existing door/opening checks ran separately.
14. IFC GUID roles were coordinated across branches: grids59/60, wall intent61/62, property63-66, stairs67/68. Steel's51/52/57/58 remain unchanged. The combined audit checks actual unique GlobalIds.
15. A preliminary task14 instruction allowed stopping unrelated reader compilation. **That did not describe the final outcome:** compilation completed, exposed a missing test type and then a public-response-shape assertion error. Both were corrected; the enabled CLI/MCP suite subsequently passed27/27, including after hardening. Full details and original failures are in the task's evidence.
16. The title-rendering task was allowed to stop an unrelated default-reader build if necessary and use a real CLI build without optional readers. Actual observed outcomes are retained in that task's logs; no default-feature success is inferred from the alternative build.
17. Steel work was started earlier to overlap independent smaller tasks. Its scope/base and independent review requirements stayed unchanged.
18. Raised opening lookup is a narrow policy enhancement beyond the existing diagnostic PR: cut-eligible hosts have priority, raised fallback is same-level and unique, and ambiguity requires an explicit host. Existing fit/vertical/overlap validation remains authoritative.
19. An explicitly requested different opening level controls absolute elevation after host selection. Omitted/same-host-level base-offset behavior retains its existing contract. Invalid levels and impossible fits are refused with rollback.
20. The separator-end task discovered that existing room_separator accepts an unknown key. That broader policy was not changed by this geometry fix. Its rollback regression instead uses genuinely invalid numeric input, and does not claim unknown-key rejection.
21. Coordinate PR284's native horizontal-only scale contract was preserved. Its measured IFC4 isotropic Z-scale mismatch was reported and remains unresolved. The main legacy-field migration is a separate contribution.
22. Combined omitted-level opening lookup first performs ordered all-level cut-host search, then a unique raised fallback on the default level. Explicit level lookup stays exclusive. This composes the actual production contracts of tasks18/21/22.
23. In combination, explicit wall controls host and explicit level controls elevation. The formerly ignored-level below-host case remains as refusal/rollback. A valid tall lower-level host now proves the successful explicit upper-level elevation.
24. Strict whole-model IFC comparison found room height2700mm becoming2500mm; pristine base18 reproduces it without these features. The strict probe remains **FAILED**, with baseline evidence. Targeted feature validation explicitly excludes room-height equivalence and must never be represented as complete mixed-model geometry fidelity. No existing assertion or tolerance was weakened to conceal this result.

Steel and stairs independently use native format19 in their main PRs. Their combined audit uses steel19 then stairs20 and checks both original19 fixtures and both older readers. The later upstream contribution still needs migration reconciliation before both land.

If a bounded decision is wrong, a caller may need explicit host/level parameters, a parent contribution may need rebasing, or an unsupported IFC roundtrip may remain unsupported. Those limits are disclosed in the inventory and evidence; synthetic migrations do not establish compatibility with unrelated older/custom projects.


## Final review decisions

25. final-review F1 is a real new rendering defect and must be corrected on existing task11 PR352 and combined audit, preserving original failed sheet evidence. Why: a natural coincident room/plot center makes printed areas unreadable. Cost if wrong: bounded automatic text placement may need later adjustment; no geometry/model-coordinate workaround is accepted.

26. retain M1-M3 as nonblocking recommendations after explicit final review: tighter planar verifier tolerance, permanent spawned-CLI regression and permanent final CRS/map assertions. Why: separate strict geometry/Rust checks and executed real probes address current behavior. Cost if wrong: future regressions may escape those particular permanent harness assertions; no broad precision/whole-parent claim is made.

27. accept final reviewer declined-to-judge external coordinate/site production completeness, baseline room-height repair, general baseline level-label layout, IFC terrain constraints, unexecuted combined GPU/default-reader/platform behavior and structural engineering certification as bounded exclusions. Why: external/baseline/engineering scope is not established by these contributions, and missing evidence cannot certify it. Cost if wrong: parent rebase/IFC fidelity/layout/platform or design-safety work remains required; inventory must explicitly retain these limits and no upstream merge/release is authorized.


28. The one consolidated F1 fix wave and scoped re-review are complete: F1 is addressed, no new Critical/Important breakage. The exact reviewed task11 head and audit branch were pushed with clean source/remote parity. Cost if wrong: the bounded label-avoidance behavior may need further refinement; no general annotation or full upstream merge readiness is claimed.

29. preserve the independently tested contribution heads and30527-based audit after late upstream main6227 adds93changed files. Record read-only current-main merge results (18of22conflicting, fourclean) and PR352 hostedCI blocked byconflicts; do not present prior green runs as current-main compatibility. Why: current-main integration also changes native history, dry-run, sheets, rooms and rendering, and needs a separate coordinated rebase/verification rather than silently expanding the audited set. Cost if wrong:18PRs require additional integration work before landing; these published contributions are reviewable, not merge-ready as a set.
