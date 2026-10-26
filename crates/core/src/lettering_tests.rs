//! What drawings letter: names and notes with the marks engineers write (primes, dashes, German letters), a character the
//! lettering has no glyph for drawn as a crossed box, and the command that stored it warning about it. Driven through the
//! commands every front end drives, and checked through the PNG.

use serde_json::{Value, json};

use crate::engine::Session;
use crate::render;

fn run(s: &mut Session, command: &str, params: Value) -> Value {
    s.execute(command, &params).unwrap_or_else(|e| panic!("{command} {params}: {e}"))
}

/// The first level's plan, as a PNG, with one grid named `name` in it.
fn plan_with_grid(name: &str) -> Vec<u8> {
    let mut s = Session::new();
    run(&mut s, "grid", json!({"start": [0, 0], "end": [0, 10000], "name": name}));
    let level = s.model.levels()[0].0;
    render::png(&render::plan(&s.model, level, 600, 600)).unwrap()
}

#[test]
fn a_grid_named_with_a_prime_is_drawn_with_a_prime_not_a_seven() {
    // Grid 1′ (U+2032 PRIME), as engineers name a secondary grid.
    let prime = plan_with_grid("1′");
    assert_ne!(prime, plan_with_grid("17"), "1′ must not read as 17");
    assert_ne!(prime, plan_with_grid("1?"), "1′ is not drawn as a stand-in");
    assert_ne!(prime, plan_with_grid("1'"), "a prime is not an apostrophe");
}

#[test]
fn a_character_the_lettering_cannot_draw_is_a_box_not_a_question_mark() {
    let unknown = plan_with_grid("1☃");
    assert_ne!(unknown, plan_with_grid("1?"));
    assert_ne!(unknown, plan_with_grid("17"));
    assert_eq!(unknown, plan_with_grid("1✓"), "every character it can't draw is the same box");
}

#[test]
fn a_name_the_drawings_cannot_letter_is_stored_with_a_warning() {
    let mut s = Session::new();
    let r = run(&mut s, "grid", json!({"start": [0, 0], "end": [0, 10000], "name": "1☃"}));
    assert_eq!(r["name"], "1☃", "the name is kept as given");
    let warnings = r["warnings"].as_array().unwrap_or_else(|| panic!("no warnings in {r}"));
    assert_eq!(warnings.len(), 1, "{r}");
    let w = warnings[0].as_str().unwrap();
    assert!(w.contains("☃") && w.contains("U+2603") && w.contains("`name`"), "{w}");

    // Levels, rooms, sheets and text notes too; a name it can draw has no warning.
    let l = run(&mut s, "level", json!({"elevation": 6000, "name": "Dach ✓"}));
    assert!(l["warnings"][0].as_str().unwrap().contains("✓"), "{l}");
    let t = run(&mut s, "text", json!({"text": "Achse 1′–3″ ⚠", "at": [0, 0]}));
    assert!(t["warnings"][0].as_str().unwrap().contains("⚠"), "{t}");
    let sh = run(&mut s, "sheet", json!({"number": "A✓", "name": "Grundriss"}));
    assert!(sh["warnings"][0].as_str().unwrap().contains("`number`"), "{sh}");
    run(&mut s, "wall", json!({"points": [[0, 0], [8000, 0], [8000, 6000], [0, 6000]], "closed": true}));
    let room = run(&mut s, "room", json!({"point": [4000, 3000], "name": "Büro ☕"}));
    assert!(room["warnings"][0].as_str().unwrap().contains("☕"), "{room}");
    // Renaming one through modify warns too.
    let g = r["id"].clone();
    let m = run(&mut s, "modify", json!({"id": g, "name": "2☃"}));
    assert!(m["warnings"][0].as_str().unwrap().contains("☃"), "{m}");

    for ok in [
        run(&mut s, "grid", json!({"start": [3000, 0], "end": [3000, 10000], "name": "1′"})),
        run(&mut s, "level", json!({"elevation": 9000, "name": "Straße ±0.00 – 1. OG …"})),
        run(&mut s, "text", json!({"text": "45° × 2 m² ⌀".replace('⌀', "Ø"), "at": [0, 0]})),
    ] {
        assert!(ok.get("warnings").is_none(), "{ok}");
    }
}
