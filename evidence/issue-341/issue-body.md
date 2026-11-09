### Before you send

- [x] I looked through the open and closed issues and did not find this one.
- [x] I am not attaching Autodesk files (`.rvt`, `.rfa`) or anything copied from Autodesk's products. BIMCraft is clean-room.

### What happened, and what did you expect?

A room separator continuing along the finish face of an offset wall ends at its end cap, parallel to the wall centreline. At the default centreline area setting the boundary fails to join: South takes the whole 61.44 m² enclosure and North reports `shares_space_with` South and zero area. At the finish-face setting both rooms are correctly enclosed (28.32 and 29.14 m²).

Expected: both rooms remain independently enclosed at either area setting, with appropriate gross and net quantities. The separator should remain at its drawn coordinates.

Related closed #149 and merged #152 cover separators crossing a wall face. This is the remaining parallel wall-end case: extending that separator forward never intersects the wall centreline. It is separate from raised-wall issue #312 / PR #317.

### Steps to reproduce

1. Start an empty CLI project.
2. Create the rectangular enclosure and partial offset partition below.
3. Continue its finish face with a room separator to the opposite wall's inside face.
4. Place South and North, then compare the room output at centre and finish.

### Where does it happen?

Command line or MCP server (bimcraft-cli)

### BIMCraft version

Official Windows portable `bimcraft-cli 0.11.0`, reproduced 2026-11-09. Source investigation base: upstream main `30527e323e9c2e59abff927e4b76fe3ce0f83e15`. Source regression results will follow separately.

### Operating system

Windows 11 (build 26300)

### Mouse, trackpad or pen

Not applicable (CLI).

### Display and graphics

Not applicable (CLI reproduction).

### A project or commands that show it

Save as `repro.txt`, then run `bimcraft-cli run --script repro.txt`:

```text
wall {"points":[[0,0],[9600,0],[9600,6400],[0,6400]],"closed":true}
wall {"points":[[0,3200],[4200,3200]],"location":"exterior-finish"}
room_separator {"points":[[4200,3200],[9500,3200]]}
room {"point":[2000,1600],"name":"South"}
room {"point":[2000,4800],"name":"North"}
elements {"category":"room"}
room_area {"at":"finish"}
elements {"category":"room"}
```

Entirely synthetic geometry. Searched all current open/closed issue bodies and open PR bodies for separator, room boundary/enclosure, finish face and offset wall matches before filing.

### Error messages

No command error. North is placed with this warning at the default centreline setting:

```text
room 100008 is already in this space; two rooms can't share one, so this one is placed unbounded: move it, delete one of them, or part them with a room separation line
```

Observed room fields:

```text
centre: South area_m2=61.44 enclosed=true; North area_m2=0.0 enclosed=false shares_space_with=100008
finish: South area_m2=28.32 enclosed=true; North area_m2=29.14 enclosed=true
```

Source inference: `to_centre` only finds forward intersections with wall centrelines, so parallel continuations cannot reach their adjacent wall end.
