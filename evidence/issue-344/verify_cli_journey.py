import hashlib
import json
from pathlib import Path
import subprocess
import sys

cli = Path(sys.argv[1])
checks=[]
def call(name,*args):
    p=subprocess.run([str(cli),*args],capture_output=True,text=True,encoding='utf-8',timeout=120)
    Path(name+'.log').write_text(p.stdout+p.stderr,encoding='utf-8')
    checks.append(dict(name=name,args=list(args),exit_code=p.returncode))
    assert p.returncode==0,(name,p.returncode,p.stderr)
    return [json.loads(line) for line in p.stdout.splitlines() if line.startswith('{')]
def last_elements(responses):
    return next(r['elements'] for r in reversed(responses) if isinstance(r.get('elements'),list))
call('cli-journey','run','--script','journey.txt')
project=Path('industrial-gates.bimcraft')
original_hash=hashlib.sha256(project.read_bytes()).hexdigest()
native=last_elements(call('cli-native-reopen','run',project.name,'--cmd','elements {"category":"door"}','--save','native-reopened.bimcraft'))
assert {'sectional','roller'} <= {r.get('operation') for r in native}
assert json.loads(project.read_text(encoding='utf-8')) == json.loads(Path('native-reopened.bimcraft').read_text(encoding='utf-8'))
imported=last_elements(call('cli-ifc-reopen','run','industrial-gates.ifc','--cmd','elements {"category":"door"}',
    '--save','ifc-reopened.bimcraft','--save','ifc-reexport.ifc','--cmd','render {"path":"ifc-reopened-plan.png","width":1400,"height":650}'))
by_guid={r['guid']:r for r in imported}
for door in native:
    assert door['guid'] in by_guid
    if door.get('operation') in ('sectional','roller'):
        back=by_guid[door['guid']]
        assert back['operation']==door['operation']
        assert back['width']==door['width'] and back['height']==door['height']
sectional=next(r for r in native if r.get('operation')=='sectional')
call('cli-flipped','run',project.name,'--cmd','modify '+json.dumps({'id':sectional['id'],'flip_facing':True}),
    '--save','flipped-gates.bimcraft','--save','flipped-gates.ifc',
    '--cmd','render {"view":"{3D}","path":"flipped-gates-3d.png","azimuth":-70,"elevation":20,"width":1400,"height":650}')
assert hashlib.sha256(project.read_bytes()).hexdigest()==original_hash,'opening/exporting altered the source model'
for name in ['industrial-gates.ifc','industrial-gates.glb','industrial-gates-plan.png','industrial-gates-plan.svg','industrial-gates-3d.png','ifc-reopened-plan.png','flipped-gates-3d.png']:
    assert Path(name).stat().st_size>0
print(json.dumps(dict(cli_sha256=hashlib.sha256(cli.read_bytes()).hexdigest(),checks=checks,native_guids_preserved=True,
    native_save_reopen_equal=True,ifc_guids_operations_and_sizes_preserved=True,original_native_sha256=original_hash,
    original_native_unchanged=True,doors=[dict(id=r['id'],guid=r['guid'],operation=r.get('operation')) for r in native]),indent=2))
