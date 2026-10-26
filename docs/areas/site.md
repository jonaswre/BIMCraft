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
