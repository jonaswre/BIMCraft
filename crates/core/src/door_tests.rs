//! Door kinds, through the commands every front end drives: swing, double swing, sliding, double sliding, sectional and
//! roller doors; their plan symbols, their leaves in 3D, their schedules, their IFC operation types and what comes back.

use serde_json::{Value, json};

use crate::element_json;
use crate::engine::Session;
use crate::geom::P2;
use crate::render::{self, Drawing, PAPER, Prim, Rgb, View};

fn run(s: &mut Session, id: &str, p: Value) -> Value {
    s.execute(id, &p).unwrap_or_else(|e| panic!("{id} {p}: {e}"))
}

fn fails(s: &mut Session, id: &str, p: Value) -> String {
    let before = s.model.clone();
    let e = s.execute(id, &p).expect_err(&format!("{id} {p} should fail"));
    assert_eq!(*s.model, *before, "a refused {id} changes nothing");
    e
}

/// A project with a 200 wall from (0, 0) east to (40000, 0), its exterior to the north, and a door of each kind in it, 4000
/// apart: their ids, by kind.
fn gates(s: &mut Session) -> Vec<(&'static str, u64)> {
    let wall = run(s, "wall", json!({"points": [[0, 0], [40000, 0]], "height": 7000}))["ids"][0].as_u64().unwrap();
    let kinds = ["swing", "double_swing", "sliding", "double_sliding", "sectional", "roller"];
    let mut out = Vec::new();
    for (i, kind) in kinds.into_iter().enumerate() {
        let name = format!("Gate {kind}");
        let (width, height) = if kind.contains("swing") { (1800, 2400) } else { (3000, 4000) };
        run(s, "type", json!({"category": "door", "name": name, "width": width, "height": height, "operation": kind}));
        let id = run(s, "door", json!({"wall": wall, "offset": 3000 + 6000 * i, "type": name}))["id"].as_u64().unwrap();
        out.push((kind, id));
    }
    out
}

/// What a drawing draws of element `id`.
fn of(d: &Drawing, id: u64) -> Vec<&Prim> {
    d.prims.iter().zip(&d.owners).filter(|(_, o)| **o == Some(id)).map(|(p, _)| p).collect()
}

/// The points, in model mm, of the lines of `id` that are dashed (or not).
fn lines(d: &Drawing, id: u64, dashed: bool) -> Vec<Vec<P2>> {
    of(d, id)
        .into_iter()
        .filter_map(|p| match p {
            Prim::Line { pts, dash, .. } if dash.is_some() == dashed => Some(pts.iter().map(|q| d.view.mm(*q)).collect()),
            _ => None,
        })
        .collect()
}

/// Whether the PNG of `d` has ink within two pixels of model point `p`.
fn inked_near(d: &Drawing, p: P2) -> bool {
    let pm = tiny_skia::Pixmap::decode_png(&render::png(d).unwrap()).unwrap();
    let (x, y) = d.view.px(p);
    (-2..=2)
        .flat_map(|i| (-2..=2).map(move |j| (i, j)))
        .any(|(i, j)| pm.pixel((x as i32 + i) as u32, (y as i32 + j) as u32).is_some_and(|c| Rgb(c.red(), c.green(), c.blue()) != PAPER))
}

fn view(cx: f64) -> View {
    View { scale: 0.2, cx, cy: 1500.0, width: 1600, height: 1400 }
}

#[test]
fn a_door_type_says_how_it_opens_and_a_window_type_cannot() {
    let mut s = Session::new();
    let r = run(&mut s, "type", json!({"category": "door", "name": "Gate", "width": 4000, "height": 5000, "operation": "sectional"}));
    assert_eq!(r["type"], json!({"width": 4000.0, "height": 5000.0, "sill": 0.0, "operation": "sectional"}));
    // Changed, it keeps what isn't said; a swing door says nothing of it.
    let r = run(&mut s, "type", json!({"category": "door", "name": "Gate", "width": 3600}));
    assert_eq!(r["type"]["operation"], "sectional");
    let r = run(&mut s, "type", json!({"category": "door", "name": "Gate", "operation": "Double Sliding"}));
    assert_eq!(r["type"]["operation"], "double_sliding");
    let r = run(&mut s, "type", json!({"category": "door", "name": "Plain", "width": 900, "height": 2100}));
    assert!(r["type"].get("operation").is_none(), "{r}");
    let e = fails(&mut s, "type", json!({"category": "door", "name": "Gate", "operation": "revolving"}));
    assert!(e.contains("sectional") && e.contains("roller"), "the message lists the kinds: {e}");
    assert!(fails(&mut s, "type", json!({"category": "door", "name": "Gate", "operation": 3})).contains("operation"));
    let e = fails(&mut s, "type", json!({"category": "window", "name": "W", "width": 900, "height": 900, "operation": "sliding"}));
    assert!(e.contains("door"), "{e}");
}

