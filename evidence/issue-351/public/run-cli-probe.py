"""Run the external-CLI desktop probe and restore the checkout's test source afterward.
Usage: python run-cli-probe.py <checkout> <built-bimcraft-cli-executable>
Build the CLI from the same checkout before invoking this script.
"""
import os
from pathlib import Path
import subprocess
import sys

repo = Path(sys.argv[1]).resolve()
cli = Path(sys.argv[2]).resolve()
assert cli.is_file(), "Build bimcraft-cli --no-default-features first"
test_source = repo / "crates/ui-egui/src/tests/control.rs"
probe_source = Path(__file__).with_name("cli-probe-test.rs").read_text(encoding="utf-8-sig")
original = test_source.read_bytes()
assert b"fn actual_cli_mcp_reads_desktop_context_and_reports_headless_unavailability" not in original
command = ["cargo", "--config", "profile.dev.package.bimcraft-core.opt-level=0", "--config", "profile.dev.debug=0",
           "test", "-p", "bimcraft-ui-egui", "actual_cli_mcp_reads_desktop_context_and_reports_headless_unavailability",
           "--", "--nocapture", "--test-threads=4"]
env = os.environ.copy()
env.update(ISSUE23_CLI=str(cli), CARGO_BUILD_JOBS="2", RAYON_NUM_THREADS="2")
try:
    test_source.write_bytes(original + b"\n" + probe_source.encode("utf-8"))
    result = subprocess.run(command, cwd=repo, env=env, check=False)
finally:
    test_source.write_bytes(original)
sys.exit(result.returncode)
