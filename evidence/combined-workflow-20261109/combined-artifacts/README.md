# Synthetic combined workflow audit

This local audit combines22 exact main-target contributions from base30527e323e9c2e59abff927e4b76fe3ce0f83e15. It is not an upstream merge, release or replacement feature PR. Source heads are listed in included-heads.json; combined-source.patch reproduces the final source tree from that base. Final audit head: `4056f2795e7b46bec20cfd56589e9b4058973a2d`.

The fixture uses invented geometry, names and map coordinates. It combines steel sections, sectional and roller gates, open stairs, property boundaries, grids, raised walls and an upper window, a room, georeference, section/elevation and a sheet. No customer models or executable binaries are provided.

Verified:144 dedicated integration tests;37 focused core tests; 34 focused desktop tests; the CLI test target was stopped during unconditional reader dev-dependency compilation and is INCOMPLETE (actualCLI runtime passed). The actual CLI saves and reopens native/IFC files. Both independent original19 fixtures retain their known native fields and geometry, and both old19 readers refuse combined20. IfcOpenShell0.8.5 checks all7 IFCs with schema/EXPRESS, unique GlobalIds and actual shape construction. Mixed original and both reopened IFCs each have26 products and21 shapes. The20 regenerated catalogs preserve complete original message/translation entries.

**Strict full-model IFC roundtrip failed.** A room's height changes2700mm->2500mm and its volume changes; pristine baseline18 reproduces it. This remains a real failed check in public-logs/strict-ifc-roundtrip.log and strict-ifc-roundtrip-failures.json. The passing feature validator excludes IfcSpace geometry equivalence, while still checking all schema/EXPRESS, GUID uniqueness and shape construction. Do not interpret feature success as complete mixed-model IFC fidelity. Derived stair/metadata identities can regenerate on IFC import, as in the existing exporter/importer. Nearby default Level2 and Upper elevation labels overlap in these synthetic drawings.

Reproduction after building the combined CLI with no default reader features:

```text
python verify_cli.py COMBINED_CLI STEEL19_CLI STAIR19_CLI
python verify_ifc.py
python verify_ifc_full_model.py
```

Use IfcOpenShell0.8.5 and NumPy for the second and third commands. Original19 fixtures are included; compile old readers from the task9/task26 exact heads in included-heads.json. The final command exits1 on the recorded room-height discrepancy. It is deliberately not marked as a passing or expected-failure test. The baseline-room.txt and two baseline IFC outputs independently demonstrate the same older behavior.

mixed.png, section.png, elevation.png, sheet.png and sheet-3d.png were visually inspected. sheet.pdf and sheet-3d.pdf are actual CLI outputs. The plan keeps a room/property-label collision visible for final review (potential new feature interaction; readability is not claimed passing). original-layout-reproduction.* re-renders the original single-sheet overflow layout; the original command transcript is retained separately, while its first PNG was overwritten during artifact reruns. These are software-rendered model outputs, not desktop/GPU screenshots. Tests for gate section/elevation and property-boundary Level reassignment/undo/reopen are in combined_audit.rs.

No full-workspace, default-reader-feature or cross-platform combined CI claim is made. Checks used2 Cargo jobs,2 Rayon threads and4 test threads in an exclusive target. All publication files are explicitly listed in public-evidence-files.json, with SHA256 in checksums.json. Raw local logs and intermediate verifier/setup failures remain private in the audit ledger; the strict product failure is retained in the public evidence.

Final formatting, i18n/catalog checks and Clippy for core/UI/RVT/CLI all targets passed with warnings denied. The merged IFC coverage fixture independently builds361 shapes for365 products with0 schema/EXPRESS/shape problems. Root independent review remains pending.
