# Levels, walls, roofs, floors, ceilings, shafts, stairs

Moved from the root `CLAUDE.md`, word for word: the root keeps the rules and the map, and this file is read when the area is touched.

Levels have elevations, and each is a plane over its `extent` (`Model::level_extent`; without one, `Model::model_extent`):
sections and elevations draw a level over as much of its extent as they see, and the 3D view outlines it, depth-tested and
picked as the level; unselected, its two edges nearer the camera are left out. Its name goes on a `view3d::LevelHead`, an
unseen vertical plane past the right end of its far edges, where an elevation marker family will stand. Selected alone, a
level's ends in a section are grips (`Model::level_extent_along`) and so are its plane's edge middles in 3D
(`geom::edge_grips`, `Camera::on_plane`); both run `modify {extent}`.
A section or an elevation cropped shows its line's length and its crop's heights; uncropped, everything along its line
(`render::crop_region` is none, and there are no grips).
A level is also its plan: its view range (`Model::plan_range`)
cuts 1200 mm above it unless set, and its crop region bounds what the plan shows. Rooms find their walls at 1200 mm whatever
a plan cuts (`Model::bounding`), and those walls' centre lines run on to meet only each other's (`walls::centre_line`): a wall
raised above that height (a high side over a lower roof) neither bounds a room nor breaks one where it meets, higher up, a wall
that does. In a section or an elevation, points run along its line from its start, and up; on a sheet,
they're paper mm from its bottom left corner; notes and spot elevations live in those terms in their view.
A door or window put `at` a point goes in the wall its level's plan cuts there (`wall_at`); with no `level`, the lowest level
at or above ground (elevation 0) with one there, else the highest below ground with one (`wall_on_any_level`), so a
foundation level under the ground floor isn't where it looks first. A `level` given is the only one it looks on.
A wall's `start` and `end` are on its location line: its centre line unless `location` says otherwise (`walls::faces`).
IFC keeps it: a wall's axis is its location line, and the `BIMCraft_Wall` property set names the line (`Location`) and the
GlobalIds of what its top and base are attached to (`AttachTop`, `AttachBase`); import puts such a wall back on that line with
its ends as drawn, and a wall without the set on the middle of its body. A door or window placed `at` a point goes in the wall
a plan of the level cuts there, else in any wall of the level there, whatever its height and base offset; openings in a wall
may share a place along it, never a place in its elevation (along it and up it), so a window may stand over a door.
Its exterior is to the left of start to end, as in Revit, so walls drawn clockwise face out; a compound type lists its
layers from the exterior in, and a floor, roof or ceiling type from the top down (IFC import turns a wall round, or a slab's
layers over, by the way its layer set's usage stacks them). A layer's material is a name in the project's library
(`Model::materials`): its colour shades the layer in 3D, its cut pattern is a drafting pattern (`GAP` px apart on screen and
paper, where the layer is `PATTERN_ROOM` px wide), its surface pattern a model pattern (`MODULE` mm, its real size). A curved wall turns `arc` degrees, counter-clockwise positive (`walls::Axis`). Where walls meet,
their faces meet in mitred caps at any angle, and a wall ending on another stops at its face (`walls::cap`); plans, 3D,
sections and IFC all draw that outline (`walls::outline`, `walls::prisms`). Room areas are measured at the walls' centre
lines, run on or back to meet at joins (`walls::centre_line`). An attached top or base follows the roof's or floor's plane
over each part of the wall (a roof's slopes each over its own part of the plan); a mirrored wall runs back the other way.
A roof by footprint is the straight skeleton of its outline and its `courtyards` (`skeleton::faces`: the outline
counter-clockwise, the courtyards clockwise, the roof to the left of every edge): each edge moves in at speed 1, a gable's at
0, and its slope rises from it, reaching the roof's base `overhang` in from its eaves; a courtyard's edges number on after
the outline's, and every ring keeps its numbers whichever way round it runs, so `gables` stay put through a mirror.
A `boundary` given to `roof` is drawn where the walls are, as Revit's footprint over walls: with an `overhang` the eaves
(the roof's stored `boundary`) are moved that far out from it, and into each given courtyard, its points kept in their order
so `gables` keep their edges (`commands::eaves`), and the base at `offset` is at the line drawn; a ring too small for the
overhang is refused. `roof_by_face` and `update_by_face` give a mass's top face its eaves the same way.
Over the walls (`floor_outline`, which `roof_footprint` grows by the overhang), a loop of them standing free inside the outer
ones, joined to none of them and with no room in it, is a courtyard, open through the floor and the roof; one joined to them,
or with a room in it, is rooms, and both run over it.
`Model::roof_faces` are its slopes, each the loops round it in plan (a slope can close round a courtyard whose far side is a
gable) with the plane of its underside, trimmed where it `join`s another; `Model::roof_pieces` are those less their holes
and its `openings`, convex where cut. A floor's top is at its level plus `offset`,
or with a slope arrow, there at the arrow's tail and rising `slope` degrees toward its head (`Model::floor_top`); it's its
type's thickness across the slope, so plumb it's thicker. Its `openings` are holes inside it, apart, and `Model::floor_slabs`
what's left, convex where cut. A ceiling's `offset` is its underside; its `openings` are holes as a floor's
are, and `Model::ceiling_slabs` what's left. A shaft cuts each floor,
ceiling and roof its span reaches into over its outline, all the way through (`Model::less_shafts`). A stair or a ramp
is runs and landings laid out from its start (`stairs::stair`, `stairs::ramp`): a landing after `at` risers (a ramp's, `at`
mm up), as deep as the runs are wide, the next run turning by its `turn`; a railing on one follows its walk line moved out to
its side (`stairs::railing`), so it goes wherever its host goes. `modify` merges into fields that are objects (JSON Merge Patch), so `{"view_range": {"cut": 1500}}` keeps the rest; an element's own field can't be
called `category`, the tag every element carries.
