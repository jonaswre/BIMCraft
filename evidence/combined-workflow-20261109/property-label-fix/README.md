# F1 property/room label correction evidence

All model coordinates and names in this package are invented. `synthetic-mixed.bimcraft` is an exact copy of the existing combined synthetic input; the original input and original failing evidence were not edited.

`before-plan.png`, `before-sheet.png` and `before-sheet.pdf` show the reviewed behavior. The before sheet PNG is byte-identical to the original failing combined sheet. `after-*` are separate outputs from the rebuilt corrected CLI, opening the same native model and rendering Level 1 / S-01.

`red-plan.log` and `red-sheet.log` record public output-geometry/pixel regressions failing before the production edit. The intermediate test-harness failures remain as separately named logs. Final green logs identify the selected targets and counts; no workspace-wide validation is claimed.

Both `*-invariance.json` files assert exact equality of every persisted native field after reopen/render/save. This includes room identity, anchor and area inputs, property identity, polygon, types, library and georeference. The fix changes only drawn property lettering when a visible room tag conflicts. It does not fix baseline IfcSpace IFC height/volume loss, terrain exchange, coordinate scale discrepancies, or general annotation collisions. No IFC geometry acceptance was repeated for this render-only correction.

`catalog-reference-only-*.json` records all catalog files as unchanged after ignoring only source-reference lines. `original-evidence-integrity.json` checks all 61 previously checksummed combined artifacts remain unchanged.

`verify_cli.py CLI before|after` reproduces the bounded synthetic CLI journey; the CLI path is runtime-only. No executable binary, customer input, credentials or private path is included in the public allowlist. `public-evidence-files.json` is the explicit publication scope; `checksums.json` hashes that scope. Test/compiler log paths are normalized to public tree labels.

This evidence supports only the scoped F1 correction. M1 (planar verifier tolerance), M2 (permanent spawned CLI coverage) and M3 (permanent final CRS/map-conversion assertions) remain nonblocking recommendations. Whole-delivery merge/version coordination, final-head CI and maintainer approval remain separate.
