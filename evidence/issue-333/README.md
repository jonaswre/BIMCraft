# Steel section evidence

Synthetic public command reproduction for IPedrax/BIMCraft issue [333](https://github.com/IPedrax/BIMCraft/issues/333).

`steel.txt` creates catalogue IPE/HEB members, parametric I/U/L/RHS/CHS sections, a rotated column, a sloped beam, equal bounding boxes with different plates and a solid rectangular control. It then queries member quantities and schedules. No customer model or site coordinates are included.

Run a built CLI:

```sh
bimcraft-cli run --script steel.txt --save steel.bimcraft --save steel.ifc
```

For the complete check, install IfcOpenShell 0.8.5 and run:

```sh
python verify_steel.py /path/to/BIMCraft /path/to/bimcraft-cli
```

The verifier creates native save/reopen and IFC reimport/reexport outputs, compares schedules and element takeoff with independent analytical areas and steel density 7850 kg/m³, and compares IfcOpenShell meshes with exported volumes and lateral surfaces. It checks shared body/type/material profiles, cardinal points, GlobalId uniqueness and native-reopen identity. Repository `ci/validate_ifc.py` checks IFC4 EXPRESS rules and builds every product shape. Results are in `independent-steel-results.json` and `ifc-validation.log`. All 16 member meshes passed; all three IFC files have 20 products, 16 represented shapes and zero validation problems. Meshes use 0.01 mm linear deflection and 0.05 rad angular deflection, with a 0.01% relative comparison limit; observed maximum volume and lateral differences are 0.0039% and 0.0021%. All 172 IFC root GlobalIds are unique and stable through native save/reopen.

`steel-plan.png`, `steel-section.png` and `steel-3d.png` show native output. Steel weight assumes the fixed density independently of assigned material. Native fillets use four chords per quarter; CHS meshes use 24 sides; RHS corners are sharp. IFC uses parameterized profiles and quantities use analytical areas/perimeters. Sloped native drawings retain upright ends. This does not model connections or perform structural sizing.

Native storage uses format 19. Format-18 readers reject it before dropping fields, preventing steel from becoming its bounding rectangle/circle. Older format-18 projects migrate with their original plain sections and GUIDs unchanged. Profile authoring is available through the CLI/MCP `type` command; the desktop's existing Edit Type/Duplicate paths retain and validate sections.

Full default-feature/workspace and platform CI remain required; this evidence records focused local Windows checks and the standalone CLI built without optional external DWG/RVT readers.

The native compatibility check is reproducible with an older format-18 binary:

```sh
python verify_format_guard.py /path/to/format18/bimcraft-cli steel.bimcraft
```

It must refuse the file with "file format 19; this one reads up to 18" before removing unknown fields. Version regression logs are separate from earlier implementation logs. Final native/profile migration tests pass 14/14; the public steel suite passes 7/7; the desktop structure suite passes 15/15, including invalid resize recovery and native reopen. Earlier-source 124-test results were 123 pass/1 fail (hollow cut fill); the retained pixel assertion passes after subtraction-based filling. The pre-existing hostile structure sweep completed; it was not skipped.

Final format-19 logs: `native-format-green.public.log` (7/7), `core-final-migrations.public.log` (14/14), `ui-final.public.log` (15/15), and `clippy-final.public.log`; formatting/catalog logs and actual old-reader refusal are included. `native-format-red.public.log` fails before the version bump. All `green-initial`, `ui-green`, `green-expanded`, `core-final-profile-drawing` and `core-final-integrations` log names refer to earlier source stages; `earlier-core-selection.public.log` retains the 123/1 run before the hollow fill correction. They are retained as development evidence, not presented as final-format runs. No test assertions were disabled or weakened.
