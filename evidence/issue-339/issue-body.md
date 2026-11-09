### Before you send

- [x] I looked through the open and closed issues and did not find this one.
- [x] I am not attaching Autodesk files (`.rvt`, `.rfa`) or anything copied from Autodesk's products. BIMCraft is clean-room.

### What happened, and what did you expect?

A window placed on an explicitly requested upper level in a full-height wall is accepted, but its sill stays at the lower wall base. For a level at +3400 mm and a 900 mm sill, the resulting window starts at +900 mm instead of +4300 mm. The upper plan has no window to pick there; the elevation, 3D geometry, IFC and reopened native project keep the lower placement.

Expected contract: an explicitly supplied level different from the host wall's level sets the opening's absolute placement to that level's elevation plus its sill (door: level elevation). Keep the native host-relative sill representation, existing host validation/rollback and omitted/same-host-level behavior, including raised host base offsets. Resolve and validate a supplied level even with an explicit wall.

Related closed #201 described upper-floor placement symptoms; its follow-up #212 improved off-cut wall diagnostics. This reproduction finds and accepts the full-height wall, then places the opening at the wrong elevation. #215 concerns heads above wall tops. Open #316 / PR #320 concern omitted-level host lookup, and #335 / PR #338 concern raised-wall host lookup. Their scopes differ; no exact requested-level elevation report was found in current full issue bodies and open PR bodies.

### Steps to reproduce

1. Start a fresh CLI session and run the synthetic commands below.
2. The wall spans 0–7200 mm and the requested Upper plan cuts it at 4600 mm, so host lookup succeeds.
3. Save the project as `red.bimcraft` and `red.ifc`, then reopen it.
4. Observe that the window is at +900 mm in the elevation/3D and independent IFC geometry, rather than +4300 mm.
5. Replacing point lookup with the returned wall id and `level: "Upper"` also reproduces it. A door similarly starts at 0 instead of +3400 mm.

### Where does it happen?

Command line or MCP server (bimcraft-cli); shared desktop Door/Window placement builder; exported geometry.

### BIMCraft version

Source `30527e323e9c2e59abff927e4b76fe3ce0f83e15` (0.11.0), standalone CLI built with `--no-default-features` and local core opt-level=0/debug=0 overrides. Those omit optional external readers only. Independent IFC inspection: IfcOpenShell 0.8.5.

### Operating system

Windows 11 (10.0.26300).

### Mouse, trackpad or pen

Not applicable to the command reproduction.

### Display and graphics

Not applicable to the geometry reproduction.

### A project or commands that show it

Original synthetic geometry only; save as `repro.txt`:

```text
level {"name":"Upper","elevation":3400}
wall {"points":[[0,0],[8000,0]],"level":"Level 1","height":7200}
window {"at":[4000,0],"level":"Upper","sill":900}
elements {"category":"window"}
section {"start":[-1000,-500],"end":[9000,-500]}
```

```text
bimcraft-cli run --script repro.txt --cmd 'render {"view":"Upper","path":"upper.png"}' --cmd 'render {"view":"Section 1","path":"elevation.png"}' --save red.bimcraft --save red.ifc
bimcraft-cli run red.bimcraft --cmd 'elements {"category":"window"}' --save red-reopened.ifc
```

Observed independent IFC world geometry: `IfcWindow` bounding-box Z minimum 0.9 m and maximum 2.4 m in both files. Expected 4.3–5.8 m. Both IFC files validate with zero statements, so syntactic validity does not catch the wrong position.

### Error messages

No command error: placement, saves and reopen all exit 0. A public Session regression run against unmodified production source reports 1 compatibility control passed and 3 behavior tests failed:

```text
requested level + sill, not host base + sill
left: 900.0
right: 4300.0

explicit host door
left: 0.0
right: 3400.0

requested level outside host: expected rejection, command returned an opening
```
