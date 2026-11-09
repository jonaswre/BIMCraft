"""Verify foreign IFC import through the real CLI. Usage: python verify_foreign.py CLI."""
import json, subprocess, sys
from pathlib import Path
out=Path(__file__).resolve().parent
cli=str(Path(sys.argv[1]).resolve())
r=subprocess.run([cli,'run','foreign-metre-rotated.ifc','--cmd','elements {"category":"property lines"}','--save','foreign-import.bimcraft'],cwd=out,capture_output=True,text=True,encoding='utf8')
(out/'foreign-import.log').write_text(r.stdout+r.stderr,encoding='utf8')
assert r.returncode==0,(r.stdout,r.stderr)
rows=[json.loads(x) for x in r.stdout.splitlines() if x.startswith('{')]
elements=next(x['elements'] for x in rows if 'returned' in x)
assert sorted(e['area_m2'] for e in elements)==[16.0,24.0]
assert all(e['perimeter_m']==20 for e in elements)
pts=[pt for e in elements for pt in e['points']]
assert [min(pt[0] for pt in pts),min(pt[1] for pt in pts),max(pt[0] for pt in pts),max(pt[1] for pt in pts)]==[6000,20000,10000,34000]
assert all(e['name'].startswith('Property Line ') for e in elements)
print('PASS: foreign IFC metre units, rotated/translated placement, default names, area/perimeter and imported world bounds.')
