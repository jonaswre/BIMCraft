# The site: property lines and topography

Read when the site's elements are touched: `PropertyLine` and `Topography` in `model.rs`, their commands in `commands.rs`, and
`site.rs` (the triangulation, contours, ground lines, edges and bearings). Tests: `site_tests.rs` (through the commands) and
`site::tests`.

## Property lines

A property line (`property_line`, Revit's Property Line) is a plot's boundary: a closed outline of points (its last point not
its first again; a ring given closed loses the repeat) with a `name` (Property Line 1, 2… by default, in the project's
language) on a level, the lowest by default. `Model::check` holds it to a boundary's rules (3 to `MAX_POINTS` points around
some area). Only its level's plan draws it, the site plan: the dash-dot line of a site plan (`render::DASH_DOT`, a long dash,
a gap, a dot, a gap, in pixels, cut into separate strokes so SVG, PNG and PDF all draw it alike; past 20,000 dashes on screen
one solid line), in Object Styles' `propertyline` pen (MEDIUM), with its name and its area at its centroid
(`site::centroid`); the plan frames it. `elements` gives `area_m2`, `perimeter_m` and `segments`, each segment's `length`,
its `bearing` as a survey writes it, from north or south toward east or west in degrees, minutes and seconds (`site::bearing`,
due east `N 90°00'00" E`, due south `S 0°00'00" E`, due west `S 90°00'00" W`) and its `azimuth` (degrees clockwise from
north). A schedule of property lines (`OWN`'s `propertyline`) lists Name, Area and Perimeter (Level and Count available).

In IFC the property lines are the site's footprint: IfcSite's Representation is a `FootPrint` `Curve2D` shape (in a
FootPrint subcontext, PLAN_VIEW) with one closed IfcPolyline per property line, `Pset_SiteCommon.TotalArea` their area in m²,
and `BIMCraft_PropertyLines.Names` (an IfcPropertyListValue of labels) their names in the curves' order. Import reads each
FootPrint curve of an IfcSite back as a property line on the lowest level, named from that list (else Property Line n).

## Topography

A terrain (`topography`, Revit's Toposurface) is a `Topography`: its points in the project's coordinates (mm; `z` an
elevation, as levels' are), a name (Topography 1, 2… by default), an optional library `material` and a contour interval
(`contours`, 1000 mm unless given, 0 for none), on a level (the lowest by default) whose plan draws it. Up to
`MAX_TERRAIN_POINTS` (250,000) points each and `MAX_TERRAIN_TOTAL` (1,000,000) in a project; `Model::check` refuses points
past `MAX_COORD`, points all on one line (`site::spread`), an unknown material and an interval under 10 mm. The points come
inline (`points: [[x, y, z], …]`, mm) or from a file (`path`): what `point_cloud` reads (`cloud::read`: XYZ/TXT, PTS, PLY,
LAS, E57) and CSV (a header row skipped), in `units` (m by default). Placed as a scan is: the middle of its points on `at`
and its lowest point `offset` over its level (`origin: centre`, the default), the file's own coordinates (`origin: file`),
or a file point (in the file's units) on the project's origin, heights measured from it (`origin: [x, y, z]`); a 1 m DGM in
UTM with NHN heights comes in with `origin: [E, N, H]` of the project's base point. `spacing` keeps one point in each square
that wide (a 2D grid, unlike a scan's cubes), and one point stays at each place in plan.

Its triangles aren't stored: `site::triangles` makes them from the points (`site::delaunay`, an S-hull sweep with Lawson
flips, written here, no dependency) and keeps the last eight terrains' (keyed by their points), so a `modify` of `points` is
always in step. They cover the points' convex hull. A 27,750-point DGM (185 × 150 m at 1 m) triangulates in about 0.1 s and
its plan draws in about 30 ms (`site_tests`' timing test prints them). In 3D it is a `view3d::Solid::Ground`: its surface,
shaded smoothly in its material's colour (else a ground green), picked as the terrain. A section cuts it with
`site::profile` (where the section's plane slices the triangles, `site::slice`), and fills the ground below each ground line
down to `view3d::GROUND_DEPTH` (1 m) under its lowest point, in poché like any cut. Its level's plan draws it under
everything else: its edge (`site::edge`, the edges only one triangle has) in its category's pen and its contour lines
(`site::contours`), every fifth heavier (`Pen::PROJECTION`) and the rest fine and faint, at most `render::MAX_CONTOURS`
heights (past that the interval doubles). Spot elevations on it read its height (`view3d::tops_at`).

A big terrain doesn't inflate what the rest is framed by: `Model::model_extent` (the levels' default extent) and a section's
span along its line leave terrains out, and a plan frames a terrain only when it shows nothing else; the 3D view fits all
of it, as Revit's default 3D view does. `elements` gives `point_count`, `triangles`, `area_m2` (in plan) and `elevations`
(lowest, highest); a schedule of topography lists Name, Level, Points and Plan Area.

IFC: an IfcGeographicElement, PredefinedType TERRAIN, contained in the IfcSite (not a storey), placed at the site's origin,
its Body an open IfcTriangulatedFaceSet (`Closed` .F.) over an IfcCartesianPointList3D of all its points, styled and
associated with its material if it has one. Import reads every IfcGeographicElement that is TERRAIN (or says nothing of
what it is), and an IfcSite with a Body (as IFC2x3 files carry the terrain), as a terrain on the lowest level: the corners
of its surface, one per place in plan, triangulated again.
