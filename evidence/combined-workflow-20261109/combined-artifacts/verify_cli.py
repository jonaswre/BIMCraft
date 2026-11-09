"""Run synthetic combined CLI/native/IFC journeys; arguments are CLI, steel19 CLI, stair19 CLI."""
import hashlib,json,pathlib,subprocess,sys,math
root=pathlib.Path(__file__).resolve().parent
cli,steel19,stair19=sys.argv[1:4]
def run(exe,*args,ok=True):
 p=subprocess.run([exe,*args],cwd=root,text=True,encoding='utf-8',capture_output=True)
 print('COMMAND',*args);print(p.stdout,end='');print(p.stderr,end='')
 if ok:assert p.returncode==0,(p.returncode,p.stderr)
 return p
run(cli,'run','--script','mixed.txt','--save','mixed.bimcraft','--save','mixed.ifc','--save','mixed.glb',
 '--cmd','render {"view":"{3D}","path":"mixed.png","width":1000,"height":700}',
 '--cmd','render {"view":"Synthetic Section","path":"section.png","width":900,"height":600}',
 '--cmd','render {"view":"Synthetic South","path":"elevation.png","width":900,"height":600}',
 '--cmd','render {"view":"S-01","path":"sheet.pdf"}',
 '--cmd','render {"view":"S-01","path":"sheet.png"}',
 '--cmd','render {"view":"S-02","path":"sheet-3d.pdf"}',
 '--cmd','render {"view":"S-02","path":"sheet-3d.png"}')
a=json.loads((root/'mixed.bimcraft').read_text(encoding='utf-8-sig'));assert a['format']==20
assert any(x.get('open') is True for x in a['elements'].values())
for ext in ['bimcraft','ifc']:
 run(cli,'run',f'mixed.{ext}','--save',f'reopened-{ext}.bimcraft','--save',f'reopened-{ext}.ifc')
 b=json.loads((root/f'reopened-{ext}.bimcraft').read_text(encoding='utf-8-sig'))
 if ext=='bimcraft':
  for k in ['elements','types','library','georeference']:assert a[k]==b[k],k
 else:
  for cat in ['beam','structuralcolumn','door','window','stair','propertyline','wall']:
   x={e['guid']:e for e in a['elements'].values() if e['category']==cat}
   y={e['guid']:e for e in b['elements'].values() if e['category']==cat}
   assert set(x)==set(y),(cat,set(x)-set(y))
   if cat=='stair':assert all(e['open'] is True for e in y.values())
  for k in ['crs','eastings','northings','height','rotation','scale']:
   assert (a['georeference'][k]==b['georeference'][k] if k=='crs' else math.isclose(a['georeference'][k],b['georeference'][k],rel_tol=1e-12,abs_tol=1e-9)),(k,a['georeference'],b['georeference'])
# Preserve the exact independently authored format19 fixtures, not relabelled combined documents.
for dest,old in [('steel19.bimcraft',steel19),('stair19.bimcraft',stair19)]:
 raw=(root/dest).read_bytes()
 original=json.loads(raw);assert original['format']==19
 run(old,'run',dest,'--save',dest+'.old.ifc')
 run(cli,'run',dest,'--save',dest+'.new.bimcraft','--save',dest+'.new.ifc')
 got=json.loads((root/(dest+'.new.bimcraft')).read_text(encoding='utf-8-sig'))
 assert got['format']==20
 for k in ['elements','types','library']:
  # Added default-false stair field is the only permissible absent-field enrichment.
  def normalize(v):
   if isinstance(v,dict):return {k:normalize(x) for k,x in v.items() if not(k=='open' and x is False)}
   if isinstance(v,list):return [normalize(x) for x in v]
   return v
  assert normalize(got[k])==normalize(original[k]),(dest,k)
 refusal=run(old,'run','mixed.bimcraft',ok=False)
 assert refusal.returncode!=0 and 'file format 20' in refusal.stdout+refusal.stderr
 print('READER_SHA256',pathlib.Path(old).name,hashlib.sha256(pathlib.Path(old).read_bytes()).hexdigest())
print('PASS combined native/IFC represented identities and two original19 fixtures/refusals')
