# Synthetic IFC quantity evidence

Issue: https://github.com/IPedrax/BIMCraft/issues/348
Fixed local head: `be6881c9d72fa7973bc4d328867448bf60cc9b54`.
Tested base: `30527e323e9c2e59abff927e4b76fe3ce0f83e15`, BIMCraft 0.11.0, Windows 11 Enterprise.

All shapes, dimensions and coordinates in this package are synthetic. No customer model or Autodesk data is included.

## Reproduction

```text
bimcraft-cli run --script reproduce.txt --save baseline.ifc
python quantities.py baseline.ifc
```

`baseline-quantities.json` was observed with the unmodified base: roof, ceiling and pad/strip footing have no quantity sets, and the attached wall has only Length/Width. `fixtures/` contains the fixed command scripts, native saved projects and IFC exports for the ten regression tests (eleven fixture files because the round-pad/strip test exports two projects).

## Regression and independent checks

```text
cargo test -p bimcraft-core --test ifc_supplementary_quantities -- --test-threads=4
cargo test -p bimcraft-core --lib ifc -- --test-threads=4
cargo test -p bimcraft-core --test agent_docs -- --test-threads=4
python validate_ifc.py --cli bimcraft-cli --test crates/core/tests/ifc_supplementary_quantities.rs --out fixtures
python validate_measurements.py independent-ifc.json
```

Redirect the validator's stdout to `independent-ifc.json`; the measurement checker reads that file. IfcOpenShell 0.8.5 is required. For a standalone CLI build, `cargo build -p bimcraft-cli --no-default-features` omits unrelated optional external readers. Local compilation additionally used uncommitted core opt-level=0/debug=0 overrides, two build jobs and RAYON_NUM_THREADS=2. Full default-feature CI remains required.

Observed results:

- RED: seven initial numerical public Session save/reopen/export cases failed on the missing quantity data before the exporter change.
- GREEN: all ten final cases pass; 92 existing IFC-filtered core tests and three documentation-map tests pass.
- Eleven independent CLI command -> native save -> reopen -> IFC exports validate as IFC4, with zero EXPRESS/schema statements and unique GlobalIds in each file.
- Every physical product's shape builds. Footing and attached-wall net volumes match independent tessellated volumes (round tessellation allows 0.2%); ceiling/roof net areas match upward mesh surface areas. All exported quantity names match IfcOpenShell's IFC4 templates.
- Six independently measured bounds and ten additional hand-calculated quantities pass (see `independent-measurements.json`).
- Core all-target clippy with warnings denied, fmt, i18n update/check pass. Generated UI catalog changes only update the existing `Phase Filters` source-reference line; translated text is preserved.

No desktop controls or native format fields changed. Full workspace/platform/default-feature/IFC CI is publication follow-through and is not represented as a local pass.


The explicit public manifest contains 53 files. Actual sanitized RED/GREEN, existing IFC, docs, clippy, formatting, translation and build logs are in public-logs/. public-logs/checks.json records exact commands, environment overrides and exit codes; an empty fmt.log is the actual successful command output. Raw local logs remain private. This packaging step repeated no tests or builds.
