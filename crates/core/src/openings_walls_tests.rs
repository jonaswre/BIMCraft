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
