# Validation

Base source: `30527e323e9c2e59abff927e4b76fe3ce0f83e15`. Validated implementation: `71398cf01f2a444bc53412af58bcf124ce964d29`. Windows 11 Enterprise, version 10.0.26300. Local profile overrides: `--config profile.dev.package.bimcraft-core.opt-level=0 --config profile.dev.debug=0`; no profile edits are committed. Cargo ran sequentially with two build jobs, RAYON_NUM_THREADS=2 and at most four test threads.

## Test-first evidence

Before production changes, `cargo [profile overrides] test -p bimcraft-core --test raised_wall_openings -- --test-threads=4` failed with the expected host refusal (see public-logs/red.log). With the full new target, two existing-policy controls passed and three enhancement tests failed (public-logs/red-complete.log). Issue #335 was filed with that evidence before the implementation.

After the change, all five tests passed (public-logs/green-final.log). The primary test creates each opening through public Session commands, checks its desktop preview matches, checks the selected host and offset, verifies that 3D output changes, saves a native project, opens it in a fresh session without dropped content, and checks complete public element records and rendered output remain identical.

## Existing behavior and repository checks

- Existing openings-past-top module: 7 passed, including 3D, plan, elevation/section and IFC opening output.
- Existing door/window lookup and IFC marks: 2 passed.
- Existing door, preview, #212 diagnostic, family sill, graphics and phasing tests selected by `tests::a_door`: 12 passed.
- Existing curved-wall opening test: 1 passed.
- Documentation-map target: all 3 passed. The existing root map already links the updated levels/walls/roofs area.
- `cargo i18n update` and `cargo i18n check`: passed. PO/POT changes only update source references; messages and translations are preserved.
- `cargo clippy [profile overrides] -p bimcraft-core --all-targets -- -D warnings`: passed.
- `cargo fmt --check`: passed.
- Standalone `cargo [profile overrides] build -p bimcraft-cli --no-default-features`: passed. Optional DWG/Revit reader features are omitted from this local executable.
- Existing desktop `tests::doors_and_windows_go_into_the_wall_clicked`: 1 passed (one test selected, runtime 1.08 s). This opens the real tool, places a door, observes an empty-space refusal and places a window.

Each check has its own sanitized public log; checkout and target directory prefixes are replaced by `<checkout>` and `<target>`.

## Real CLI output

The two commands in README.md ran as separate real CLI processes. They created door 100003 and window 100004, both hosted by wall 100002, with unchanged host-relative sills 0 and 900 mm. Reopened complete public opening records match the created records. Native format remains the existing format 18; no migration or schema change is introduced.

The before/after images differ. The after image was inspected: the door and window are visible in the elevated wall, with both level datum lines below it. After and reopened PNG SHA-256 are identical:

```text
6512E6774808ABFA727401DFB033923C618E38B7487430519EA5E6BA030FA815
```

The exact command output, original synthetic model, images and hashes are included. No customer model or real site geometry is used.

## Limits

This validates the raised-wall CLI/MCP command journey and shared preview behavior. It does not claim an interactive desktop raised-wall placement journey. Existing desktop placement is checked separately. Full workspace/default-feature/platform tests, the hostile-command sweep and independent IFC CI were not run locally; those CI checks remain required. Local verification was deliberately bounded to affected behavior on the shared build host; the full workspace and stress suites remain CI follow-through. No test was disabled or weakened to obtain a pass.
