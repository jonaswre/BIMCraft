//! Room separation lines (Revit's Room Separator) through the commands every front end drives: lines on a level that bound
//! rooms as walls do, drawn thin in its plan, with no body in 3D, sections or IFC's solids.

use serde_json::{Value, json};

use crate::engine::Session;
use crate::model::{Element, Id};
use crate::render;

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

/// A hall 20 m by 10 m with nothing inside.
fn hall() -> Session {
    let mut s = Session::new();
    run(&mut s, "wall", json!({"points": [[0, 0], [20000, 0], [20000, 10000], [0, 10000]], "closed": true}));
    s
}

fn area(s: &mut Session, room: Id) -> f64 {
    let all = run(s, "elements", json!({"category": "room"}));
    let r = all["elements"].as_array().unwrap().iter().find(|e| e["id"] == room).cloned().unwrap();
    r["area_m2"].as_f64().unwrap()
}

#[test]
fn a_separator_splits_an_open_hall_into_two_rooms_with_the_right_areas() {
    let mut s = hall();
    // Without one, the hall is one room of 200 m².
    let whole = run(&mut s, "room", json!({"point": [5000, 5000], "name": "Hall"}));
    assert_eq!(whole["area_m2"], 200.0);
    let r = run(&mut s, "room_separator", json!({"points": [[10000, 0], [10000, 10000]]}));
    let sep = id(&r);
    assert!(matches!(s.model.elements.get(sep), Some(Element::RoomSeparator(_))), "{r}");
    assert_eq!(s.history().last().copied(), Some("Room Separator"));
    // The room it was is now the western half; the eastern half is a room of its own.
    assert_eq!(area(&mut s, id(&whole)), 100.0);
    let east = run(&mut s, "room", json!({"point": [15000, 5000], "name": "Hall 2"}));
    assert_eq!(east["area_m2"], 100.0);
    // Taken away, the rooms are one again.
    run(&mut s, "delete", json!({"ids": [sep]}));
    assert_eq!(area(&mut s, id(&whole)), 200.0);
}

#[test]
fn a_drive_through_open_at_both_ends_becomes_a_room_once_two_separators_close_it() {
    let mut s = Session::new();
    // Two halls, 8 m apart: the drive-through between them is open to the south and to the north.
    run(&mut s, "wall", json!({"points": [[0, 0], [10000, 0], [10000, 10000], [0, 10000]], "closed": true}));
    run(&mut s, "wall", json!({"points": [[18000, 0], [28000, 0], [28000, 10000], [18000, 10000]], "closed": true}));
    let e = fails(&mut s, "room", json!({"point": [14000, 5000]}));
    assert!(e.contains("isn't enclosed"), "{e}");
    run(&mut s, "room_separator", json!({"points": [[10000, 0], [18000, 0]]}));
    run(&mut s, "room_separator", json!({"points": [[10000, 10000], [18000, 10000]]}));
    let r = run(&mut s, "room", json!({"point": [14000, 5000], "name": "Drive-through"}));
    assert_eq!(r["area_m2"], 80.0);
    // The halls are as they were.
    let west = run(&mut s, "room", json!({"point": [5000, 5000]}));
    assert_eq!(west["area_m2"], 100.0);
}

#[test]
fn a_separator_drawn_to_a_walls_face_reaches_on_to_its_centre_line() {
    // Revit's way: a separator is drawn from face to face; rooms are measured at the walls' centre lines.
    let mut s = hall();
    let t = run(&mut s, "elements", json!({"category": "wall"}))["elements"][0]["thickness"].as_f64().unwrap();
    run(&mut s, "room_separator", json!({"points": [[10000, t / 2.0], [10000, 10000.0 - t / 2.0]]}));
    let west = run(&mut s, "room", json!({"point": [5000, 5000]}));
    assert_eq!(west["area_m2"], 100.0);
}

#[test]
fn a_separator_is_a_polyline_on_its_level_that_closes_when_asked() {
    let mut s = hall();
    // A closed loop of separators inside the hall is a room of its own; the hall round it keeps the rest.
    let r = run(
        &mut s,
        "room_separator",
        json!({"points": [[2000, 2000], [6000, 2000], [6000, 6000], [2000, 6000]], "closed": true, "level": "Level 1"}),
    );
    let e = run(&mut s, "elements", json!({"category": "roomseparator"}));
    assert_eq!(e["count"], 1);
    let el = &e["elements"][0];
    assert_eq!(el["id"], r["id"]);
    assert_eq!(el["points"].as_array().map(Vec::len), Some(5), "closed: back to its first point, {el}");
    assert_eq!(el["length"], 16000.0);
    let inner = run(&mut s, "room", json!({"point": [4000, 4000]}));
    assert_eq!(inner["area_m2"], 16.0);
    let outer = run(&mut s, "room", json!({"point": [15000, 5000]}));
    assert_eq!(outer["area_m2"], 184.0);
    // On Level 2 there are neither walls nor separators.
    let e = fails(&mut s, "room", json!({"point": [4000, 4000], "level": "Level 2"}));
    assert!(e.contains("isn't enclosed"), "{e}");
}

