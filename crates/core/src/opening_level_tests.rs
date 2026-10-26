//! The level a door or window finds its wall on when it isn't told one. Driven through the commands every front end drives.

use serde_json::{Value, json};

use crate::engine::Session;

fn run(s: &mut Session, command: &str, params: Value) -> Value {
    s.execute(command, &params).unwrap_or_else(|e| panic!("{command} {params}: {e}"))
}

/// Levels as Werk 4 has them: a foundation level below ground, the ground floor and an upper floor; a box of walls on the
/// ground floor.
fn werk() -> Session {
    let mut s = Session::new();
    s.script(
        "level {\"elevation\": -2000, \"name\": \"Gründung -2.00\"}\n\
         wall {\"level\": \"Level 1\", \"points\": [[0,0],[8000,0],[8000,6000],[0,6000]], \"closed\": true}",
    )
    .unwrap();
    s
}

#[test]
fn a_door_without_a_level_finds_its_wall_on_whichever_level_has_one_there() {
    let mut s = werk();
    let walls = run(&mut s, "elements", json!({"category": "walls"}));
    let d = run(&mut s, "door", json!({"at": [2000, 0]}));
    assert_eq!(d["host"], walls["elements"][0]["id"], "{d}");
    let w = run(&mut s, "window", json!({"at": [8000, 3000]}));
    assert_eq!(w["host"], walls["elements"][1]["id"], "{w}");
}

#[test]
fn a_door_with_a_level_looks_only_there() {
    let mut s = werk();
    let e = s.execute("door", &json!({"at": [2000, 0], "level": "Gründung -2.00"})).unwrap_err();
    assert!(e.contains("no wall at [2000, 0] on Gründung -2.00"), "{e}");
}

#[test]
fn a_door_without_a_level_where_no_level_has_a_wall_says_so() {
    let mut s = werk();
    let e = s.execute("door", &json!({"at": [4000, 3000]})).unwrap_err();
    assert!(e.contains("no wall at [4000, 3000] on any level"), "{e}");
}

#[test]
fn a_door_without_a_level_prefers_the_ground_floor_over_one_below_it() {
    // A foundation wall under the ground floor's: the door goes in the ground floor's.
    let mut s = werk();
    let f = run(&mut s, "wall", json!({"level": "Gründung -2.00", "points": [[0,0],[8000,0]], "height": 1000}));
    let d = run(&mut s, "door", json!({"at": [2000, 0]}));
    assert!(d["host"].is_u64() && d["host"] != f["ids"][0], "{d}: not the foundation wall");
    // And above ground, the lowest: Level 2's wall over the ground floor's is not the one.
    let upper = run(&mut s, "wall", json!({"level": "Level 2", "points": [[0,6000],[8000,6000]]}));
    let d = run(&mut s, "door", json!({"at": [5000, 6000]}));
    assert!(d["host"].is_u64() && d["host"] != upper["ids"][0], "{d}: the ground floor's wall, not Level 2's");
}
