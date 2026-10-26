//! A roof's `overhang` past an explicit `boundary`: the outline reaches past it, and the slope starts from it at `offset`,
//! as Revit's footprint roof over walls with an overhang. Driven through the commands every front end drives.

use serde_json::{Value, json};

use crate::engine::Session;

fn run(s: &mut Session, command: &str, params: Value) -> Value {
    s.execute(command, &params).unwrap_or_else(|e| panic!("{command} {params}: {e}"))
}

fn roof_json(s: &mut Session, id: &Value) -> Value {
    let all = run(s, "elements", json!({"category": "roofs"}));
    all["elements"].as_array().unwrap().iter().find(|e| e["id"] == *id).cloned().unwrap()
}

#[test]
fn a_roof_over_a_boundary_overhangs_it_and_rises_from_it() {
    // Given a mono-pitch roof sketched on the walls' line, 10 x 20 m, rising east at 10°, with a 750 mm overhang:
    let mut s = Session::new();
    let r = run(
        &mut s,
        "roof",
        json!({"level": "Level 1", "boundary": [[0,0],[10000,0],[10000,20000],[0,20000]], "slope": 10, "gables": [0,1,2],
               "overhang": 750, "offset": 6000}),
    );
    let roof = roof_json(&mut s, &r["id"]);
    // Then its outline reaches 750 mm past the boundary all round (its gable ends too, as Revit's do),
    let outline: Vec<[f64; 2]> = serde_json::from_value(roof["outline"].clone()).unwrap();
    let mut corners = outline.clone();
    corners.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(corners, vec![[-750.0, -750.0], [-750.0, 20750.0], [10750.0, -750.0], [10750.0, 20750.0]], "{roof}");
    // and its base is at `offset` on the boundary: up 10° from the west line, so 10.75 m across at the overhanging ridge.
    let ridge = roof["ridge"].as_f64().unwrap();
    let expected = 6000.0 + 10750.0 * 10f64.to_radians().tan();
    assert!((ridge - expected).abs() < 0.02, "ridge {ridge}, expected {expected}");
}

#[test]
fn a_roof_over_a_boundary_without_overhang_is_the_boundary() {
    let mut s = Session::new();
    let r = run(
        &mut s,
        "roof",
        json!({"level": "Level 1", "boundary": [[0,0],[10000,0],[10000,20000],[0,20000]], "slope": 10, "gables": [0,1,2], "offset": 6000}),
    );
    let roof = roof_json(&mut s, &r["id"]);
    let outline: Vec<[f64; 2]> = serde_json::from_value(roof["outline"].clone()).unwrap();
    assert_eq!(outline, vec![[0.0, 0.0], [10000.0, 0.0], [10000.0, 20000.0], [0.0, 20000.0]]);
    let expected = 6000.0 + 10000.0 * 10f64.to_radians().tan();
    assert!((roof["ridge"].as_f64().unwrap() - expected).abs() < 0.02, "{roof}");
}

#[test]
fn a_clockwise_boundary_overhangs_outward_too_and_keeps_its_edge_numbers() {
    let mut s = Session::new();
    // The same rectangle, clockwise from the north-west corner: edge 3 (the west one, from [0,0] up) is the one sloping.
    let r = run(
        &mut s,
        "roof",
        json!({"level": "Level 1", "boundary": [[0,20000],[10000,20000],[10000,0],[0,0]], "slope": 10, "gables": [0,1,2],
               "overhang": 750, "offset": 6000}),
    );
    let roof = roof_json(&mut s, &r["id"]);
    let outline: Vec<[f64; 2]> = serde_json::from_value(roof["outline"].clone()).unwrap();
    assert!(outline.iter().all(|p| p[0].abs() == 750.0 || p[0] == 10750.0), "{roof}");
    let expected = 6000.0 + 10750.0 * 10f64.to_radians().tan();
    assert!((roof["ridge"].as_f64().unwrap() - expected).abs() < 0.02, "{roof}");
}

#[test]
fn a_roof_over_the_walls_still_overhangs_their_faces() {
    // Unchanged: over the walls of the level below, `overhang` (500) runs past their outer faces.
    let mut s = Session::new();
    run(&mut s, "wall", json!({"points": [[0, 0], [8000, 0], [8000, 6000], [0, 6000]], "closed": true}));
    let r = run(&mut s, "roof", json!({}));
    let roof = roof_json(&mut s, &r["id"]);
    let outline: Vec<[f64; 2]> = serde_json::from_value(roof["outline"].clone()).unwrap();
    let min_x = outline.iter().map(|p| p[0]).fold(f64::MAX, f64::min);
    assert!(min_x < -500.0 - 1.0, "past the wall's outer face, not its centre line: {min_x}");
    let expected = run(&mut s, "elements", json!({"category": "roofs"}))["elements"][0]["overhang"].clone();
    assert_eq!(expected, 500.0, "{roof}");
}

#[test]
fn a_roof_by_face_overhangs_its_mass_and_keeps_overhanging_when_the_mass_moves() {
    let mut s = Session::new();
    let mass = run(&mut s, "mass", json!({"boundary": [[0, 0], [8000, 0], [8000, 6000], [0, 6000]], "height": 3000}))["id"].clone();
    let r = run(&mut s, "roof_by_face", json!({"mass": mass, "overhang": 400}));
    let outline = |s: &mut Session| -> Vec<[f64; 2]> { serde_json::from_value(roof_json(s, &r["id"])["outline"].clone()).unwrap() };
    assert_eq!(outline(&mut s), vec![[-400.0, -400.0], [8400.0, -400.0], [8400.0, 6400.0], [-400.0, 6400.0]]);
    run(&mut s, "modify", json!({"id": mass, "boundary": [[0, 0], [10000, 0], [10000, 6000], [0, 6000]]}));
    run(&mut s, "update_by_face", json!({"mass": mass}));
    assert_eq!(outline(&mut s), vec![[-400.0, -400.0], [10400.0, -400.0], [10400.0, 6400.0], [-400.0, 6400.0]]);
}

#[test]
fn an_overhang_a_boundary_or_courtyard_cannot_take_is_refused() {
    let mut s = Session::new();
    let e = s.execute("roof", &json!({"boundary": [[0,0],[1000,0],[1000,1000],[0,1000]], "slope": 0, "overhang": -600})).unwrap_err();
    assert!(e.contains("`boundary` is too small"), "{e}");
    let e = s
        .execute(
            "roof",
            &json!({"boundary": [[0,0],[10000,0],[10000,10000],[0,10000]], "courtyards": [[[4000,4000],[4000,5000],[5000,5000],[5000,4000]]],
                    "slope": 0, "overhang": 600}),
        )
        .unwrap_err();
    assert!(e.contains("a courtyard is too small"), "{e}");
    // A courtyard that can take it gets eaves of its own, into it.
    let r = run(
        &mut s,
        "roof",
        json!({"boundary": [[0,0],[10000,0],[10000,10000],[0,10000]], "courtyards": [[[3000,3000],[3000,7000],[7000,7000],[7000,3000]]],
               "slope": 0, "overhang": 500}),
    );
    let all = run(&mut s, "elements", json!({"category": "roofs"}));
    let roof = all["elements"].as_array().unwrap().iter().find(|e| e["id"] == r["id"]).unwrap();
    assert_eq!(roof["courtyards"], json!([[[3500.0, 3500.0], [3500.0, 6500.0], [6500.0, 6500.0], [6500.0, 3500.0]]]));
}
