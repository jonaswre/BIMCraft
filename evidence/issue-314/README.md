# Native grid IFC exchange: issue #314

Reproduction base: upstream `30527e323e9c2e59abff927e4b76fe3ce0f83e15` (0.11.0). Fixed-source outputs come from branch `fix/ifc-grid-export` on Windows 11.

From a fresh project, run `bimcraft-cli run --script oblique.txt --save oblique.ifc`, then `bimcraft-cli run oblique.ifc --cmd 'elements {"category":"grid"}'`. The unmodified base omits all grid entities; fixed output retains the three labels and endpoints. Parallel, single-axis and empty-project examples are included too.

`red-regressions.log` records four failures and a passing empty-project control on the base. `green.log` records the first five regressions passing after the fix. `green-final.log` records all **seven** final exchange regressions passing, including direct STEP entity-graph assertions, preservation of distinct/coincident/transformed axes, and unrelated or malformed annotation cases.

`verify_grids.py` independently reads IFC axes and placements with IfcOpenShell 0.8.5, checks nonempty disjoint U/V sets and labels/endpoints, then reopens each file through the actual CLI. Run `python verify_grids.py <source-checkout> <bimcraft-cli-executable>`. See `independent-grid-results.json` and `independent-verification.log` for observed results.

`ifc-validation.log` records IFC schema/EXPRESS validation and construction of every represented product's shape: oblique, parallel and lone-axis outputs each **5 products / 1 shape / 0 problems**; empty project **4 / 0 / 0**; sample house **51 / 47 / 0**; representative coverage **355 / 351 / 0**. `solid-volumes.log` independently compares a wall and flat roof with hand-calculated volumes: both **2.4 m3**, with zero validation problems.

A lone axis uses an IfcAnnotation marked `BIMCraft_GridAxis`, because IFC4 requires at least one separate U and V axis. No axes are invented. BIMCraft restores this annotation as a native axis; other IFC readers identify it as an annotation. Parallel grids use disjoint real axis sets and `IRREGULAR`.

Cargo regression runs used local-only overrides `--config profile.dev.package.bimcraft-core.opt-level=0 --config profile.dev.debug=0`. Full-workspace and platform-CI results are reported separately with the linked fix PR.

Final source commit: `fb092ab888e7c04b8ca19d594f5e9f39a592ee3f`. `green-after-catalogs.log`, `clippy-final.log` and `i18n-check.log` retain the final focused test, lint and translation checks. The eleven generated catalog changes contain source references only, checked in `catalog-reference-verification.log`.

The broad local core run is **INCOMPLETE**. `core-tests.log` and `core-incomplete-summary.json` retain 1,444 completed passes, eight preexisting ignored tests and zero observed assertion failures. This includes 81 completed IFC/import tests. The run was intentionally interrupted during unrelated long geometry/hostile-input cases after severe shared-host paging. It has no full-suite summary and did not reach integration/doc tests. The seven grid integration regressions ran separately and passed. No source or CI test was disabled, skipped or weakened.

PR CI follow-up at source head 1d4279ebce23bd15a249038e82da9afd4346acc0: initial Linux and Windows runs each completed all1540 core tests (nine existing ignored) and then failed agent_docs because root CLAUDE.md lacked a direct link to the new IFC exchange note. The one-line documentation correction reproduces RED and passes all3 documentation tests, format and i18n checks locally. Initial macOS exited SIGABRT after1504 logged passes and eight ignored; no assertion, stack trace or stated cause appears. It is not a passing run or an identified root cause. The unchanged full CI workflow reruns on the corrected head; final check results belong to the PR.
