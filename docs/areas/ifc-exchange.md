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
