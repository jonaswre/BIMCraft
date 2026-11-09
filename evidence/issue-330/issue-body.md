### Before you send

- [x] I looked through the open and closed issues and did not find this one.
- [x] I am not attaching Autodesk files (`.rvt`, `.rfa`) or anything copied from Autodesk's products. BIMCraft is clean-room.

### What happened, and what did you expect?

Placing the default 3D view on a sheet without a scale uses 1 : 100 even when the model does not fit the paper. In the synthetic reproduction below, the viewport is 734 x 405 mm on a 420 x 297 mm A3 sheet. Its picture runs beyond all four sheet edges and hides the border and title block. The commands succeed without an error.

I expected an unscaled 3D view to use an appropriate offered project scale that fits the available paper above the title block. Explicit viewport scales and a view/template's assigned scale should remain under the user's control.

Observed using the official Windows portable CLI and inspecting its generated PNG. Source inspection at `30527e323e9c2e59abff927e4b76fe3ce0f83e15` shows `place` unconditionally uses `graphics::scale_of` when scale is omitted; that defaults to 100. The source explanation is separate from the released-binary observation.

Related #325 concerns overlapping viewport placement and titles, rather than choosing a scale for a large unscaled 3D view. I searched full bodies of current open/closed issues and open PRs before filing.

### Steps to reproduce

1. Save the commands below as `repro.txt`.
2. Run `bimcraft-cli run --script repro.txt --save before.bimcraft`.
3. Inspect `before.png`: only part of the building is visible and it covers the sheet border/title block.
4. Reopen `before.bimcraft` and render A101 again; the oversized scale is retained.

### Where does it happen?

Command line or MCP server (bimcraft-cli); Plans, sections, sheets, schedules or PDF output.

### BIMCraft version

Runtime: official Windows portable `bimcraft-cli 0.11.0`. Source inspected: upstream main `30527e323e9c2e59abff927e4b76fe3ce0f83e15`.

### Operating system

Windows 11 Enterprise (10.0.26300).

### Mouse, trackpad or pen

Not applicable; CLI commands.

### Display and graphics

Headless software-rendered PNG, 2100 x 1485 pixels. No desktop display or GPU involved.

### A project or commands that show it

Synthetic geometry only:

```text
wall {"points": [[0,0],[60000,0],[60000,40000],[0,40000]], "closed": true}
sheet {"number": "A101", "name": "Synthetic 3D", "size": "A3"}
place {"sheet": "A101", "view": "{3D}"}
elements {"category": "viewport"}
render {"view": "A101", "path": "before.png"}
```

### Error messages

No command error. The viewport query returns (GUIDs omitted):

```json
{"at":[215.0,168.5],"category":"viewport","scale":100.0,"size":[734.0,405.0],"title":"{3D}"}
```
