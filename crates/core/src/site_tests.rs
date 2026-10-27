//! The site through the commands every front end drives: property lines (the plot's boundary, drawn as the dash-dot line
//! of a site plan, its area in a schedule and in IFC as the site's footprint) and topography (a terrain from points,
//! triangulated, drawn with contours in plan, cut in section, a surface in 3D and an IfcGeographicElement in IFC).

use serde_json::{Value, json};

use crate::engine::Session;
use crate::geom::P2;
use crate::model::{Element, Id};
use crate::render::{self, Drawing, Prim};

fn run(s: &mut Session, id: &str, p: Value) -> Value {
    s.execute(id, &p).unwrap_or_else(|e| panic!("{id} {p}: {e}"))
}

fn fails(s: &mut Session, id: &str, p: Value) -> String {
    match s.execute(id, &p) {
        Ok(v) => panic!("{id} {p} was taken: {v}"),
        Err(e) => e,
    }
}

fn id(v: &Value) -> Id {
    v["id"].as_u64().unwrap_or_else(|| panic!("no id in {v}"))
}

fn one(s: &mut Session, category: &str) -> Value {
    let all = run(s, "elements", json!({"category": category}));
    assert_eq!(all["count"], 1, "{all}");
    all["elements"][0].clone()
}

/// The lines element `id` draws in `d`, but for its lettering.
fn lines_of(d: &Drawing, id: Id) -> Vec<&Prim> {
    d.prims
        .iter()
        .zip(&d.owners)
        .zip(&d.parts)
        .filter(|((p, o), part)| **o == Some(id) && **part != render::Part::Text && matches!(p, Prim::Line { .. }))
        .map(|((p, _), _)| p)
        .collect()
}

// ---------- property lines ----------

/// A plot 100 m by 70 m with a corner cut off: 7,000 m² less 50 m².
const PLOT: [[f64; 2]; 5] = [[0.0, 0.0], [100_000.0, 0.0], [100_000.0, 60_000.0], [90_000.0, 70_000.0], [0.0, 70_000.0]];

#[test]
fn a_property_line_is_a_closed_boundary_with_its_area_perimeter_and_bearings() {
    let mut s = Session::new();
    let r = run(&mut s, "property_line", json!({"points": PLOT, "name": "Flurstück 6/17"}));
    assert!(matches!(s.model.elements.get(id(&r)), Some(Element::PropertyLine(_))), "{r}");
    assert_eq!(r["area_m2"], 6950.0);
    assert_eq!(s.history().last().copied(), Some("Property Line"));
    let e = one(&mut s, "property lines");
    assert_eq!(e["name"], "Flurstück 6/17");
    assert_eq!(e["area_m2"], 6950.0);
    // 100 + 60 + √200 + 90 + 70 metres, to the centimetre.
    assert_eq!(e["perimeter_m"], 334.14);
    // Each segment, from each point to the next and back to the first: its length and its bearing, as a survey gives them.
    let segs = e["segments"].as_array().unwrap();
    assert_eq!(segs.len(), 5);
    assert_eq!(segs[0]["length"], 100_000.0);
    assert_eq!(segs[0]["bearing"], "N 90°00'00\" E");
    assert_eq!(segs[1]["bearing"], "N 0°00'00\" E");
    assert_eq!(segs[2]["bearing"], "N 45°00'00\" W");
    assert_eq!(segs[3]["bearing"], "S 90°00'00\" W");
    assert_eq!(segs[4]["bearing"], "S 0°00'00\" E");
    assert_eq!(segs[2]["azimuth"], 315.0);
}

#[test]
fn property_lines_are_scheduled_with_their_areas() {
    let mut s = Session::new();
    run(&mut s, "property_line", json!({"points": PLOT, "name": "Plot A"}));
    run(&mut s, "property_line", json!({"points": [[0, 80000], [20000, 80000], [20000, 100000], [0, 100000]], "name": "Plot B"}));
    let t = run(&mut s, "schedule", json!({"name": "Property Lines", "category": "property lines"}));
    assert_eq!(t["columns"], json!(["Name", "Area (m²)", "Perimeter"]), "{t}");
    assert_eq!(t["rows"], json!([["Plot A", 6950.0, 334140.0], ["Plot B", 400.0, 80000.0]]), "{t}");
}

