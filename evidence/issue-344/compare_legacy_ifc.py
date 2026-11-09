"""Compare unchanged-base and final CLIs using only existing door/window operations."""
import collections
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import ifcopenshell

results=[]
for label, executable in zip(('baseline18','final'),sys.argv[1:]):
    cli=Path(executable)
    script=Path('legacy-'+label+'.txt')
    script.write_text('\n'.join([
        'wall {"points":[[0,0],[7000,0]],"height":4500}',
        'door {"at":[2000,0]}',
        'window {"at":[5000,0],"family":"Sliding Window"}',
        'save {"path":"legacy-'+label+'.ifc"}',
    ])+'\n',encoding='utf-8')
    def call(name,*args):
        p=subprocess.run([str(cli),*args],capture_output=True,text=True,encoding='utf-8',timeout=120)
        Path(name+'.log').write_text(p.stdout+p.stderr,encoding='utf-8')
        assert p.returncode==0,(label,p.returncode,p.stderr)
    call('legacy-'+label+'-export','run','--script',str(script))
    call('legacy-'+label+'-reopen','run','legacy-'+label+'.ifc','--save','legacy-'+label+'-reexport.ifc')
    row=dict(cli=label,sha256=hashlib.sha256(cli.read_bytes()).hexdigest())
    for key,path in [('before','legacy-'+label+'.ifc'),('after','legacy-'+label+'-reexport.ifc')]:
        model=ifcopenshell.open(path)
        row[key]=dict(root_counts=dict(sorted(collections.Counter(e.is_a() for e in model.by_type('IfcRoot')).items())),
            window_partitioning=[e.PartitioningType for e in model.by_type('IfcWindowType')])
    results.append(row)
assert results[0]['before']==results[1]['before']
assert results[0]['after']==results[1]['after']
assert results[0]['before']['window_partitioning']==['DOUBLE_PANEL_VERTICAL']
assert results[0]['after']['window_partitioning']==['SINGLE_PANEL']
print(json.dumps(dict(synthetic_legacy_only=True,unchanged_base_and_final_observations_equal=True,results=results),indent=2))
