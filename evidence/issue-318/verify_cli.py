import json, subprocess, pathlib, hashlib, sys
from PIL import Image, ImageChops, ImageDraw
if len(sys.argv) not in (2, 3):
 raise SystemExit('Usage: python verify_cli.py <bimcraft-cli executable> [output directory]')
cli=pathlib.Path(sys.argv[1]).resolve()
out=pathlib.Path(sys.argv[2]) if len(sys.argv)==3 else pathlib.Path('glyph-output')
out.mkdir(parents=True,exist_ok=True)
def call(key,args):
 p=subprocess.run([str(cli),*map(str,args)],capture_output=True,encoding='utf-8')
 (out/(key+'.stdout')).write_text(p.stdout,encoding='utf-8')
 (out/(key+'.stderr')).write_text(p.stderr,encoding='utf-8')
 assert p.returncode==0,(key,p.returncode,p.stdout,p.stderr)
 return [json.loads(l) for l in p.stdout.splitlines() if l.strip()]
print(subprocess.run([str(cli),'--version'],capture_output=True,text=True).stdout.strip())
for key,label in [('prime','1′'),('double-prime','1″'),('ellipsis','1…'),('question','1?'),('unsupported','1漢')]:
 path=out/(key+'.png'); native=out/(key+'.bimcraft'); script=out/(key+'.txt')
 script.write_text('grid '+json.dumps(dict(start=[0,0],end=[0,5000],name=label),ensure_ascii=False)+'\nrender '+json.dumps(dict(view='Level 1',width=640,height=480,path=str(path)))+'\n',encoding='utf-8')
 result=call(key,['run','--script',script,'--save',native])
 render=result[1]
 if key=='unsupported':
  assert len(render['notes'])==1 and 'U+6F22' in render['notes'][0],render
 else: assert 'notes' not in render,render
 reopened=out/(key+'-reopened.png')
 call(key+'-reopened',['run',native,'--cmd','render '+json.dumps(dict(view='Level 1',width=640,height=480,path=str(reopened)))])
 assert path.read_bytes()==reopened.read_bytes(),key
 print(key,'native roundtrip PNG identical; notes:',render.get('notes',[]),'sha256:',hashlib.sha256(path.read_bytes()).hexdigest())
question=Image.open(out/'question.png').convert('RGB')
for key in ['prime','double-prime','ellipsis']:
 diff=ImageChops.difference(Image.open(out/(key+'.png')).convert('RGB'),question)
 assert diff.getbbox() is not None,key
 print(key,'different RGB pixel bbox:',diff.getbbox())
assert Image.open(out/'unsupported.png').convert('RGB').tobytes()==question.tobytes()
# Enlarge actual exported head pixels without interpolation for visual review.
canvas=Image.new('RGB',(1000,210),'white'); draw=ImageDraw.Draw(canvas)
for i,(key,title) in enumerate([('prime','Prime'),('double-prime','Double prime'),('ellipsis','Ellipsis'),('question','Question mark'),('unsupported','Unsupported')]):
 im=Image.open(out/(key+'.png')).convert('RGB').crop((300,52,340,92)).resize((160,160),Image.Resampling.NEAREST)
 canvas.paste(im,(i*200+20,36)); draw.text((i*200+20,10),title,fill='black')
canvas.save(out/'drawing-heads.png')
