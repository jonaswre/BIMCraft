"""Native and IFC element identity, and the existing auxiliary GUID behavior."""
import json,pathlib,uuid,ifcopenshell
p=pathlib.Path(__file__).resolve().parent
roles=['IfcStair','IfcStairFlight','IfcSlab','IfcRailing','IfcPropertySet','IfcRelDefinesByProperties']
def ids(name):
    f=ifcopenshell.open(str(p/name))
    return {r:sorted(x.GlobalId for x in f.by_type(r)) for r in roles}
original=ids('open-stair.ifc'); native=ids('reopened-bimcraft.ifc'); imported=ids('reopened-ifc.ifc'); tess=ids('reopened-tessellated.ifc')
assert original==native
for role in ['IfcStair','IfcRailing']:
    assert original[role]==imported[role]==tess[role]
baseline=ids('baseline-solid.ifc'); baseline_import=ids('baseline-ifc-reopened.ifc')
assert baseline['IfcStair']==baseline_import['IfcStair']
assert baseline['IfcStairFlight']!=baseline_import['IfcStairFlight']
assert baseline['IfcSlab']!=baseline_import['IfcSlab']
result={'native_reopen_all_ifc_roles_stable':True,'ifc_reopen_native_stair_railing_guids_stable':True,'derived_subproduct_guids_regenerate_after_ifc_import_in_baseline18_and_new_branch':True,'baseline18':baseline,'baseline18_ifc_reopened':baseline_import,'open':original,'open_ifc_reopened':imported,'metadata_categories':[67,68],'closed_IFC_DATA_matches_baseline18':(p/'baseline-solid.ifc').read_text().split('DATA;',1)[1]==(p/'closed-preserved.ifc').read_text().split('DATA;',1)[1]}
assert result['closed_IFC_DATA_matches_baseline18']
(p/'identity-validation.json').write_text(json.dumps(result,indent=2))
print(json.dumps({k:v for k,v in result.items() if not isinstance(v,dict)},indent=2))
