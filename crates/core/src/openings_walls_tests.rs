//! Walls that stand off their level and the openings in walls, as an industrial model drives them: raised walls over a
//! lower roof, openings stacked in a wall's elevation, the host a point finds, and walls through an IFC round trip.
//! Driven through the commands every front end drives.

use serde_json::{Value, json};

use crate::engine::Session;
use crate::model::Id;

fn run(s: &mut Session, command: &str, params: Value) -> Value {
    s.execute(command, &params).unwrap_or_else(|e| panic!("{command}: {e}"))
}

fn id(v: &Value) -> Id {
    v["id"].as_u64().unwrap()
}

fn first_level(s: &Session) -> Id {
    s.model.levels()[0].0
}

/// The rooms `elements` lists: (area in m², enclosed).
fn rooms(s: &mut Session) -> Vec<(f64, bool)> {
    let r = run(s, "elements", json!({"category": "room"}));
    r["elements"].as_array().unwrap().iter().map(|e| (e["area_m2"].as_f64().unwrap(), e["enclosed"].as_bool().unwrap())).collect()
}

/// A hall on the first level, 20 x 8 m, its walls on their exterior faces and drawn clockwise; its south side in two
/// walls meeting at x = 10000, the west one of them a gable. A mono-pitch roof rises from the west eave to the east, and
/// the gable is attached to it, so it reaches above 7.2 m at its east end. Returns the gable.
fn hall(s: &mut Session) -> Id {
    let level = first_level(s);
    let wall = |points: Value, height: f64| json!({"points": points, "level": level, "height": height, "location": "exterior-finish"});
    run(s, "wall", wall(json!([[0, 0], [0, 8000], [20000, 8000], [20000, 0], [10000, 0]]), 7200.0));
    let gable = run(s, "wall", wall(json!([[10000, 0], [0, 0]]), 6000.0))["ids"][0].as_u64().unwrap();
    let roof = run(
        s,
        "roof",
        json!({"level": level, "boundary": [[-500, -500], [10500, -500], [10500, 8500], [-500, 8500]], "gables": [0, 1, 2],
               "slope": 10, "offset": 6000, "overhang": 0}),
    );
    run(s, "modify", json!({"id": gable, "attach_top": id(&roof)}));
    gable
}

#[test]
fn a_raised_wall_meeting_a_roof_attached_wall_leaves_the_room_below_enclosed() {
    // Given a hall whose room is enclosed
    let mut s = Session::new();
    hall(&mut s);
    let level = first_level(&s);
    let room = run(&mut s, "room", json!({"point": [5000, 4000], "level": level}));
    let area = room["area_m2"].as_f64().unwrap();
    assert!(area > 150.0, "{room}");
    // When a wall standing 7.2 m above the level, over the room, ends at the corner of the attached gable
    run(
        &mut s,
        "wall",
        json!({"points": [[10000, 8000], [10000, 0]], "level": level, "base_offset": 7200, "height": 3000,
               "location": "exterior-finish"}),
    );
    // Then the room below is still enclosed, with the same area: the raised wall doesn't reach down to it
    assert_eq!(rooms(&mut s), vec![(area, true)]);
    // And a new room there is placed as before.
    let again = run(&mut s, "room", json!({"point": [15000, 4000], "level": level}));
    assert_eq!(again["area_m2"].as_f64(), Some(area));
}

#[test]
fn a_window_above_a_door_in_the_same_stretch_of_wall_is_placed() {
    // Given a tall wall with a door in it, 2.1 m high
    let mut s = Session::new();
    let level = first_level(&s);
    run(&mut s, "wall", json!({"points": [[0, 0], [0, 20000]], "height": 6000, "level": level}));
    let door = run(&mut s, "door", json!({"at": [0, 5000], "level": level}));
    // When a window goes over it, its sill at 2.5 m
    let transom = s.execute("window", &json!({"at": [0, 5000], "level": level, "sill": 2500}));
    // Then both are in the wall
    let transom = transom.unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(transom["host"], door["host"]);
    // And a window over that one, as an upper floor's, too
    run(&mut s, "window", json!({"at": [0, 5000], "level": level, "sill": 4000}));
    assert_eq!(run(&mut s, "elements", json!({"category": "window"}))["elements"].as_array().unwrap().len(), 2);
    // But a window that would cut into the door, or into another window, is still refused
    let e = s.execute("window", &json!({"at": [0, 5300], "level": level, "sill": 1500})).unwrap_err();
    assert!(e.contains("overlap"), "{e}");
    let e = s.execute("window", &json!({"at": [0, 5300], "level": level, "sill": 3000})).unwrap_err();
    assert!(e.contains("overlap"), "{e}");
    // And moving the upper window down onto the transom is refused too
    let upper = run(&mut s, "elements", json!({"category": "window"}))["elements"][1]["id"].as_u64().unwrap();
    let e = s.execute("modify", &json!({"id": upper, "sill": 2800})).unwrap_err();
    assert!(e.contains("overlap"), "{e}");
}

