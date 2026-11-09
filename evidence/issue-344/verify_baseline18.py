import hashlib
import json
from pathlib import Path
import subprocess
import sys

reader = Path(sys.argv[1])
reader_hash = hashlib.sha256(reader.read_bytes()).hexdigest()
assert reader_hash == '44aa58ea0924903672def0e92832be624b1f14c3b45730d322b32001cd45237f'
rows=[]
for op in ['sectional','roller']:
    initial=Path('enum-'+op+'-format18.bimcraft')
    initial_copy=Path('initial-mislabeled-'+op+'-actually17.bimcraft')
    if not initial_copy.exists():
        initial_copy.write_bytes(initial.read_bytes())
    fixture=json.loads(initial_copy.read_text(encoding='utf-8-sig'))
    assert fixture['format'] == 17
    fixture['format']=18
    fixture_path=Path('enum-'+op+'-format18.bimcraft')
    fixture_path.write_text(json.dumps(fixture,ensure_ascii=False,indent=2),encoding='utf-8')
    def request(i,name,args):
        return {'jsonrpc':'2.0','id':i,'method':'tools/call','params':{'name':name,'arguments':args}}
    requests=[{'jsonrpc':'2.0','id':1,'method':'initialize','params':{'protocolVersion':'2025-06-18'}},
              request(2,'execute',{'command':'wall','params':{'points':[[0,0],[7000,0]]}}),
              request(3,'query_elements',{}),
              request(4,'execute',{'command':'open','params':{'path':fixture_path.name}}),
              request(5,'query_elements',{}),
              request(6,'execute',{'command':'save','params':{'path':'baseline18-after-refused-'+op+'.bimcraft'}})]
    p=subprocess.run([str(reader),'mcp'],input='\n'.join(map(json.dumps,requests))+'\n',capture_output=True,text=True,encoding='utf-8',timeout=60)
    Path('baseline18-'+op+'-mcp.log').write_text(p.stdout+p.stderr,encoding='utf-8')
    assert p.returncode==0,(p.returncode,p.stderr)
    responses={r['id']:r for r in map(json.loads,p.stdout.splitlines())}
    error=responses[4]['result']
    assert error['isError'] is True
    text=error['content'][0]['text']
    assert 'unknown variant `'+op+'`' in text,text
    assert responses[3]['result']==responses[5]['result'], 'failed open changed the existing model'
    after=json.loads(Path('baseline18-after-refused-'+op+'.bimcraft').read_text(encoding='utf-8'))
    assert after['format']==18
    rows.append(dict(operation=op,fixture=fixture_path.name,fixture_format=fixture['format'],
                     error=text,model_unchanged=True,saved_after_failure_format=after['format']))
print(json.dumps(dict(reader='baseline18-reader.exe',reader_sha256=reader_hash,
    source_delivery_base='30527e323e9c2e59abff927e4b76fe3ce0f83e15',
    reader_build_source_head='533073c5663a40038b0c9f6e2a168110734d56a2',
    source_core_cli_diff_vs_delivery_base='empty (verified by baseline build owner)',
    previous_portable_probe_correction='Initial files named format18 actually inherited portable format17. These corrected format18 probes use the unchanged-base format18 reader.',
    probes=rows),indent=2))
