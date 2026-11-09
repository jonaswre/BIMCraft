### Before you send

- [x] I looked through the open and closed issues and did not find this one.
- [x] I am not attaching Autodesk files (`.rvt`, `.rfa`) or anything copied from Autodesk's products. BIMCraft is clean-room.

### What happened, and what did you expect?

Feature request: allow sectional overhead gates and roller shutters as door operations. The existing operation model supports swing, double-swing, sliding, double-sliding, folding and fixed, but refuses `sectional` and `roller`. Industrial gates should retain their operation through commands, native save/open, schedules and IFC, and their plan symbol should not imply a horizontal swing.

### Steps to reproduce

1. Start an empty project in the CLI.
2. Run `wall {"points":[[0,0],[12000,0]],"height":4500}`.
3. Run `type {"category":"door","name":"Sectional gate","width":3000,"height":3200,"operation":"sectional"}`.
4. Repeat with `operation: "roller"`.

### Where does it happen?

Command line or MCP server (bimcraft-cli). Also relevant to the existing Family Editor Operation selector, plan output and IFC import/export.

### BIMCraft version

Official Windows portable 0.11.0, reproduced 2026-11-09. Implementation target: upstream main `30527e323e9c2e59abff927e4b76fe3ce0f83e15`; source inspection confirms the same six-variant operation enum and parser.

### Operating system

Windows 11 Enterprise

### Mouse, trackpad or pen

Not applicable (CLI reproduction).

### Display and graphics

Not applicable (parameter rejection before drawing).

### A project or commands that show it

The commands above are entirely synthetic. No customer model or Autodesk files are involved. Searched live full bodies of open/closed issues and open PRs for sectional, roller and door/gate operations; no matching issue or open PR was found (the existing general door operations work is already merged).

### Error messages

```text
line 2 (type): type: `operation` is one of swing, double-swing, sliding, double-sliding, folding, fixed, not `sectional`
line 2 (type): type: `operation` is one of swing, double-swing, sliding, double-sliding, folding, fixed, not `roller`
```