#[test]
fn stacked_openings_are_each_cut_drawn_and_exported() {
    // Given a door with a window above it in one wall
    let mut s = Session::new();
    let level = first_level(&s);
    let wall = run(&mut s, "wall", json!({"points": [[0, 0], [8000, 0]], "height": 6000, "level": level}))["ids"][0].as_u64().unwrap();
    let door = id(&run(&mut s, "door", json!({"at": [4000, 0], "level": level})));
    let window = id(&run(&mut s, "window", json!({"at": [4000, 0], "level": level, "sill": 2500})));
    // Then the 3D wall is open through the door and through the window, and solid between them
    let open_at = |s: &Session, z: f64| {
        crate::walls::prisms(&s.model, wall, true).iter().all(|p| {
            !(crate::geom::contains(&p.ring, crate::geom::P2(4000.0, 0.0))
                && p.bottom.at(crate::geom::P2(4000.0, 0.0)) <= z
                && z <= p.top.at(crate::geom::P2(4000.0, 0.0)))
        })
    };
    assert!(open_at(&s, 1000.0), "through the door");
    assert!(!open_at(&s, 2300.0), "the wall between door and window");
    assert!(open_at(&s, 3000.0), "through the window");
    // And the plan, cut at 1.2 m, shows the door's swing but not the window above the cut
    let plan = crate::render::draw(&s.model, level, crate::render::plan(&s.model, level, 800, 600).view);
    assert!(plan.owners.contains(&Some(door)));
    assert!(!plan.owners.contains(&Some(window)));
    // And IFC has an opening element for each, voiding the wall, filled by its door or window
    let file = crate::ifc::export(&s.model);
    assert_eq!(file.matches("IFCOPENINGELEMENT(").count(), 2);
    assert_eq!(file.matches("IFCRELVOIDSELEMENT(").count(), 2);
    assert_eq!(file.matches("IFCRELFILLSELEMENT(").count(), 2);
    // And the schedules list both
    let doors = run(&mut s, "elements", json!({"category": "door"}));
    let windows = run(&mut s, "elements", json!({"category": "window"}));
    assert_eq!((doors["elements"].as_array().unwrap().len(), windows["elements"].as_array().unwrap().len()), (1, 1));
}

#[test]
fn a_window_at_a_point_finds_a_wall_raised_above_the_plan_cut() {
    // Given a wall standing 7.2 m above its level, 3 m tall
    let mut s = Session::new();
    let level = first_level(&s);
    let raised = run(&mut s, "wall", json!({"points": [[0, 0], [10000, 0]], "level": level, "base_offset": 7200, "height": 3000}))["ids"]
        [0]
    .as_u64()
    .unwrap();
    // When a window is placed at a point on its line, on either face, or inside it
    for (k, at) in [[2000, 0], [4000, 100], [6000, -100], [8000, 50]].into_iter().enumerate() {
        let w = s.execute("window", &json!({"at": at, "level": level, "mark": format!("R{k}")}));
        // Then it goes in that wall
        let w = w.unwrap_or_else(|e| panic!("{at:?}: {e}"));
        assert_eq!(w["host"].as_u64(), Some(raised), "{at:?}");
    }
    // And a low parapet, below the cut, is found too
    let parapet =
        run(&mut s, "wall", json!({"points": [[0, 5000], [10000, 5000]], "level": level, "height": 900}))["ids"][0].as_u64().unwrap();
    let w = run(&mut s, "window", json!({"at": [5000, 5000], "level": level, "sill": 0, "type": "Fixed 600 x 900"}));
    assert_eq!(w["host"].as_u64(), Some(parapet));
    // But far from any wall, it says so
    let e = s.execute("window", &json!({"at": [5000, 2500], "level": level})).unwrap_err();
    assert!(e.contains("no wall at [5000, 2500]"), "{e}");
}
