# Desktop selection and view context

Related feature request: [IPedrax/BIMCraft #351](https://github.com/IPedrax/BIMCraft/issues/351).

A connected agent can call `desktop_context` with `{}` to read the selection and active view from the desktop window. The authenticated bridge operation is `context`. Selected elements have `id`, `guid` and `category`; active views have `kind`, `name`, `id` and `guid`.

The snapshot follows user selection, navigation, clearing, deletion and New/Open. It adds no model edit or undo step. Home returns an empty selection and null active view. Built-in views without model elements have null IDs/GUIDs. A project replacement between frames asks the agent to retry instead of returning stale IDs. Headless MCP returns an explicit `--connect` diagnostic. The existing token and read-only boundaries remain in force.

The synthetic fixture is a rectangular wall loop, independent of any customer project:

```text
wall {"points":[[0,0],[8000,0],[8000,6000],[0,6000]],"closed":true}
```

The RED log shows a real canvas wall pick followed by a TCP request rejected as an unknown operation. GREEN tests exercise the actual BimApp, egui pointer/navigation controls and TCP bridge. The external CLI probe additionally runs the actual standalone CLI against that window harness, checking selection/GUID, schedule navigation, read-only edit refusal, headless unavailability and closed-window connection behavior. This is an automated egui workflow, not a native operating-system window screenshot.

To reproduce the external CLI proof after applying the change:

```text
cargo --config profile.dev.package.bimcraft-core.opt-level=0 --config profile.dev.debug=0 build -p bimcraft-cli --no-default-features
python public/run-cli-probe.py <checkout> <built-bimcraft-cli-executable>
```

Set `CARGO_TARGET_DIR` to the build directory first. The probe temporarily appends a test to the existing control test module and restores its exact original bytes afterward. It uses only synthetic shapes and a local temporary bridge file; no generated token is printed. It requires the checkout and CLI to be built from the same source.

Cargo checks used at most two build jobs, four test threads and `RAYON_NUM_THREADS=2`. The local profile overrides above reduce build cost; no profile change is committed. The standalone CLI omitted only optional DWG/RVT readers; full default-feature workspace/platform/IFC CI remains required before merge.

Public logs are actual captured outputs with local checkout/target paths replaced by placeholders. No bridge credentials, private models, site coordinates or customer data are included. Exact included files are enumerated in `public-evidence-files.json`.

Verified local source: `3219a10bfce22a70199a7c6caef5dd35e1882f1b`, based on `30527e323e9c2e59abff927e4b76fe3ce0f83e15`.

Passing checks: 16 desktop control tests, 8 core bridge tests, 5 core read-only tests, 3 documentation-map tests; actual CLI workflow probe (1) and argument/headless assertions (3); affected default-feature all-target Clippy with `-D warnings`; `cargo fmt --check`; `cargo i18n update` and `cargo i18n check`. All 11 changed catalogs preserve message text and translations; only source references changed.
