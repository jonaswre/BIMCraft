# Review fix 1: sloped CHS bounds

Synthetic public-command regression for issue 333 / review I1. The fixture creates a CHS 200x10 beam with a 3000 mm plan span and 4000 mm rise, plus a column at [4500,200] spanning -100..4100 mm. The column hides the former inaccurate CHS bounds (-60..4060 mm), but not the native steel extremities (-165.240810..4165.240810 mm).

The public regression in `crates/core/tests/steel_profiles.rs`, module `sloped_hollow_bounds`, checks actual native sweep vertices in four orientation/rise/justification cases, preserves the plain-round tube control, and compares full versus deferred section fit, pixels, datum endpoints, overlay PNG, and every pixel's pick/face/coverage. It also asserts that the beam is visible. The former implementation fails the enclosure and fitted-view assertions (1 pass, 2 failures); the correction passes all three tests.

The corrected conservative prism envelope is -166.666666668..4166.666666668 mm for the central sloping case. Plain-round geometry retains its perpendicular tube envelope -60..4060 mm. The prism bounds are conservative; they need not coincide with the 24-sided native CHS vertices.

- `sloped-chs-deferred.png`: deferred native section geometry, 800x600.
- `sloped-chs-section.png`: full reference section with datum/title overlay, 800x600.
- Both PNGs were opened and visually inspected. The automated test proves identical full/deferred geometry and overlaid output.
- `*.public.log`: sanitized command output. `$REPO` and `$TARGET` replace local absolute paths. Historical RED output intentionally contains assertion failures.

Reproduce with a checkout containing the review fix, an exclusive Cargo target, `CARGO_BUILD_JOBS=2`, `RAYON_NUM_THREADS=2`, and the local compiler overrides below. Set `BIMCRAFT_STEEL_BOUNDS_EVIDENCE` to an output directory to generate the PNGs.

```text
cargo --config 'profile.dev.package.bimcraft-core.opt-level=0' --config 'profile.dev.debug=0' test -p bimcraft-core --test steel_profiles sloped_hollow_bounds:: -- --test-threads=4 --nocapture
```

The source change distinguishes profiled CHS from plain round runs before choosing the existing perpendicular tube envelope. Profiled CHS falls through to the existing upright-end prism envelope, without triangulating for bounds. This round does not change IFC output or the independent verifier; the initial evidence remains historical and unchanged.
