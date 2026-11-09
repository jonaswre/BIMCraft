import json, sys
import ifcopenshell
from ifcopenshell.util.element import get_psets
m=ifcopenshell.open(sys.argv[1])
print(json.dumps([{"class":e.is_a(),"tag":e.Tag,"quantities":get_psets(e,qtos_only=True)} for e in m.by_type('IfcElement')],indent=2))
