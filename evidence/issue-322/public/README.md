# Script result references: synthetic reproduction and evidence

Feature request: https://github.com/IPedrax/BIMCraft/issues/322

Tested BIMCraft 0.11.0 on Windows 11 Enterprise (10.0.26300), base `30527e323e9c2e59abff927e4b76fe3ce0f83e15`, feature head `c8683ecbebf28c90230f9ac8189de3bc355ef6ff`. All models, command files and outputs here are original synthetic examples. No customer or Autodesk files are included. Log copies replace workstation paths with placeholders; assertions, failures and counts are preserved.

## Reproduce the user journey

With the feature build on PATH, run from this directory:

```sh
bimcraft-cli run --script references.txt --save references.bimcraft
bimcraft-cli run references.bimcraft --cmd 'elements {"category":"doors"}'
```

The script makes two walls, hosts a door on the second via `@walls.ids.1`, then uses `$last` to set its mark to `Entry`. Reopening returns that door with its original host GUID. `references.png` shows the resulting opening on the vertical wall. `cli-output.json` captures creation and reopened query results.

`subprocess_probe.py` exercises the actual CLI and JSON-line MCP server. Set `BIMCRAFT_CLI` to the built executable, or put `bimcraft-cli` on PATH, then run `python subprocess_probe.py`. It verifies atomic failure and retry, save/reopen/undo, read-only edits and own-file overwrite protection. It writes only synthetic files in its directory and uses an empty plugin folder. The JSON transcripts and `subprocess-results.json` capture the observed output.

## Syntax and limits

A line can be named `@name = command {params}`. Whole-string values `@name`, `$last` and `$-2` use the earlier result's `id`; paths such as `@walls.ids.1` select a field or array item. Names/results are local to one script. Comments and blank lines do not count. `@@` and `$$` escape a literal leading character; strings are not interpolated.

Expanded parameters are limited per line to 16,777,216 accounting bytes: 128 bytes per JSON value plus UTF-8 string/key bytes, charged before copying. Depth is limited to 128, including values returned by external commands. Errors summarize the reference without copying the earlier reply. Existing 10,000-item ID lists remain supported.

## Evidence

- Baseline `logs/red.log`: 0 passed / 6 failed before implementation.
- `logs/expansion-red.log`: 1 passed / 3 failed before hardening; repeated references reached a recipient, a diagnostic grew to 61,575 bytes, and a 256-level external reply reached execution. These deterministic tests avoid OOM and timing assertions.
- `logs/expansion-green.log`: all 13 public Session regressions pass, including bounded expansion/depth/diagnostics, rollback/retry, 10,000 IDs, bridge restrictions and save/reopen host identity.
- `logs/cli-focused.log` retains an intermediate new-test Guid inference compiler error; `logs/cli-all.log` retains an intermediate assertion using the wrong MCP response shape. Both were corrected; `logs/cli-final.log` shows all 27 enabled default-feature CLI/MCP tests passing.
- Existing plugin permission subset: 3 tests passed. `logs/clippy-final.log`, `logs/i18n-update.log` and `logs/i18n-check.log` record the successful focused lint and translation checks. Formatting also passed. Catalog canonical contents and translation messages were unchanged.
- `logs/subprocess-final.log` plus the JSON transcripts show the real executable journeys passing.

Local Cargo used 2 build jobs, 4 test threads, `RAYON_NUM_THREADS=2`, a dedicated target directory, and uncommitted profile overrides `profile.dev.package.bimcraft-core.opt-level=0` / `profile.dev.debug=0`. Full workspace/platform/independent IFC checks remain CI follow-through; these focused results do not claim those checks passed.

## Unmerged dry-run PR compatibility

PR https://github.com/IPedrax/BIMCraft/pull/287 (inspected head `f8710d95d798b5d3ef73d1d70ff08e61f3105588`) is not included in the feature branch. A temporary comparison combined its actual patch and these references. Without adapting its raw-token preflight, a named `save` wrote a synthetic file during dry-run and a named `new` was accepted: `logs/compat-red.log` shows 1 pass / 2 failures.

`pr287-named-preflight.patch` is the narrow integration adjustment: use shared `engine::script_line` in dry-run and bridge preflight. `compat_script_references.rs` contains the comparison regression. `compat-full.patch` is the full reference/compatibility difference against the applied-PR tree, including normal regressions. With the adaptation and final bounds, `logs/compat-final-green.log` shows 3 dry-run compatibility + 13 normal reference tests passing: model/history/IDs restored, named file/replacement commands refused, no file output. This does not claim all PR287 tests passed.
