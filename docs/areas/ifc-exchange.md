# IFC beside the elements: grids, base quantities, georeference

What `ifc.rs` writes and `import.rs` reads besides the building elements themselves (IFC4 is ISO 16739; the entities and
their attributes below are its). The tests are `ifc_exchange_tests.rs` (`grids::`, `quantities::`, `georeference::`), which
save through `save` and read back through `open`, looking at the STEP file's entities and references. How to check a file
with IfcOpenShell: [looking at the result](looking-at-results.md).

## Grids

The project's grids are one `IfcGrid` (`Step::grid`), placed at the building's origin and contained in the building
(`IfcRelContainedInSpatialStructure`), since a BIMCraft grid runs through every storey. Each grid is an `IfcGridAxis` tagged
with its name (`AxisTag`), its line an `IfcPolyline` of its two ends in plan (an axis curve is 2D), `SameSense` true. The
grids along the first grid's direction (within 1e-3 of parallel) are the U axes, every other grid a V axis; `PredefinedType`
is `.RECTANGULAR.` when every V axis is square to the U axes, else `.IRREGULAR.`. IFC wants at least one axis in each of U
and V: of parallel grids the last is the V axis, and a lone grid is written in both. The `IfcGrid` has no representation of its own: viewers draw
its axes, and a curve-only `FootPrint` is a shape IfcOpenShell's geometry won't build.

Import reads every axis of every `IfcGrid` (U, V and W) as a grid: its tag the name (a taken name gets ` 2`…), its curve's
ends through the grid's placement, the other way round where `SameSense` is false. An axis that repeats another of the
same grid (same tag, same line either way round) comes in once, so a lone grid comes back alone.

## Base quantities

Each element's IFC4 base quantities go in an `IfcElementQuantity` named for its set (`MethodOfMeasurement`
`'BaseQuantities'`), related to it by an `IfcRelDefinesByProperties` (GlobalIds from its GUID, roles `QUANTITIES` and
`QUANTITY_DEFINES`). Lengths are in the file's millimetres, areas in m², volumes in m³ (the exporter's units); a value that
isn't a finite size is left out, and so is a set with nothing in it. `Step::quantities` writes them.

| Element | Set | Quantities |
|---|---|---|
| Wall | `Qto_WallBaseQuantities` | Length (its location line), Width (its type's thickness), Height (lowest base to highest top), GrossFootprintArea (its mitred outline), GrossSideArea and NetSideArea (its volumes over its thickness), GrossVolume and NetVolume (`walls::prisms` without and with its doors' and windows' holes, each prism's area times its height at its triangles, exact for flat tops and bottoms, so a wall attached to a roof measures right) |
| Floor | `Qto_SlabBaseQuantities` | Width (its type's thickness), Perimeter, GrossArea (its boundary) and NetArea (less its openings and the shafts through it), along its slope; GrossVolume, NetVolume |
| Ceiling | `Qto_CoveringBaseQuantities` | Width, GrossArea, NetArea |
| Room | `Qto_SpaceBaseQuantities` | Height (to the level above, as its body), GrossPerimeter, NetPerimeter, GrossFloorArea, NetFloorArea (its region, measured as BIMCraft measures a room: at the walls' centre lines, so net is gross), GrossVolume, NetVolume; a room that isn't enclosed has none |
| Column, structural column | `Qto_ColumnBaseQuantities` | Length, CrossSectionArea, GrossVolume, NetVolume |
| Footing | `Qto_FootingBaseQuantities` | Length, Width, Height, GrossVolume, NetVolume |
| Beam | `Qto_BeamBaseQuantities` | Length (along its axis), CrossSectionArea, GrossVolume, NetVolume |
| Strip footing | `Qto_FootingBaseQuantities` | Length, Width, Height, CrossSectionArea, GrossVolume, NetVolume |
| Door, window | `Qto_DoorBaseQuantities`, `Qto_WindowBaseQuantities` | Width, Height, Perimeter, Area (of its opening) |
| Roof | `Qto_RoofBaseQuantities` | GrossArea (its slopes along their pitch, round its courtyards), NetArea (less its openings), ProjectedArea (its outline in plan less its courtyards) |

Import ignores them: what an element measures comes from its geometry. Pipes, conduits, trays, stairs, ramps, railings,
masses and components have none yet.

## Georeference

Where the project stands on the earth is `Model::georeference` (`georef.rs`), set by the `location` command (Revit's
Manage > Location and its shared coordinates): the map (`crs`, an EPSG code or a name, with its geodetic and vertical datums,
projection and zone), where the model's origin lies on it (`eastings`, `northings`, `orthogonal_height`, in mm like every
length), how far the model's +y is turned clockwise from the map's grid north (`rotation`, or given as `x_axis`, the way its
+x runs on the map), true north (`true_north`, clockwise from it to the model's +y; grid north's when not given), and the
site's `latitude`, `longitude` (decimal degrees) and `elevation`. `inspect` shows it. The model's own coordinates never
change. `Georeference::check` (called by `Model::check`) keeps every number finite and in range and refuses a position on a
map without the map's name. A file holds it as an optional field, so an older BIMCraft opens the file without it (it is
listed as left out); `FORMAT` didn't change.

IFC4: an `IfcMapConversion` from the model's `IfcGeometricRepresentationContext` to an `IfcProjectedCRS` whose `MapUnit` is
the metre, `Eastings`, `Northings` and `OrthogonalHeight` in metres and `Scale` 0.001 (the model's millimetres to the
map's metres), `XAxisAbscissa` and `XAxisOrdinate` the cosine and sine of minus the rotation; the context's `TrueNorth` the
2D direction to true north in the model's plan; the site's `RefLatitude` and `RefLongitude` as compound plane angles
(degrees, minutes, seconds, millionths, each with the angle's sign) and `RefElevation` in mm. IfcOpenShell's
`ifcopenshell.util.geolocation` reads them as written (`auto_xyz2enh` of the model's origin gives the eastings and
northings). Import reads them back: the map's unit from `MapUnit` (or the project's over `Scale`), the rotation from the
x axis, true north kept only where it isn't grid north, and the georeference moved with the model when a far-off file is
brought near the origin.

A scan in map coordinates comes in with `point_cloud {origin: map}`: its middle placed where the georeference puts it,
its height over the vertical datum less the orthogonal height, and turned by the rotation (see
[point clouds](point-clouds.md)).
