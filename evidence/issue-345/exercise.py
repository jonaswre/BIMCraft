"""Run the original synthetic property-boundary user journey. Usage: python exercise.py CLI [LEGACY_CLI]."""
import json, subprocess, sys
from pathlib import Path
OUT = Path(__file__).resolve().parent
cli = str(Path(sys.argv[1]).resolve())
def run(args, log, executable=cli, success=True):
    p = subprocess.run([executable, *args], cwd=OUT, capture_output=True, text=True, encoding="utf8")
    (OUT/log).write_text(p.stdout+p.stderr, encoding="utf8")
    assert (p.returncode == 0) == success, (args, p.returncode, p.stdout, p.stderr)
    return [json.loads(line) for line in p.stdout.splitlines() if line.strip()]
commands = [
 'property_line '+json.dumps({"name":"Synthetic Plot","points":[[0,0],[6000,0],[6000,4000],[0,4000],[0,0]]}),
 'level '+json.dumps({"name":"Upper synthetic plan","elevation":3000}),
 'property_line '+json.dumps({"name":"Concave synthetic plot","level":"Upper synthetic plan","points":[[8000,4000],[10000,4000],[10000,2000],[14000,2000],[14000,0],[8000,0]]}),
 'elements '+json.dumps({"category":"property lines"}),
 'schedule '+json.dumps({"category":"property lines","path":"schedule.csv"}),
 'render '+json.dumps({"view":"Level 1","path":"plan.png","width":1000,"height":700}),
 'render '+json.dumps({"view":"Upper synthetic plan","path":"upper-plan.png","width":1000,"height":700}),
]
(OUT/'commands.txt').write_text("\n".join(commands)+"\n",encoding="utf8")
run(['run','--script','commands.txt','--save','plots.bimcraft','--save','plots.ifc','--save','plan.svg'],'cli-create.log')
native = run(['run','plots.bimcraft','--cmd','elements {"category":"property lines"}'],'cli-native-reopen.log')[-1]
ifc = run(['run','plots.ifc','--cmd','elements {"category":"property lines"}'],'cli-ifc-reopen.log')[-1]
for result in (native, ifc):
    plots=result['elements']; assert len(plots)==2
    assert sorted(p['area_m2'] for p in plots)==[16.0,24.0]
    assert sorted(p['perimeter_m'] for p in plots)==[20.0,20.0]
assert sorted(p['guid'] for p in native['elements'])==sorted(p['guid'] for p in ifc['elements'])
if len(sys.argv)>2:
    legacy=str(Path(sys.argv[2]).resolve())
    run(['run','--save','legacy-empty.bimcraft'],'legacy-create.log',legacy)
    run(['run','legacy-empty.bimcraft','--cmd','inspect {}'],'cli-old-file-open.log')
    run(['run','plots.bimcraft','--cmd','elements {}'],'legacy-refusal.log',legacy,False)
    assert 'unknown variant `propertyline`' in (OUT/'legacy-refusal.log').read_text(encoding="utf8")
print('PASS: CLI author -> drawings/schedule/native/IFC -> reopen, areas/perimeters and GUID identity; optional legacy refusal.')
