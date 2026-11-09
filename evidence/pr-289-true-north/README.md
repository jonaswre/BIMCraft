# Site and TrueNorth acceptance

Tested PR [#289](https://github.com/IPedrax/BIMCraft/pull/289) at `5f666a84a45f6f71bf7dd399eb2e0edfe752be40`, BIMCraft CLI 0.11.0, Windows 11. This is a passing acceptance of the existing convergence implementation, not a reproduced defect.

All six fixtures are synthetic: latitude ±45 degrees, longitude 1/3/5 degrees, UTM zones 31N/31S and Lambert-93; the geometry is one 10 m × 0.1 m × 3 m wall. No site/customer data or external service is involved.

Run with Python packages `ifcopenshell==0.8.5`, `pyproj==3.7.2`, `pytest`:

```text
python accept_georef.py PATH/TO/bimcraft-cli OUTPUT_DIRECTORY
```

The script builds each fixture through the CLI, saves its native project, reopens it in a new process, exports IFC, validates it independently, then reopens IFC in the CLI. PROJ 9.5.1 independently recomputes the numeric oracle in `georef_cases.json`.

All six cases passed schema/EXPRESS validation (zero errors), GUID uniqueness (14 IDs per file), site latitude/longitude and elevation, map origin/rotation/scale, TrueNorth direction, wall bounds and 3 m³ volume. `synthetic/acceptance.json` contains the measured results. Both signs of convergence and both hemispheres are covered; the central-meridian case rotates the project by 90 degrees. The largest absolute geodesic bearing of a mapped 1 km true-north ray was 0.000111 degrees; the largest distance difference was 0.002043 m.

Convention: TrueNorth is counter-clockwise from project +y. PROJ convergence is signed clockwise from true north to grid north. Therefore map x-axis rotation is `convergence - true_north`; the context direction is `(-sin(true_north), cos(true_north))` without adding convergence again. Tolerances: 1 mm map positions, 2e-8 direction components, 1e-7 point scale, 0.001 degrees ray bearing, 0.1 m ray length (the map conversion approximates a tangent plane).

Scope: horizontal placement and origin elevation. The native horizontal-only scale versus IFC4 isotropic nonzero-Z issue is separate. These checks do not claim vertical datum support, desktop interaction coverage, default DWG/RVT reader coverage, or full workspace/platform CI. The CLI was built without optional DWG/RVT readers using local opt-level=0/debug=0 overrides; those settings are not source changes.

Acceptance contribution commit: 3317917e24a305cd91387d81e4051e6975dbed24 (parent 5f666a84a45f6f71bf7dd399eb2e0edfe752be40).

Final local checks: all passed. New public-Session acceptance: 1 test / 6 cases; agent docs: 3 checks; existing georeferencing suite: 30 tests. Actual CLI / PROJ / IfcOpenShell acceptance: 6 cases. Formatting, targeted core acceptance Clippy (-D warnings), cargo i18n update and cargo i18n check passed. Translation catalogs have no content changes. Exact commands and sanitized captured output are in checks/. Full workspace/default-feature/platform CI remains pending publication.

There is no RED product failure: the parent behavior already passes. These changes add independent coverage; no production fix is claimed.
