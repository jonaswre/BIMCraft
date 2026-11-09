# Native grid IFC exchange: issue #314

Reproduction base: upstream `30527e323e9c2e59abff927e4b76fe3ce0f83e15` (0.11.0). Fixed-source outputs come from branch `fix/ifc-grid-export` on Windows 11.

From a fresh project, run `bimcraft-cli run --script oblique.txt --save oblique.ifc`, then `bimcraft-cli run oblique.ifc --cmd 'elements {"category":"grid"}'`. The unmodified base omits all grid entities; fixed output retains the three labels and endpoints. Parallel, single-axis and empty-project examples are included too.

`red-regressions.log` records four failures and a passing empty-project control on the base. `green.log` records the first five regressions passing after the fix. `green-final.log` records all **seven** final exchange regressions passing, including direct STEP entity-graph assertions, preservation of distinct/coincident/transformed axes, and unrelated or malformed annotation cases.

`verify_grids.py` independently reads IFC axes and placements with IfcOpenShell 0.8.5, checks nonempty disjoint U/V sets and labels/endpoints, then reopens each file through the actual CLI. Run `python verify_grids.py <source-checkout> <bimcraft-cli-executable>`. See `independent-grid-results.json` and `independent-verification.log` for observed results.

`ifc-validation.log` records IFC schema/EXPRESS validation and construction of every represented product's shape: oblique, parallel and lone-axis outputs each **5 products / 1 shape / 0 problems**; empty project **4 / 0 / 0**; sample house **51 / 47 / 0**; representative coverage **355 / 351 / 0**. `solid-volumes.log` independently compares a wall and flat roof with hand-calculated volumes: both **2.4 m3**, with zero validation problems.

A lone axis uses an IfcAnnotation marked `BIMCraft_GridAxis`, because IFC4 requires at least one separate U and V axis. No axes are invented. BIMCraft restores this annotation as a native axis; other IFC readers identify it as an annotation. Parallel grids use disjoint real axis sets and `IRREGULAR`.

Cargo regression runs used local-only overrides `--config profile.dev.package.bimcraft-core.opt-level=0 --config profile.dev.debug=0`. Full-workspace and platform-CI results are reported separately with the linked fix PR.
