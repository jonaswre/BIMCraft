import json
import subprocess
import sys
result = subprocess.run([sys.argv[1], "run", "--cmd", 'stair {"start":[0,0],"height":2880,"open":true}', "--cmd", 'elements {"category":"stair"}'], text=True, capture_output=True, check=True)
print(result.stdout)
stair = json.loads(result.stdout.splitlines()[1])["elements"][0]
assert stair.get("open") is True, "open:true must create open construction, not silently create a solid stair"
print("PASS: CLI creates open construction")
