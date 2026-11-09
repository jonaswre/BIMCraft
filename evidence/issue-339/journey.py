"""Synthetic public CLI journey. Usage: python journey.py BIMCRAFT_CLI red|green"""
import json, pathlib, subprocess, sys
root = pathlib.Path(__file__).parent.resolve()
cli = pathlib.Path(sys.argv[1]).resolve()
phase = sys.argv[2]
commands = [str(cli), 'run', '--script', 'repro.txt',
 '--cmd', 'render {"view":"Upper","path":"'+phase+'-upper.png"}',
 '--cmd', 'render {"view":"Section 1","path":"'+phase+'-elevation.png"}',
 '--cmd', 'render {"view":"{3D}","path":"'+phase+'-3d.png"}',
 '--save', phase+'.bimcraft', '--save', phase+'.ifc']
r = subprocess.run(commands, cwd=root, capture_output=True, text=True)
(root/(phase+'-cli.log')).write_text(r.stdout+r.stderr, encoding='utf-8')
print(r.stdout, end=''); print(r.stderr, end='')
assert r.returncode == 0, r.returncode
r = subprocess.run([str(cli), 'run', phase+'.bimcraft', '--cmd', 'elements {"category":"window"}',
 '--cmd', 'render {"view":"Upper","path":"'+phase+'-reopened-upper.png"}',
 '--save', phase+'-reopened.ifc'], cwd=root, capture_output=True, text=True)
(root/(phase+'-reopen.log')).write_text(r.stdout+r.stderr, encoding='utf-8')
print(r.stdout, end=''); print(r.stderr, end='')
assert r.returncode == 0, r.returncode