#[test]
fn what_isnt_a_separator_is_refused_and_changes_nothing() {
    let mut s = hall();
    let before = s.model.clone();
    for (p, says) in [
        (json!({"points": [[0, 0]]}), "2 to"),
        (json!({"points": [[0, 0], [0, 0]]}), "too short"),
        (json!({"points": [[0, 0], [1e300, 0]]}), "point"),
        (json!({"points": "x"}), "points"),
        (json!({"points": [[0, 0], [1000, 0]], "closed": "yes"}), "closed"),
        (json!({"points": [[0, 0], [1000, 0]], "level": "nowhere"}), "no level"),
    ] {
        let e = fails(&mut s, "room_separator", p.clone());
        assert!(e.contains(says), "{p}: {e}");
    }
    assert_eq!(*s.model, *before);
}

#[test]
fn a_separator_is_drawn_thin_in_its_levels_plan_and_has_no_body() {
    let mut s = hall();
    let sep = id(&run(&mut s, "room_separator", json!({"points": [[10000, 0], [10000, 10000]]})));
    let (l1, l2) = (s.model.levels()[0].0, s.model.levels()[1].0);
    let d = render::plan(&s.model, l1, 800, 600);
    let lines: Vec<f32> = d
        .prims
        .iter()
        .zip(&d.owners)
        .filter_map(|(p, o)| match p {
            render::Prim::Line { width, .. } if *o == Some(sep) => Some(*width),
            _ => None,
        })
        .collect();
    assert_eq!(lines.len(), 1, "one line in its own plan");
    let wall = s.model.elements.iter().find(|(_, e)| matches!(e, Element::Wall(_))).map(|(i, _)| *i).unwrap();
    let wall_width = d
        .prims
        .iter()
        .zip(&d.owners)
        .filter_map(|(p, o)| match p {
            render::Prim::Line { width, .. } if *o == Some(wall) => Some(*width),
            _ => None,
        })
        .fold(0.0, f32::max);
    assert!(lines[0] < wall_width, "thinner than a cut wall: {} vs {wall_width}", lines[0]);
    // In the plan's pixels: ink along x = 10 m, half way up.
    let png = tiny_skia::Pixmap::decode_png(&render::png(&d).unwrap()).unwrap();
    let (x, y) = d.view.px(crate::geom::P2(10000.0, 5000.0));
    let dark = |x: f32, y: f32| png.pixel(x.round() as u32, y.round() as u32).is_some_and(|c| c.red() < 235);
    assert!(dark(x, y) || dark(x + 1.0, y) || dark(x - 1.0, y), "the line shows at [10000, 5000]");
    // Not on Level 2's plan, and nothing of it in 3D or picked there.
    let d2 = render::plan(&s.model, l2, 800, 600);
    assert!(!d2.owners.contains(&Some(sep)));
    assert!(crate::view3d::solids(&s.model).iter().all(|x| x.owner() != sep));
    // It moves, turns and mirrors as lines do.
    run(&mut s, "move", json!({"ids": [sep], "by": [-2000, 0]}));
    let Some(Element::RoomSeparator(rs)) = s.model.elements.get(sep) else { panic!() };
    assert_eq!(rs.points.first().map(|p| p.0), Some(8000.0));
}

#[test]
fn separators_go_to_ifc_as_virtual_elements_and_come_back_so_rooms_stay_closed() {
    let mut s = hall();
    run(&mut s, "room_separator", json!({"points": [[10000, 0], [10000, 10000]]}));
    run(&mut s, "room", json!({"point": [5000, 5000], "name": "West"}));
    run(&mut s, "room", json!({"point": [15000, 5000], "name": "East"}));
    let ifc = crate::ifc::export(&s.model);
    let virtuals: Vec<&str> = ifc.lines().filter(|l| l.contains("=IFCVIRTUALELEMENT(")).collect();
    assert_eq!(virtuals.len(), 1, "{ifc}");
    // Contained in its storey, with a curve for its line and no body.
    let ent = virtuals[0].split('=').next().unwrap();
    assert!(
        ifc.lines()
            .any(|l| l.contains("IFCRELCONTAINEDINSPATIALSTRUCTURE(") && l.contains(&format!("{ent},")) || l.contains(&format!("{ent})")))
    );
    assert!(ifc.contains("'FootPrint','Curve2D'") || ifc.contains("'Axis','Curve2D'"));
    let i = crate::import::ifc(ifc.as_bytes(), "hall").unwrap();
    let back: Vec<&Element> = i.model.elements.values().filter(|e| matches!(e, Element::RoomSeparator(_))).collect();
    assert_eq!(back.len(), 1, "{:?}", i.skipped);
    let Element::RoomSeparator(rs) = back[0] else { panic!() };
    assert_eq!(rs.points, vec![crate::geom::P2(10000.0, 0.0), crate::geom::P2(10000.0, 10000.0)]);
    for e in i.model.elements.values() {
        if let Element::Room(r) = e {
            assert_eq!(i.model.room_region(r).map(|g| (g.area() / 1e6).round()), Some(100.0), "{}", r.name);
        }
    }
}

#[test]
fn a_separator_is_saved_with_the_project_and_read_back() {
    let mut s = hall();
    run(&mut s, "room_separator", json!({"points": [[10000, 0], [10000, 10000]]}));
    let (m, left_out) = crate::model::Model::read(&s.model.to_json().unwrap()).unwrap();
    assert!(left_out.is_empty(), "{left_out:?}");
    assert_eq!(m.elements, s.model.elements);
}
