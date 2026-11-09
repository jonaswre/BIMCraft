# Engineering drawing glyphs: issue #318

Official Windows portable **0.11.0** renders `1′`, `1″`, `1…`, `1漢` and `1?` identically in grid labels and supplies no fallback diagnostic. All five baseline PNG SHA-256 hashes are `f4d01f882fa3d221e5feae1bec913f471c3988427984b8f78eae977b34f3a54f`. See `baseline-portable-repro.log` and `baseline-cli/` for the reproduced output. Source base: `30527e323e9c2e59abff927e4b76fe3ce0f83e15`.

Fixed source commit: `7d5424d2d8e7cd6ac732097f8046d988023fa33f`.

With Pillow installed, run `python repro.py <official-0.11.0-cli> baseline-cli` to reproduce the original labels, then `python verify_cli.py <fixed-cli> green-cli` for the corrected command/save/open journey. Both scripts accept the executable path and generate only synthetic files. `cli-verification.log` records the fixed CLI results. The verification checks distinct RGB pixels, the unsupported codepoint diagnostic, and identical PNGs after native save/reopen. `green-cli/drawing-heads.png` enlarges the actual exported label pixels; root and implementer visually inspected it.

The fix adds prime, double-prime and ellipsis to the original lettering, and reports unsupported drawing codepoints through existing render `notes`. Repeated diagnostics are deduplicated and bounded. Unsupported labels keep their question-mark pixels. Text-note Plex/system fallback is preserved.

`red-summary.log` records four expected failures and one passing note-font control before production edits. `green-final.log` records all **five** Session integration tests passing, covering rendered pixels, diagnostics, inline/file replies, diagnostic bounds, failed-export retry, native reopen and non-Latin note-font fallback.

Affected library tests passed: font **9**, text **8**, render **62** (four preexisting ignored probes), sheet **4**, 3D dimensions **3**. Core all-target Clippy, formatting and translation checks passed. The catalog check verifies that generated changes contain source-reference comments only.

Final verification also completed seven existing text journeys (`text-journeys.log`) and one existing note journey (`existing-note-journey.log`). The five Session regressions passed again (`final-session.log`). Across the affected targets, **99 distinct tests** passed; repeated runs are not added to that count.

Local Cargo checks use uncommitted `profile.dev.package.bimcraft-core.opt-level=0` and `profile.dev.debug=0` overrides. The CLI build uses `--no-default-features` to omit optional DWG/Revit readers. Full workspace/platform/IFC CI results are tracked with the fix PR; they are not claimed by these local checks. No tests were disabled or weakened.
