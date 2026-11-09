import json, subprocess, pathlib, hashlib, sys
from PIL import Image, ImageChops
if len(sys.argv) not in (2, 3):
 raise SystemExit('Usage: python repro.py <bimcraft-cli executable> [output directory]')
cli=pathlib.Path(sys.argv[1]).resolve()
out=pathlib.Path(sys.argv[2]) if len(sys.argv)==3 else pathlib.Path('glyph-baseline')
out.mkdir(parents=True,exist_ok=True)
print(subprocess.run([str(cli),'--version'],capture_output=True,text=True).stdout.strip())
for key,label in [('prime','1′'),('double-prime','1″'),('ellipsis','1…'),('question','1?'),('unsupported','1漢')]:
 script=out/(key+'.txt')
 script.write_text('grid '+json.dumps(dict(start=[0,0],end=[0,5000],name=label),ensure_ascii=False)+'\nrender '+json.dumps(dict(view='Level 1',width=640,height=480,path=str(out/(key+'.png'))))+'\n',encoding='utf-8')
 p=subprocess.run([str(cli),'run','--script',str(script)],capture_output=True,encoding='utf-8')
 (out/(key+'.stdout')).write_text(p.stdout,encoding='utf-8')
 (out/(key+'.stderr')).write_text(p.stderr,encoding='utf-8')
 print(key,'exit:',p.returncode,'reply:',p.stdout.strip(),'stderr:',p.stderr.strip())
 if p.returncode: raise SystemExit(p.returncode)
question=Image.open(out/'question.png').convert('RGB')
for key in ['prime','double-prime','ellipsis','unsupported']:
 im=Image.open(out/(key+'.png')).convert('RGB')
 print(key,'RGB pixel difference bbox:',ImageChops.difference(im,question).getbbox(),'sha256:',hashlib.sha256((out/(key+'.png')).read_bytes()).hexdigest())
