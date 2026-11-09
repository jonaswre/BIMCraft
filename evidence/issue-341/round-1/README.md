# Straight-wall cap matching: curved-gap regression

Issue [#341](https://github.com/IPedrax/BIMCraft/issues/341), review correction from `13ee0085603f1ffa8c45168dd06f905f7b220400` to `18b91317cd357c2030fdff1f2b541cdd055df5bc`.

The new parallel-cap connection is limited to **straight walls**. Curved walls retain their existing centreline chords and crossing extensions, but cannot supply a candidate to this new connection rule. A tessellation chord does not describe the true curved endpoint tangent. Positive curved tangent-cap support is pre-existing missing behavior and is outside this change.

`arc-gap.txt` is the original synthetic public-command reproduction. Its endpoint is 2.533271431 mm beyond the real curved-wall cap along the true tangent, exceeding the 1 mm geometric tolerance. The earlier implementation incorrectly joined it using the final chord direction.

| Actual standalone CLI | South m² | North m² | North state |
|---|---:|---:|---|
| Initial local commit 13ee0085 | 39.63 | 21.81 | Enclosed incorrectly |
| Official Windows 0.11.0 control | 61.44 | 0.0 | Redundant |
| Corrected local commit 18b91317 | 61.44 | 0.0 | Redundant; gap stays open |

`cli-red-identity.json` proves that the pre-fix executable matched the original package's recorded hash. `red-cli.jsonl`, `official-cli.jsonl`, and `green-cli.jsonl` contain the actual runs. Repeat with `bimcraft-cli run --script arc-gap.txt`; `python verify-cli.py` checks all three retained results and emits `cli-proof.json`.

The new public Session test first failed on North's incorrect 21.81 m² (`red.log`, exit 101), then the complete target passed 6/6 (`green.log`). This includes the existing straight-wall, gap/side/nonbounding, invalid-input, native/IFC save/reopen/quantity and rendered-pixel tests. The focused separator selection passed 4/4, including existing crossing behavior and the internal arc-junction control; the existing curved-wall test passed 1/1. Formatting, affected-package all-target clippy, i18n update/check and the rebuilt no-default-feature standalone CLI passed. `checks.json` records exact results. Cargo was sequential with two build jobs, two Rayon threads, four test threads, and the same local opt0/debug0 overrides described in the parent README.

The parent directory's stable native/IFC/PNG artifacts and IfcOpenShell results remain evidence from commit 13ee0085. They were preserved and were not regenerated in this correction round. The final Session target reran its native/IFC/render checks; no external IFC validation or full workspace/platform CI is claimed for this round. Original IFC reopening preserves the tested areas/GUIDs but uses the existing 2500 mm default room height rather than the native 2700 mm height; this correction does not change IFC height import.

The original 62-test/tower run was not repeated. Full default-feature/workspace/platform and standard IFC CI remain required publication follow-through. The initial package is retained locally for review; only explicit manifest files are published.
