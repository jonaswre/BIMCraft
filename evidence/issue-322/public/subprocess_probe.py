from pathlib import Path
import os, json, subprocess, hashlib, sys
art = Path(__file__).resolve().parent
exe = Path(os.environ.get('BIMCRAFT_CLI', 'bimcraft-cli'))
env = dict(os.environ, BIMCRAFT_PLUGINS=str(art/'no-plugins'))
good='''@walls = wall {"points":[[0,0],[8000,0],[8000,6000]]}
door {"wall":"@walls.ids.1","offset":3000}
modify {"id":"$last","mark":"Entry"}'''
bad='''@walls = wall {"points":[[0,0],[8000,0]]}
door {"wall":"@walls.ids.99","offset":3000}'''

def cli(*args):
    r=subprocess.run([str(exe),*map(str,args)],capture_output=True,text=True,encoding='utf-8',env=env,cwd=art,timeout=60)
    assert r.returncode==0,(r.returncode,r.stdout,r.stderr)
    return [json.loads(l) for l in r.stdout.splitlines() if l]

class MCP:
    def __init__(self,*args):
        self.p=subprocess.Popen([str(exe),'mcp',*map(str,args)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,encoding='utf-8',env=env,cwd=art)
        self.i=0
        self.transcript=[]
    def tool(self,name,args={},error=False):
        self.i+=1
        request=dict(jsonrpc='2.0',id=self.i,method='tools/call',params=dict(name=name,arguments=args))
        self.p.stdin.write(json.dumps(request)+'\n');self.p.stdin.flush()
        response=json.loads(self.p.stdout.readline())
        self.transcript.append(dict(request=request,response=response))
        result=response['result']
        assert bool(result.get('isError'))==error,response
        text=result['content'][0]['text']
        return text if error else json.loads(text)
    def close(self,name):
        self.p.stdin.close()
        self.p.wait(timeout=30)
        assert self.p.returncode==0,self.p.stderr.read()
        (art/name).write_text(json.dumps(self.transcript,indent=2),encoding='utf-8')

script=art/'references.txt';script.write_text(good,encoding='utf-8')
model=Path('references.bimcraft')
made=cli('run','--script',script,'--save',model)
query=cli('run',model,'--cmd','elements {"category":"doors"}')[-1]
assert query['count']==1 and query['elements'][0]['mark']=='Entry'
assert query['elements'][0]['id']==made[1]['id']
(art/'cli-output.json').write_text(json.dumps(dict(made=made,reopened=query),indent=2),encoding='utf-8')
cli('run',model,'--cmd','render '+json.dumps(dict(path='references.png')))

m=MCP()
error=m.tool('script',dict(text=bad,atomic=True),error=True)
assert 'line 2 (door)' in error,error
assert 'wall' not in m.tool('inspect_model')['counts']
results=m.tool('script',dict(text=good,atomic=True))['results']
assert m.tool('query_elements',dict(category='doors'))['elements'][0]['mark']=='Entry'
saved=Path('mcp-references.bimcraft')
m.tool('save',dict(path=str(saved)))
m.tool('execute',dict(command='undo'))
assert 'wall' not in m.tool('inspect_model')['counts']
m.tool('open',dict(path=str(saved)))
reopened=m.tool('query_elements',dict(category='doors'))
assert reopened['elements'][0]['id']==results[1]['id']
m.close('mcp-transcript.json')

before=hashlib.sha256((art/saved).read_bytes()).hexdigest()
r=MCP('--read-only',saved)
e=r.tool('script',dict(text='@walls = wall {"points":[[0,0],[8000,0]]}',atomic=True),error=True)
assert 'read-only' in e,e
e=r.tool('script',dict(text='@file = save '+json.dumps(dict(path=str(saved))),atomic=True),error=True)
assert 'read-only' in e,e
query_script='@walls = elements {"category":"walls"}\nelements {"level":"@walls.elements.0.level","category":"walls"}'
assert r.tool('script',dict(text=query_script,atomic=True))['results'][1]['count']==2
r.close('mcp-read-only-transcript.json')
assert hashlib.sha256((art/saved).read_bytes()).hexdigest()==before
(art/'subprocess-results.json').write_text(json.dumps(dict(version='0.11.0',cli_hosted_door=True,mcp_atomic_rollback_retry_save_reopen=True,mcp_read_only_edit_and_own_file_refused=True,read_only_file_sha256=before),indent=2),encoding='utf-8')
print('PASS: actual CLI script/save/reopen/render; MCP atomic rollback/retry/save/reopen/undo; read-only references and own-file protection')