#[test]
fn each_kind_of_door_draws_its_own_symbol_in_plan() {
    let mut s = Session::new();
    let doors = gates(&mut s);
    let id = |k: &str| doors.iter().find(|(kind, _)| *kind == k).unwrap().1;
    let d = render::draw(&s.model, 100000, View { scale: 0.05, cx: 20000.0, cy: 2000.0, width: 2200, height: 600 });
    // An arc has many points; a leaf, a rail or a box four.
    let arcs = |k: &str| lines(&d, id(k), false).into_iter().chain(lines(&d, id(k), true)).filter(|l| l.len() > 8).count();
    assert_eq!((arcs("swing"), arcs("double_swing")), (1, 2));
    for k in ["sliding", "double_sliding", "sectional", "roller"] {
        assert_eq!(arcs(k), 0, "a {k} door doesn't swing");
    }
    // A sliding leaf laps the wall past both jambs, on the side it faces, and an arrow shows which way it runs.
    let (lo, hi) = (15000.0 - 1500.0, 15000.0 + 1500.0);
    let solid = lines(&d, id("sliding"), false);
    assert!(
        solid.iter().any(|l| l.iter().any(|p| p.0 < lo - 25.0) && l.iter().any(|p| p.0 > hi + 25.0) && l.iter().all(|p| p.1 > 99.0)),
        "{solid:?}"
    );
    // Arrows: open lines off the wall (the jambs are across it).
    let arrows =
        |k: &str| lines(&d, id(k), false).into_iter().filter(|l| (l.len() == 3 || l.len() == 2) && l.iter().all(|p| p.1 > 101.0)).count();
    assert!(arrows("sliding") >= 2 && arrows("double_sliding") >= 4, "{} {}", arrows("sliding"), arrows("double_sliding"));
    // A sectional door: its leaf across the opening, and overhead, dashed, where it runs open: as far in as it is high.
    let overhead = lines(&d, id("sectional"), true);
    let reach = overhead.iter().flatten().map(|p| p.1).fold(f64::MIN, f64::max);
    assert!(reach > 4000.0 && reach < 4400.0, "{reach}");
    assert!(!lines(&d, id("sectional"), false).is_empty(), "its leaf, cut");
    // A roller: its curtain, and its coil box over the opening, dashed, 450 deep.
    let coil = lines(&d, id("roller"), true);
    let reach = coil.iter().flatten().map(|p| p.1).fold(f64::MIN, f64::max);
    assert!((reach - 550.0).abs() < 1.0, "{reach}");
    // The swing door's arc is drawn in the PNG; where a sectional door of the same width would swing, nothing is.
    let swing = render::draw(&s.model, 100000, view(3000.0));
    let (pivot, r) = (P2(3000.0 - 900.0, 100.0), 1800.0);
    let on_arc = pivot + P2(std::f64::consts::FRAC_1_SQRT_2, std::f64::consts::FRAC_1_SQRT_2) * r;
    assert!(inked_near(&swing, on_arc));
    let sectional = render::draw(&s.model, 100000, view(27000.0));
    assert!(!inked_near(&sectional, P2(27000.0 - 1500.0, 100.0) + P2(0.6, 0.8) * 2900.0 + P2(-200.0, 0.0)));
}

#[test]
fn each_kind_of_door_has_its_leaves_in_3d() {
    let mut s = Session::new();
    let doors = gates(&mut s);
    let solids = crate::view3d::solids(&s.model);
    let of = |k: &str| {
        let id = doors.iter().find(|(kind, _)| *kind == k).unwrap().1;
        solids.iter().filter(|x| x.owner() == id).collect::<Vec<_>>()
    };
    let top = |k: &str| of(k).iter().map(|x| x.heights().1).fold(f64::MIN, f64::max);
    assert_eq!((of("swing").len(), of("double_swing").len()), (1, 2));
    assert_eq!((of("sliding").len(), of("double_sliding").len()), (1, 2));
    // Panels about 600 high: seven in 4000, two rails and two tracks.
    assert_eq!(of("sectional").len(), 7 + 4);
    assert!((top("sectional") - 4100.0).abs() < 1e-6);
    // The roller's coil box over the opening.
    assert!((top("roller") - 4450.0).abs() < 1e-6);
    // A sliding leaf runs outside the wall, on its north (exterior) face.
    let corners: Vec<_> = of("sliding").iter().flat_map(|x| x.corners()).collect();
    assert!(corners.iter().all(|p| p[1] >= 100.0 - 1e-6), "{corners:?}");
}

#[test]
fn schedules_say_how_each_door_opens() {
    let mut s = Session::new();
    gates(&mut s);
    let r = run(&mut s, "schedule", json!({"name": "Door Schedule", "fields": ["Type", "Operation"], "sort": ["Type"]}));
    let ops: Vec<&str> = r["rows"].as_array().unwrap().iter().filter_map(|row| row[1].as_str()).collect();
    assert_eq!(ops, ["double_sliding", "double_swing", "roller", "sectional", "sliding", "swing"]);
}

