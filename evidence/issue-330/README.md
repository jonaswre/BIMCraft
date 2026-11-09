# Default 3D viewport sheet fit

Synthetic evidence for [BIMCraft issue #330](https://github.com/IPedrax/BIMCraft/issues/330). No customer project or imported assets are included.

Tested on Windows 11 Enterprise (10.0.26300), upstream base `30527e323e9c2e59abff927e4b76fe3ce0f83e15` and the accompanying fix.

Run `bimcraft-cli run --script repro.txt --save before.bimcraft` on the base version. The default viewport is 734 x 405 mm at 1 : 100 on A3 paper (420 x 297 mm), visibly running beyond the paper in [before.png](before.png).

Run `bimcraft-cli run --script fixed.txt --save after.bimcraft --save after.pdf` with the fix. It selects 1 : 500 and a 166 x 100.2 mm viewport. [after.png](after.png) shows the entire building, sheet border, viewport title and title block. [pdf-page.png](pdf-page.png) is the first page of [after.pdf](after.pdf), rasterized by Poppler; `pdfinfo` reports one A3 page.

Reopen with:

```text
bimcraft-cli run after.bimcraft --cmd 'elements {"category":"viewport"}' --cmd 'render {"view":"A101","path":"reopened.png"}'
```

The scale and position remain unchanged. The SHA-256 of both `after.png` and `reopened.png` is `bc23170cdd7918f108e1ed2df154034b0743814d9c2c6c1c465a174fd29924e0`.

The focused public-command regression initially failed five scale-selection assertions on the base; ten tests pass with the fix. It covers the actual rendered sheet, native save/open, metric and imperial scales, technical-standard margins, explicit scales/positions, assigned view/template scales, plan-scale compatibility, an empty project, the coarsest fallback, and rollback after invalid explicit scale input. Existing desktop Place View behavior is also covered by a focused click-through test.

An assigned viewport/view/template scale stays under the user's control. With no offered scale large enough, the coarsest offered scale is used and the viewport can still exceed the paper. Choosing an initial scale does not rearrange occupied sheets or refit existing viewports.

Local development uses core opt-level 0 and debug info disabled. Standalone CLI builds use `--no-default-features`, which omits unrelated optional readers. Full workspace/default-feature, platform and IFC CI checks remain required before merge; this evidence is not a claim that those checks passed.

Focused local validation passed: 10 public-command integration tests, one new desktop click-through test, one existing desktop placement test, four existing sheet-rendering tests, one existing view-scale regression, and all three agent-documentation checks. Formatting, affected-package Clippy, `cargo i18n update` and `cargo i18n check` passed. All 20 generated catalog/template changes preserve messages and translations and refresh only source references.

Command/output logs: [RED](logs/red.log), [final GREEN](logs/green.log), [desktop](logs/ui-green.log), [existing desktop](logs/ui-existing.log), [sheet rendering](logs/sheet-module.log), [view scale](logs/view-scale-existing.log), [agent docs](logs/agent-docs.log), [Clippy](logs/clippy.log), [formatting](logs/fmt-check.log), [catalog update](logs/i18n-update.log), [catalog check](logs/i18n-check.log), [translation preservation](logs/catalog-preservation.log), [fixed CLI](logs/fixed-cli.log), [native reopen](logs/reopen-cli.log), [PDF metadata](logs/pdf-info.log). Paths in these copies are replaced with generic workspace/target tokens.

Intermediate logs are retained: the first GREEN attempt exposed ANSI's automatic switch to imperial units; the margin-only fixture now explicitly restores millimetres, with the separate imperial test unchanged. The first Clippy run requested `as_chunks::<4>()` for the new test's pixel iteration; the pixel assertion was preserved and the final integration/lint runs passed.

Fix commit: `549d322b42b33feb212c08307b86a4a8fdf6d93b`.