#[test]
fn what_isnt_a_property_line_is_refused_and_changes_nothing() {
    let mut s = Session::new();
    let before = s.model.clone();
    for (p, says) in [
        (json!({"points": [[0, 0], [1000, 0]]}), "3 to"),
        (json!({"points": [[0, 0], [1000, 0], [2000, 0]]}), "no area"),
        (json!({"points": [[0, 0], [1e300, 0], [0, 1000]]}), "point"),
        (json!({"points": PLOT, "name": "x".repeat(300)}), "name"),
        (json!({"points": PLOT, "level": "nowhere"}), "no level"),
    ] {
        let e = fails(&mut s, "property_line", p.clone());
        assert!(e.contains(says), "{p}: {e}");
    }
    assert_eq!(*s.model, *before);
}

#[test]
fn a_property_line_is_drawn_dash_dot_with_its_name_and_area_in_its_levels_plan() {
    let mut s = Session::new();
    let pl = id(&run(&mut s, "property_line", json!({"points": PLOT, "name": "Plot"})));
    let (l1, l2) = (s.model.levels()[0].0, s.model.levels()[1].0);
    let d = render::plan(&s.model, l1, 1000, 800);
    // Many pieces: dashes, and dots between them.
    let pieces: Vec<f32> = lines_of(&d, pl)
        .into_iter()
        .filter_map(|p| match p {
            Prim::Line { pts, .. } if pts.len() == 2 => {
                pts.first().zip(pts.last()).map(|(a, b)| ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt())
            }
            _ => None,
        })
        .collect();
    let longest = pieces.iter().copied().fold(0.0, f32::max);
    assert!(pieces.len() > 20, "{} pieces", pieces.len());
    assert!(pieces.iter().any(|l| *l < longest / 3.0), "dots among the dashes: {pieces:?}");
    // The plan frames it, and in its pixels the south edge is inked, but not everywhere: there are gaps.
    let png = tiny_skia::Pixmap::decode_png(&render::png(&d).unwrap()).unwrap();
    let (x0, y) = d.view.px(P2(0.0, 0.0));
    let (x1, _) = d.view.px(P2(100_000.0, 0.0));
    let inked: Vec<bool> = (x0.round() as u32 + 2..x1.round() as u32 - 2)
        .map(|x| (y.round() as u32 - 1..=y.round() as u32 + 1).any(|y| png.pixel(x, y).is_some_and(|c| c.red() < 245)))
        .collect();
    let on = inked.iter().filter(|b| **b).count();
    assert!(on > inked.len() / 2 && on < inked.len(), "{on} of {} px inked", inked.len());
    // Not drawn on Level 2.
    assert!(lines_of(&render::plan(&s.model, l2, 1000, 800), pl).is_empty());
}

#[test]
fn a_property_line_is_the_sites_footprint_in_ifc_and_comes_back() {
    let mut s = Session::new();
    run(&mut s, "wall", json!({"points": [[10000, 10000], [30000, 10000], [30000, 20000], [10000, 20000]], "closed": true}));
    run(&mut s, "property_line", json!({"points": PLOT, "name": "Flurstück 6/17"}));
    let ifc = crate::ifc::export(&s.model);
    let site = ifc.lines().find(|l| l.contains("=IFCSITE(")).unwrap();
    // IFCSITE(GlobalId, OwnerHistory, Name, Description, ObjectType, ObjectPlacement, Representation, …): a shape.
    let args: Vec<&str> = site.split(',').collect();
    assert!(args[6].starts_with('#'), "{site}");
    assert!(ifc.contains("'FootPrint','Curve2D'"), "{ifc}");
    assert!(ifc.contains("IFCPROPERTYSINGLEVALUE('TotalArea',$,IFCAREAMEASURE(6950.)"), "{ifc}");
    let i = crate::import::ifc(ifc.as_bytes(), "plot").unwrap();
    let back: Vec<&Element> = i.model.elements.values().filter(|e| matches!(e, Element::PropertyLine(_))).collect();
    assert_eq!(back.len(), 1, "{:?}", i.skipped);
    let Element::PropertyLine(pl) = back[0] else { panic!() };
    assert_eq!(pl.name, "Flurstück 6/17");
    assert_eq!(pl.points, PLOT.iter().map(|p| P2(p[0], p[1])).collect::<Vec<_>>());
}