fn ifc(s: &mut Session) -> String {
    let path = std::env::temp_dir().join(format!("bimcraft-doors-{}-{}.ifc", std::process::id(), s.model.uid));
    run(s, "save", json!({"path": path}));
    let text = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(path);
    text
}

fn entities<'a>(file: &'a str, name: &str) -> Vec<&'a str> {
    let head = format!("={name}(");
    file.lines().filter(|l| l.contains(&head)).collect()
}

#[test]
fn each_kind_of_door_is_an_ifc_door_of_its_operation_type_and_a_door_type_of_it() {
    let mut s = Session::new();
    let doors = gates(&mut s);
    let sliding = doors.iter().find(|(k, _)| *k == "sliding").unwrap().1;
    run(&mut s, "modify", json!({"id": sliding, "flip_hand": true}));
    let file = ifc(&mut s);
    let ifc_doors = entities(&file, "IFCDOOR");
    assert_eq!(ifc_doors.len(), 6);
    let op =
        |name: &str| ifc_doors.iter().find(|d| d.contains(&format!("'Gate {name}'"))).map(|d| d.rsplit(",.DOOR.,").next().unwrap_or(""));
    assert_eq!(op("swing"), Some(".SINGLE_SWING_LEFT.,$);"));
    assert_eq!(op("double_swing"), Some(".DOUBLE_DOOR_SINGLE_SWING.,$);"));
    assert_eq!(op("sliding"), Some(".SLIDING_TO_RIGHT.,$);"));
    assert_eq!(op("double_sliding"), Some(".DOUBLE_DOOR_SLIDING.,$);"));
    assert_eq!(op("sectional"), Some(".USERDEFINED.,'SECTIONAL');"));
    assert_eq!(op("roller"), Some(".ROLLINGUP.,$);"));
    // A door type for each, of its operation, which its door is defined by.
    let types = entities(&file, "IFCDOORTYPE");
    assert_eq!(types.len(), 6, "{file}");
    let sectional = types.iter().find(|t| t.contains("'Gate sectional'")).unwrap();
    assert!(sectional.ends_with(",.DOOR.,.USERDEFINED.,.F.,'SECTIONAL');"), "{sectional}");
    assert!(types.iter().any(|t| t.contains("'Gate roller'") && t.ends_with(",.DOOR.,.ROLLINGUP.,.F.,$);")));
    assert_eq!(entities(&file, "IFCRELDEFINESBYTYPE").len(), 6);
    // A roller shutter's body has its curtain, its rails and its coil box.
    let roller = ifc_doors.iter().find(|d| d.contains("'Gate roller'")).unwrap();
    let shape = roller.split(',').nth(6).unwrap().trim_start_matches('#');
    let shape_line = file.lines().find(|l| l.starts_with(&format!("#{shape}="))).unwrap();
    let rep = shape_line.split("(#").nth(1).unwrap().trim_end_matches("));");
    let rep_line = file.lines().find(|l| l.starts_with(&format!("#{rep}="))).unwrap().split_once('=').unwrap().1;
    assert_eq!(rep_line.matches('#').count(), 1 + 4, "the context and four solids: {rep_line}");
}

#[test]
fn door_kinds_come_back_from_ifc() {
    let mut s = Session::new();
    let doors = gates(&mut s);
    let sliding = doors.iter().find(|(k, _)| *k == "sliding").unwrap().1;
    run(&mut s, "modify", json!({"id": sliding, "flip_hand": true}));
    let path = std::env::temp_dir().join(format!("bimcraft-doors-{}-round.ifc", std::process::id()));
    run(&mut s, "save", json!({"path": path}));
    let mut t = Session::new();
    let r = run(&mut t, "open", json!({"path": path}));
    let _ = std::fs::remove_file(&path);
    assert_eq!(r["skipped"], json!([]), "{r}");
    let kinds = |m: &crate::model::Model| {
        let mut out: Vec<(String, f64, bool)> = m
            .elements
            .iter()
            .filter(|(_, e)| e.category() == "door")
            .map(|(id, e)| {
                let j = element_json(m, *id);
                let flip = matches!(e, crate::model::Element::Door(o) if o.flip_hand);
                (j["operation"].as_str().unwrap_or("").to_string(), j["width"].as_f64().unwrap(), flip)
            })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    };
    assert_eq!(kinds(&t.model), kinds(&s.model));
}

#[test]
fn a_project_with_door_kinds_saves_and_opens_again() {
    let mut s = Session::new();
    gates(&mut s);
    let path = std::env::temp_dir().join(format!("bimcraft-doors-{}.bimcraft", std::process::id()));
    run(&mut s, "save", json!({"path": path}));
    let mut t = Session::new();
    run(&mut t, "open", json!({"path": path}));
    let _ = std::fs::remove_file(&path);
    assert_eq!(t.model.types.door, s.model.types.door);
}
