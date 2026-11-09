import ast
import json
from pathlib import Path
import subprocess
import sys

repo = Path(sys.argv[1])
def git(*args):
    return subprocess.check_output(['git', '-C', str(repo), *args], text=True, encoding='utf-8')
def messages(text):
    out = {}
    fields = {}
    active = None
    def flush():
        if 'msgid' in fields:
            out[(fields.get('msgctxt',''), fields['msgid'])] = {k:v for k,v in fields.items() if k.startswith('msgstr')}
    for line in text.splitlines()+['']:
        if not line.strip():
            flush()
            fields = {}
            active = None
        elif line.startswith('msg'):
            key, value = line.split(' ',1)
            active = key
            fields[key] = ast.literal_eval(value)
        elif line.startswith('"') and active:
            fields[active] += ast.literal_eval(line)
    return out
rows=[]
for name in git('diff','--name-only','HEAD','--','translations').splitlines():
    old=messages(git('show','HEAD:'+name))
    new=messages((repo/name).read_text(encoding='utf-8'))
    changed=[key for key in old if key not in new or new[key] != old[key]]
    assert not changed, (name,changed)
    added=[key[1] for key in new if key not in old]
    assert set(added) <= {'Sectional','Roller'}, (name,added)
    rows.append(dict(file=name,preserved_messages=len(old),added=added))
print(json.dumps(dict(catalogs=rows,existing_translations_preserved=True),indent=2))