#[test]
fn a_property_line_moves_and_is_saved_with_the_project() {
    let mut s = Session::new();
    let pl = id(&run(&mut s, "property_line", json!({"points": PLOT})));
    let e = one(&mut s, "propertyline");
    assert_eq!(e["name"], "Property Line 1", "named in order when not given");
    run(&mut s, "move", json!({"ids": [pl], "by": [1000, 0]}));
    let Some(Element::PropertyLine(p)) = s.model.elements.get(pl) else { panic!() };
    assert_eq!(p.points[0], P2(1000.0, 0.0));
    let (m, left_out) = crate::model::Model::read(&s.model.to_json().unwrap()).unwrap();
    assert!(left_out.is_empty(), "{left_out:?}");
    assert_eq!(m.elements, s.model.elements);
}

// ---------- topography ----------

/// A slope 40 m by 20 m, 10 m apart, rising 1 m every 10 m to the east: 15 points, 16 triangles.
fn slope() -> Vec<[f64; 3]> {
    (0..3).flat_map(|j| (0..5).map(move |i| [i as f64 * 10_000.0, j as f64 * 10_000.0, i as f64 * 1000.0])).collect()
}

fn terrain(s: &mut Session) -> Id {
    id(&run(s, "topography", json!({"points": slope(), "name": "Ground"})))
}

#[test]
fn a_small_terrain_from_points_has_the_right_number_of_triangles() {
    let mut s = Session::new();
    let r = run(&mut s, "topography", json!({"points": slope(), "name": "Ground"}));
    assert!(matches!(s.model.elements.get(id(&r)), Some(Element::Topography(_))), "{r}");
    assert_eq!((r["point_count"].as_u64(), r["triangles"].as_u64()), (Some(15), Some(16)), "{r}");
    assert_eq!(s.history().last().copied(), Some("Topography"));
    let e = one(&mut s, "topography");
    assert_eq!(e["triangles"], 16);
    assert_eq!(e["area_m2"], 800.0);
    assert_eq!(e["elevations"], json!([0.0, 4000.0]));
    // A square and its middle, raised: four triangles.
    let r = run(&mut s, "topography", json!({"points": [[0, 0, 0], [1000, 0, 0], [1000, 1000, 0], [0, 1000, 0], [500, 500, 300]]}));
    assert_eq!(r["triangles"], 4);
    // A schedule of them.
    let t = run(&mut s, "schedule", json!({"name": "Topography", "category": "topography"}));
    assert_eq!(t["rows"].as_array().map(Vec::len), Some(2), "{t}");
}

#[test]
fn what_isnt_a_terrain_is_refused_and_changes_nothing() {
    let mut s = Session::new();
    let before = s.model.clone();
    for (p, says) in [
        (json!({}), "points"),
        (json!({"points": [[0, 0, 0], [1, 1, 0]]}), "3 to"),
        (json!({"points": [[0, 0], [1000, 0], [0, 1000]]}), "[x, y, z]"),
        (json!({"points": [[0, 0, 0], [1000, 0, 0], [2000, 0, 5]]}), "one line"),
        (json!({"points": [[0, 0, 0], [1000, 0, 0], [0, 1e300, 0]]}), "[x, y, z]"),
        (json!({"points": slope(), "material": "Unobtainium"}), "material"),
        (json!({"points": slope(), "contours": -5}), "contours"),
        (json!({"points": slope(), "path": "x.xyz"}), "not both"),
        (json!({"path": "/nonexistent/terrain.xyz"}), "terrain.xyz"),
        (json!({"path": "terrain.dwg"}), ".dwg"),
    ] {
        let e = fails(&mut s, "topography", p.clone());
        assert!(e.contains(says), "{p}: {e}");
    }
    assert_eq!(*s.model, *before);
}

