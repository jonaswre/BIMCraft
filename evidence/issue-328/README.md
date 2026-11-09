# Built-in sheet title fitting

Issue: https://github.com/IPedrax/BIMCraft/issues/328

Synthetic evidence for BIMCraft 0.11.0 on Windows 11. Base: `30527e323e9c2e59abff927e4b76fe3ce0f83e15`. Fixed source: `61ab42bd42e2d30df91760e1879923e6b872629b`. The CLI was built with `--no-default-features`; optional DWG/RVT readers are unrelated to sheet lettering.

Run the reproduction from an empty directory:

```powershell
bimcraft-cli run --script reproduction.txt --save titles.bimcraft --save titles.pdf
```

The original built-in title block clips the long project name and overprints the sheet name and sheet number. The fix fits each complete value into its own cell using uniform shrinkage, preserving short values' usual size and alignment. Extremely long values can become small; this change does not introduce wrapping or truncation.

Compare `baseline-title-crop.png` with `fixed-title-crop.png`. The `*-pdf-title-crop.png` images were rasterized from the actual saved PDFs at 127 dpi using PyMuPDF. Full rendered PNGs are included. All names and commands are synthetic.

Validation:

- Regression RED: four visible-output failures; native roundtrip already preserved the broken image. The corrected antialias-aware letter-visibility test also fails on the base (38 visible H instead of 80).
- Regression GREEN: 5 tests passed, including pixel bounds, all five technical standards, A0–A4 sheets, 80 H and 160 expanded S capitals visible, and native save/reopen PNG/PDF equality.
- Existing sheet-related core tests: 23 passed; PDF test: 1 passed; documentation checks: 3 passed.
- Formatting, core all-target Clippy with warnings denied, i18n update/check passed. All 20 generated catalog changes contain only source references.
- Actual fixed CLI command → PNG/PDF/native save → reopen → PNG/PDF output passed with identical bytes after reopen. Short-title PNG/PDF bytes are identical to the base outputs.

Cargo used two build jobs, `RAYON_NUM_THREADS=2`, four test threads, and local uncommitted overrides `profile.dev.package.bimcraft-core.opt-level=0` / `profile.dev.debug=0`. Full workspace/platform/default-feature and external IFC CI remain required; they were not run locally. No desktop controls changed; the desktop paper view shares the verified core display list, but a desktop UI run is not claimed.

For repeatable CLI verification, install Pillow, set `BIMCRAFT_CLI` to the executable, and run `python reproduce.py baseline` against the base, then `python reproduce.py fixed` against the fixed build. The helper asserts native output equality and unchanged short-title output. Public verification logs replace local workspace/target paths with placeholders.
