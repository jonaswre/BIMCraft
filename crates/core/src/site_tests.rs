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