#[test]
fn a_terrain_is_drawn_in_its_levels_plan_with_its_edge_and_contours() {
    let mut s = Session::new();
    run(&mut s, "wall", json!({"points": [[5000, 5000], [15000, 5000], [15000, 15000], [5000, 15000]], "closed": true}));
    let t = terrain(&mut s);
    let (l1, l2) = (s.model.levels()[0].0, s.model.levels()[1].0);
    let d = render::plan(&s.model, l1, 1000, 800);
    let lines = lines_of(&d, t);
    // Its edge, and a contour at every metre: 0 and 4 m lie on its edge, so 1, 2 and 3 m between.
    assert!(lines.len() >= 4, "{} lines", lines.len());
    let at = |x: f64| {
        lines.iter().any(|p| match p {
            Prim::Line { pts, .. } => pts.len() >= 2 && pts.iter().all(|q| (d.view.mm(*q).0 - x).abs() < 1.0),
            _ => false,
        })
    };
    assert!(at(10_000.0) && at(20_000.0) && at(30_000.0), "contours at 1, 2 and 3 m");
    // Not on Level 2; no contours with none asked for.
    assert!(lines_of(&render::plan(&s.model, l2, 1000, 800), t).is_empty());
    run(&mut s, "modify", json!({"id": t, "contours": 0}));
    let d = render::plan(&s.model, l1, 1000, 800);
    assert_eq!(lines_of(&d, t).len(), 1, "its edge alone");
    // The plan frames the building, not the whole site: a big terrain doesn't zoom it out, nor stretch the levels.
    let small = s.model.level_extent(l1).unwrap();
    assert!(small[1].0 - small[0].0 < 15_000.0, "{small:?}");
}

#[test]
fn a_section_cuts_a_terrain_at_its_height_and_fills_the_ground_below() {
    let mut s = Session::new();
    let t = terrain(&mut s);
    // Across the slope, west to east along y = 5 m.
    let v = id(&run(&mut s, "section", json!({"start": [0, 5000], "end": [40000, 5000]})));
    let view = render::section_fit(&s.model, v, 1200, 600);
    let d = render::section(&s.model, v, view);
    assert!(d.owners.contains(&Some(t)), "the terrain is cut");
    let png = tiny_skia::Pixmap::decode_png(&render::png(&d).unwrap()).unwrap();
    let shade = |along: f64, up: f64| {
        let (x, y) = d.view.px(P2(along, up));
        png.pixel(x.round() as u32, y.round() as u32).map(|c| c.red()).unwrap_or(255)
    };
    // At 25 m along the ground is 2.5 m up: poché just below it, paper above.
    assert!(shade(25_000.0, 2200.0) < 220, "ground below 2.5 m: {}", shade(25_000.0, 2200.0));
    assert!(shade(25_000.0, 3000.0) > 240, "air above: {}", shade(25_000.0, 3000.0));
    assert!(shade(5_000.0, 200.0) < 220 && shade(5_000.0, 800.0) > 240, "0.5 m at 5 m along");
    // It is a surface in 3D: the view from above sees it in the middle.
    let scene = crate::view3d::Scene::new(&s.model);
    let cam = scene.fit(0.0, 89.0, 400, 300);
    assert_eq!(scene.render(&cam, &[]).pick(200.0, 150.0), Some(t));
}

#[test]
fn a_terrain_goes_to_ifc_as_a_geographic_element_with_a_triangulated_face_set_and_comes_back() {
    let mut s = Session::new();
    run(&mut s, "topography", json!({"points": slope(), "name": "Ground", "material": "Concrete"}));
    let ifc = crate::ifc::export(&s.model);
    let geo = ifc.lines().find(|l| l.contains("=IFCGEOGRAPHICELEMENT(")).unwrap_or_else(|| panic!("{ifc}"));
    assert!(geo.ends_with(".TERRAIN.);"), "{geo}");
    let set = ifc.lines().find(|l| l.contains("=IFCTRIANGULATEDFACESET(")).unwrap();
    assert!(set.contains(",.F.,"), "an open surface: {set}");
    assert_eq!(set.matches("),(").count() + 1, 16, "{set}");
    // Contained in the site.
    let site = ifc.lines().find(|l| l.contains("=IFCSITE(")).unwrap().split('=').next().unwrap().to_string();
    let ent = geo.split('=').next().unwrap();
    assert!(ifc.lines().any(|l| l.contains("IFCRELCONTAINEDINSPATIALSTRUCTURE(") && l.contains(ent) && l.ends_with(&format!(",{site});"))));
    let i = crate::import::ifc(ifc.as_bytes(), "terrain").unwrap();
    let back: Vec<&Element> = i.model.elements.values().filter(|e| matches!(e, Element::Topography(_))).collect();
    assert_eq!(back.len(), 1, "{:?}", i.skipped);
    let Element::Topography(t) = back[0] else { panic!() };
    assert_eq!((t.name.as_str(), t.points.len()), ("Ground", 15));
    assert!(t.points.contains(&[40_000.0, 20_000.0, 4000.0]), "{:?}", t.points);
}

