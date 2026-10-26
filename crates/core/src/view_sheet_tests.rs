//! Views and sheets as their output shows them: what `render` draws of a plan view, where `place` puts a view on a sheet and
//! at what scale, the title block's lettering, and what the framing of levels and the 3D view takes in. Driven through the
//! commands every front end drives.

use serde_json::{Value, json};

use crate::engine::Session;
use crate::render::{POCHE, Rgb};

const BOX: &str = r#"wall {"points": [[0,0],[8000,0],[8000,6000],[0,6000]], "closed": true}
level {"elevation": 3000}"#;

fn house() -> Session {
    let mut s = Session::new();
    s.script(BOX).unwrap();
    s
}

fn run(s: &mut Session, command: &str, params: Value) -> Value {
    s.execute(command, &params).unwrap_or_else(|e| panic!("{command} {params}: {e}"))
}

/// The picture a reply carries.
fn picture(reply: &Value) -> tiny_skia::Pixmap {
    let bytes = crate::texture::unbase64(reply["pngBase64"].as_str().unwrap()).unwrap();
    tiny_skia::Pixmap::decode_png(&bytes).unwrap()
}

/// How many of the picture's pixels are `c`.
fn count(pm: &tiny_skia::Pixmap, c: Rgb) -> usize {
    pm.pixels().iter().filter(|p| (p.red(), p.green(), p.blue()) == (c.0, c.1, c.2)).count()
}

#[test]
fn rendering_a_duplicated_plan_draws_it_with_its_own_graphics_and_names_it() {
    // Given a level's plan and a duplicate of it that hides the walls
    let mut s = house();
    let copy = run(&mut s, "duplicate_view", json!({"view": "Level 1", "name": "No walls"}))["id"].as_u64().unwrap();
    run(&mut s, "graphics", json!({"view": "No walls", "hide": ["walls"]}));
    // When each is rendered
    let own = run(&mut s, "render", json!({"view": "Level 1", "width": 400, "height": 300}));
    let by_name = run(&mut s, "render", json!({"view": "No walls", "width": 400, "height": 300}));
    let by_id = run(&mut s, "render", json!({"view": copy, "width": 400, "height": 300}));
    // Then the level's plan cuts its walls, the duplicate shows none, and each reply names the view it drew
    let cut = count(&picture(&own), POCHE);
    assert!(cut > 1000, "the level's plan cuts its walls: {cut}");
    for r in [&by_name, &by_id] {
        // (Lettering's edges may come out the poché's grey here and there.)
        let left = count(&picture(r), POCHE);
        assert!(left * 20 < cut, "the duplicate hides them: {left} of {cut}");
        assert_eq!(r["view"], "No walls");
    }
    assert_eq!(own["view"], "Level 1");
}

#[test]
fn a_duplicated_plan_placed_on_a_sheet_keeps_its_own_graphics() {
    let mut s = house();
    run(&mut s, "duplicate_view", json!({"view": "Level 1", "name": "No walls"}));
    run(&mut s, "graphics", json!({"view": "No walls", "hide": ["walls"]}));
    run(&mut s, "sheet", json!({"name": "Own"}));
    run(&mut s, "sheet", json!({"name": "Copy"}));
    run(&mut s, "place", json!({"sheet": "A101", "view": "Level 1"}));
    run(&mut s, "place", json!({"sheet": "A102", "view": "No walls"}));
    let own = picture(&run(&mut s, "render", json!({"view": "A101"})));
    let copy = picture(&run(&mut s, "render", json!({"view": "A102"})));
    let (cut, left) = (count(&own, POCHE), count(&copy, POCHE));
    assert!(cut > 1000 && left * 20 < cut, "{left} of {cut}");
}

// ---------- point clouds and the framing of views ----------

/// A flat scan 185 m by 150 m, a point every 5 m, red.
fn terrain(test: &str) -> String {
    let path = std::env::temp_dir().join(format!("bimcraft-terrain-{}-{test}.xyz", std::process::id()));
    let mut text = String::new();
    for i in 0..=37 {
        for k in 0..=30 {
            text += &format!("{} {} 0 255 0 0\n", f64::from(i) * 5.0, f64::from(k) * 5.0);
        }
    }
    std::fs::write(&path, text).unwrap();
    path.to_string_lossy().into_owned()
}

/// How many pixels of a 3D picture are neither its background (its top left corner's) nor the scan's red.
fn building(pm: &tiny_skia::Pixmap) -> usize {
    let bg = pm.pixel(0, 0).unwrap();
    let bg = (bg.red(), bg.green(), bg.blue());
    pm.pixels().iter().map(|p| (p.red(), p.green(), p.blue())).filter(|&c| c != bg && !(c.0 > 200 && c.1 < 60 && c.2 < 60)).count()
}

#[test]
fn a_scan_round_the_building_leaves_the_levels_sections_and_3d_view_framed_on_what_is_built() {
    // Given a house, and a terrain scan far bigger than it lying behind it, north-west
    let mut s = house();
    let walls_only = run(&mut s, "render", json!({"view": "{3D}", "width": 400, "height": 300}));
    let level = s.model.levels()[0].0;
    let extent = s.model.level_extent(level);
    let section = run(&mut s, "section", json!({"start": [-500, 3000], "end": [8500, 3000]}))["id"].as_u64().unwrap();
    let across = crate::sheet::size(&s.model, section, 100.0).unwrap();
    run(&mut s, "point_cloud", json!({"path": terrain("behind"), "at": [-112_500, 85_000]}));
    // Then the levels stretch over the house, not the scan
    assert_eq!(s.model.level_extent(level), extent);
    // and a section with no crop is as wide as before
    let now = crate::sheet::size(&s.model, section, 100.0).unwrap();
    assert!((now.0 - across.0).abs() < 1.0, "{now:?} against {across:?}");
    // and the 3D view frames the house as it did, not the scan
    let with_scan = run(&mut s, "render", json!({"view": "{3D}", "width": 400, "height": 300}));
    let (before, after) = (building(&picture(&walls_only)), building(&picture(&with_scan)));
    assert!(after * 10 > before * 7, "{after} pixels of the house, {before} without the scan");
}

#[test]
fn a_scan_alone_is_what_the_levels_frame() {
    let mut s = Session::new();
    run(&mut s, "point_cloud", json!({"path": terrain("alone"), "at": [0, 0]}));
    let level = s.model.levels()[0].0;
    let [lo, hi] = s.model.level_extent(level).unwrap();
    assert!(hi.0 - lo.0 > 185_000.0 && hi.1 - lo.1 > 150_000.0, "{lo:?} to {hi:?}");
}
