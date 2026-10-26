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
