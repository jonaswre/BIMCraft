# Raised-wall opening lookup

Synthetic evidence for [feature request #335](https://github.com/IPedrax/BIMCraft/issues/335), based on source commit `30527e323e9c2e59abff927e4b76fe3ce0f83e15`.

A wall assigned to Level 1 starts at 6000 mm and ends at 9000 mm. The plan cuts at 1200 mm. Point-based opening placement previously refused this wall; explicitly naming its id already worked. The enhancement retains cut-wall priority, then accepts a unique nearby same-level wall whose lowest base is above the cut. Multiple raised candidates require `wall`; low walls and walls belonging only to another level keep the existing diagnostic.

This is a policy enhancement following #201/#212, which intentionally improved the refusal diagnostic. It does not change opening sill elevation or plan visibility.

Run the complete synthetic CLI journey in this folder:

```text
bimcraft-cli run --script cli-journey.txt --save raised-wall.bimcraft
bimcraft-cli run raised-wall.bimcraft --script cli-reopen.txt
```

`before.png` shows the raised wall before openings; `after.png` shows the door and window; `reopened.png` is the rendering after a native save and fresh CLI reopen. The latter two images must match byte for byte. The native file contains only original synthetic geometry.

The public core test target `raised_wall_openings` covers creation/render/save/open, preview consistency, cut-wall priority, explicit-wall control, stacked ambiguity with rollback, low-wall/other-level diagnostics, and sill/width/overlap validation. See `validation.md` for captured results and limits.
