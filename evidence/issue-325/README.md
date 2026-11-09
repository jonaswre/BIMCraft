# Synthetic viewport placement evidence

Issue: https://github.com/IPedrax/BIMCraft/issues/325

Synthetic 1,000 by 1,000 mm square and duplicated plan names. Tested source commit: b9343587cd224e9a131d6953feef9a616a8730e4. Baseline runtime: official portable BIMCraft 0.11.0 on Windows 11. Fixed runtime: Windows 11 source build with optional DWG/RVT readers disabled.

Run `python verify-cli.py --cli /path/to/bimcraft-cli --out .` beside reproduce.txt to create/render the sheets, generate A1/A3 PDFs, save and reopen the native project, and verify preserved viewport GUIDs/references/positions/scales and byte-identical PNGs.

`before-titles.png` shows overlapping 80-character titles. `after-titles.png` and `after-titles-pdf.png` show separate titles. `before-overflow.png` shows Plan 2/3/4 stacked. Final `after-overflow.png` and `after-overflow-pdf.png` show Plan 1 and an intact border/title block; Plan 2/3/4 remain separate wholly beyond the paper. The algorithm searches adjacent/below/corner candidates and does not promise globally optimal packing, shrink scales, or rearrange earlier views. Overflow beyond the sheet is not printed.

Current-head checks: 6 public command integration tests (including exact PNG border/title-block preservation across the four-plan A3 overflow journey), 2 existing Place View UI tests, fmt, core/UI all-target Clippy with warnings denied, i18n update/check; all passed. The new border regression failed on 364b494d before correction; public-fix-round1-red.log records the changed border pixel. Twenty regenerated catalogs change source references only and preserve every message and translation.

The 23 core sheet-related tests and 7 sheet UI tests in public-core-sheets.log/public-ui-sheets.log passed previously on 364b494d. They were retained as historical evidence, not rerun or claimed for the final head. Full workspace/platform/IFC CI remains required follow-through.

Local flags: `--config 'profile.dev.package.bimcraft-core.opt-level=0' --config 'profile.dev.debug=0'`; two build jobs, two Rayon threads, four test threads. Raw environment logs, internal reports, issue snapshots and native project files are excluded from public evidence.