/// A terrain file in a folder of this test's own.
fn file(test: &str, name: &str, text: &str) -> String {
    let dir = std::env::temp_dir().join(format!("bimcraft-site-{}-{test}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path.to_string_lossy().into_owned()
}

#[test]
fn a_terrain_comes_in_from_a_file_of_points_thinned_and_placed() {
    let mut s = Session::new();
    // A 1 m grid, 11 by 11, in survey metres, as a DGM's XYZ file has it: it rises 0.1 m a metre to the north.
    let mut xyz = String::new();
    for j in 0..11 {
        for i in 0..11 {
            xyz += &format!("{} {} {:.1}\n", 500_000 + i, 5_700_000 + j, 95.0 + j as f64 * 0.1);
        }
    }
    let path = file("xyz", "dgm.xyz", &xyz);
    // By default its middle on the origin and its lowest point on the level.
    let r = run(&mut s, "topography", json!({"path": path}));
    assert_eq!(r["point_count"], 121, "{r}");
    let Some(Element::Topography(t)) = s.model.elements.get(id(&r)) else { panic!() };
    let lowest = t.points.iter().map(|p| p[2]).fold(f64::INFINITY, f64::min);
    assert!((lowest - 0.0).abs() < 1e-6 && t.points.contains(&[-5000.0, -5000.0, 0.0]), "{:?}", &t.points[..3]);
    // Thinned to a point every 2 m; a file point put on the project's origin, heights kept from it.
    let r = run(&mut s, "topography", json!({"path": path, "spacing": 2000, "origin": [500_005, 5_700_005, 94]}));
    assert_eq!(r["point_count"], 36, "{r}");
    let Some(Element::Topography(t)) = s.model.elements.get(id(&r)) else { panic!() };
    assert!(t.points.contains(&[-5000.0, -5000.0, 1000.0]), "{:?}", &t.points[..3]);
    // A CSV with a header row.
    let csv = file("csv", "dgm.csv", "x,y,z\n0,0,1\n10,0,1\n10,10,2\n0,10,2\n");
    let r = run(&mut s, "topography", json!({"path": csv, "origin": "file"}));
    assert_eq!(r["triangles"], 2, "{r}");
}

#[test]
fn a_dgm_grid_of_27750_points_comes_in_and_draws_in_reasonable_time() {
    // 185 m by 150 m, a point every metre, gently rolling.
    let pts: Vec<[f64; 3]> = (0..150)
        .flat_map(|j| (0..185).map(move |i| [i as f64 * 1000.0, j as f64 * 1000.0, 2000.0 * ((i as f64) / 30.0).sin() + 15.0 * j as f64]))
        .collect();
    assert_eq!(pts.len(), 27_750);
    let mut s = Session::new();
    let clock = std::time::Instant::now();
    let r = run(&mut s, "topography", json!({"points": pts}));
    let made = clock.elapsed();
    assert_eq!(r["triangles"], 2 * 184 * 149, "{r}");
    let level = s.model.levels()[0].0;
    let clock = std::time::Instant::now();
    let d = render::plan(&s.model, level, 1200, 900);
    let drawn = clock.elapsed();
    assert!(d.owners.contains(&Some(id(&r))));
    let v = id(&run(&mut s, "section", json!({"start": [0, 75000], "end": [185000, 75000]})));
    let clock = std::time::Instant::now();
    let d = render::section(&s.model, v, render::section_fit(&s.model, v, 1200, 600));
    let cut = clock.elapsed();
    assert!(d.owners.contains(&Some(id(&r))));
    eprintln!("27,750 points: made in {made:?}, plan in {drawn:?}, section in {cut:?}");
    assert!((made + drawn + cut).as_secs() < 120, "{made:?} {drawn:?} {cut:?}");
}

#[test]
fn a_terrain_moves_and_is_saved_with_the_project() {
    let mut s = Session::new();
    let t = terrain(&mut s);
    run(&mut s, "move", json!({"ids": [t], "by": [1000, 0]}));
    let Some(Element::Topography(x)) = s.model.elements.get(t) else { panic!() };
    assert_eq!(x.points[0], [1000.0, 0.0, 0.0]);
    let (m, left_out) = crate::model::Model::read(&s.model.to_json().unwrap()).unwrap();
    assert!(left_out.is_empty(), "{left_out:?}");
    assert_eq!(m.elements, s.model.elements);
}
