"""Strict diagnostic: same-identity represented product geometry through IFC import. Expected to fail on current room height loss."""
import json,pathlib
import numpy as np
root=pathlib.Path(__file__).resolve().parent
report=json.loads((root/'ifc-validation.json').read_text())
a,b=report['mixed.ifc']['shapes'],report['reopened-ifc.ifc']['shapes']
errors=[]
for guid in sorted(set(a)&set(b)):
 for key in ['min','max','volume']:
  if not np.allclose(a[guid][key],b[guid][key],rtol=2e-5,atol=1e-5):errors.append({'guid':guid,'class':a[guid]['class'],'field':key,'before':a[guid][key],'after':b[guid][key]})
(root/'strict-ifc-roundtrip-failures.json').write_text(json.dumps(errors,indent=2),encoding='utf-8')
print(json.dumps(errors,indent=2))
assert not errors, 'Strict full-model IFC roundtrip FAILED; retained as a real failure, not an expected-pass test'
