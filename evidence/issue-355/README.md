# Synthetic coordinate migration and acceptance evidence

All models, dimensions and coordinates here were generated for these checks. No customer model or site data is included.

## Current-main legacy migration (issue 355)

[Report](https://github.com/IPedrax/BIMCraft/issues/355). Baseline core/CLI source: `30527e323e9c2e59abff927e4b76fe3ce0f83e15` (native format 18). The baseline CLI reproduction used an independently verified identical core/CLI checkout at `533073c5663a40038b0c9f6e2a168110734d56a2`.

An identifiable legacy `orthogonal_height` field is migrated to `height`; its clockwise rotation becomes counterclockwise by negation. Scale is retained, defaulting to 1 when absent. Modern fields are untouched. Conflicting height names, invalid values and newer formats are refused; unknown-field reporting and read-only/rollback behavior remain covered. This is a migration of these fields only, not of unrelated custom element schemas.

Candidate commit: `ca3d5e5bf3cbde1f3b0cff2d16173de967bf5d2f`, based on the baseline above. Four regression tests, five existing georeference tests and three documentation checks pass. Core all-target clippy, formatting, i18n update and i18n check pass; no translation-content change is needed.

`public-migration-red.log` records four failing regression tests on unchanged main. `public-migration-green.log` records the passing tests after the change, including the three agent-document checks. `main-legacy-red-result.json` and `main-legacy-red-transcript.json` retain the actual baseline CLI loss (height 42000→0, rotation 30 unchanged).

`main_acceptance.py` builds two synthetic fixtures through the actual CLI and opens the resulting native and IFC files again. `main-cli-transcript.json` is its actual command output. `main-acceptance-result.json` records the independent IFC checks: IfcOpenShell 0.8.5 schema/EXPRESS, GlobalId uniqueness, actual shape geometry and a separate clockwise-transform calculation. Both scales 1 and 0.9996 pass. The wall bounds remain `[0,-100,0]` to `[8000,100,3000]`mm; 14 GlobalIds are unique and the wall has eight vertices. All vertices and an additional off-axis point match the numeric reference within 0.001 mm (observed maximum 0).

```text
cargo build -p bimcraft-cli --no-default-features --config profile.dev.package.bimcraft-core.opt-level=0 --config profile.dev.debug=0
cargo test -p bimcraft-core --test legacy_georeference --test agent_docs --config profile.dev.package.bimcraft-core.opt-level=0 --config profile.dev.debug=0 -- --test-threads=4
python -m pip install ifcopenshell==0.8.5
python main_acceptance.py --cli PATH_TO_FIXED_CLI
```

Run the Python script from a writable copy of this evidence folder. `legacy_probe.py --cli PATH_TO_BASELINE_CLI` reproduces the original failure and intentionally exits nonzero on unchanged main.

## PR284 coordinate acceptance

Exact parent: `7936cfd9a241c5e5a9a0b108c722957b4077f3a0`. Acceptance-only commit: `b2ec5a400da59f7aa5373649ade761d8aec70815`, intended for the contributor's `feat/coordinates` branch. Three new public Session tests and 61 existing coordinate tests pass. Core all-target clippy, formatting and i18n update/check pass; catalog changes only update generated source references.

`acceptance.py --cli PATH_TO_PR284_CLI` creates `synthetic.bimcraft`, checks native save/reopen, exports four IFC origins and reopens them. `cli-transcript.json` is the actual output. `acceptance-result.json` contains IfcOpenShell 0.8.5 schema, shape and numeric results. All four IFC files validate and have unique IDs.

The [reported vertical-scale discrepancy](https://github.com/IPedrax/BIMCraft/pull/284#issuecomment-6741123963) remains unresolved: native shared Z is unscaled, while IFC4's single map scale is isotropic. At scale 0.9996 the 3 m wall's native top is 45 m; internal/base/survey IFC exports map it to 44.9988 m. `verify_ifc_height.py` deliberately detects this discrepancy and exits nonzero; `public-red-ifc-height.log` records that result. No failing or skipped test is committed as passing, and no production semantics are changed by the acceptance contribution. The shared-origin horizontal-scale limitation is also outside this contribution.

The PR284 branch still uses native format 17 with `coordinates`; it rejects format 18, and reports the legacy `georeference` object as left out. Its synthetic legacy/newer-format inputs document that convergence boundary. The current-main migration above does not bypass the parent's version guard.

## Execution limits

Windows 11; at most 2 Cargo build jobs, RAYON_NUM_THREADS=2 and 4 test threads. Local core opt-level 0/debug 0 overrides were used without committing profile changes. CLI builds omit optional external readers. Full default-feature workspace, platform and repository IFC CI were not run locally and remain required follow-through. Logs beginning `public-` are actual captured output with local absolute path prefixes replaced; test outcomes are unchanged. PR drafts and private review files are excluded from the public allowlist.
