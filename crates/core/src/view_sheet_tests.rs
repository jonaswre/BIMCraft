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

// ---------- placing views on sheets ----------

/// Viewport `vp`'s frame on its sheet, paper mm: its left, bottom (with its title's room under it), right and top.
fn on_paper(s: &Session, vp: u64) -> [f64; 4] {
    let (view, (x, y)) = crate::sheet::frame(&s.model, vp).unwrap();
    let height = match s.model.elements.get(vp) {
        Some(crate::model::Element::Viewport(v)) => crate::model::paper(&s.model.sheet(v.sheet).unwrap().size).unwrap().1,
        _ => panic!("{vp} is no viewport"),
    };
    let mm = |px: f32| f64::from(px) / crate::sheet::DENSITY;
    let (left, top) = (mm(x), height - mm(y));
    [left, top - mm(view.height as f32) - 13.0, left + mm(view.width as f32), top]
}

fn overlap(a: [f64; 4], b: [f64; 4]) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

/// A hall 49 m long and 20 m deep, and `n` sections across it along its length.
fn hall(n: usize) -> Session {
    let mut s = Session::new();
    s.script(r#"wall {"points": [[0,0],[49000,0],[49000,20000],[0,20000]], "closed": true, "height": 8000}"#).unwrap();
    for i in 0..n {
        let y = 2000.0 + 4000.0 * i as f64;
        run(&mut s, "section", json!({"start": [-500, y], "end": [49500, y], "name": format!("Section {i}")}));
    }
    s
}

#[test]
fn views_placed_without_a_spot_go_where_nothing_else_is_and_stay_on_the_paper() {
    // Given an A1 sheet and four sections, each some 300 mm long at 1 : 200
    let mut s = hall(4);
    run(&mut s, "sheet", json!({"name": "Sections", "size": "A1"}));
    // When each is placed with no spot given
    let placed: Vec<u64> = (0..4)
        .map(|i| run(&mut s, "place", json!({"sheet": "A101", "view": format!("Section {i}"), "scale": 200}))["id"].as_u64().unwrap())
        .collect();
    // Then no two overlap, titles and all, and each lies inside the border clear of the title box
    let frames: Vec<[f64; 4]> = placed.iter().map(|vp| on_paper(&s, *vp)).collect();
    for (i, a) in frames.iter().enumerate() {
        for b in &frames[i + 1..] {
            assert!(!overlap(*a, *b), "{a:?} overlaps {b:?}");
        }
        assert!(a[0] >= 20.0 && a[1] >= 10.0 && a[2] <= 831.0 && a[3] <= 584.0, "{a:?} runs off the A1 sheet");
        assert!(!overlap(*a, [841.0 - 10.0 - 180.0, 10.0, 831.0, 50.0]), "{a:?} is over the title box");
    }
}

#[test]
fn the_first_view_placed_goes_in_the_middle_and_the_next_right_of_it_where_there_is_room() {
    let mut s = hall(2);
    run(&mut s, "sheet", json!({"name": "Sections", "size": "A0"}));
    let first = run(&mut s, "place", json!({"sheet": "A101", "view": "Section 0", "scale": 200}));
    let second = run(&mut s, "place", json!({"sheet": "A101", "view": "Section 1", "scale": 200}));
    let x = |r: &Value| r["at"][0].as_f64().unwrap();
    let y = |r: &Value| r["at"][1].as_f64().unwrap();
    assert!((x(&first) - (20.0 + 1189.0 - 10.0) / 2.0).abs() < 1e-6, "{first}");
    assert!(x(&second) > x(&first) && (y(&second) - y(&first)).abs() < 1e-6, "{second}");
}

#[test]
fn the_3d_view_placed_with_no_scale_fits_the_paper() {
    // Given a hall 95 m long and an A1 sheet
    let mut s = Session::new();
    s.script(r#"wall {"points": [[0,0],[95000,0],[95000,40000],[0,40000]], "closed": true, "height": 12000}"#).unwrap();
    run(&mut s, "sheet", json!({"name": "3D", "size": "A1"}));
    // When the 3D view is placed with no scale
    let r = run(&mut s, "place", json!({"sheet": "A101", "view": "{3D}"}));
    // Then it comes at a scale the project offers, coarser than 1 : 100, and lies inside the border
    let scale = r["scale"].as_f64().unwrap();
    assert!(scale > 100.0 && crate::units::Units::Millimetres.scales().contains(&scale), "{r}");
    let f = on_paper(&s, r["id"].as_u64().unwrap());
    assert!(f[0] >= 20.0 && f[1] >= 10.0 && f[2] <= 831.0 && f[3] <= 584.0, "{f:?} runs off the A1 sheet");
    // A scale given is kept
    run(&mut s, "delete", json!({"ids": [r["id"]]}));
    let r = run(&mut s, "place", json!({"sheet": "A101", "view": "{3D}", "scale": 100, "at": [400, 300]}));
    assert_eq!(r["scale"], 100.0);
}

#[test]
fn a_small_models_3d_view_keeps_1_to_100() {
    let mut s = house();
    run(&mut s, "sheet", json!({"name": "3D"}));
    assert_eq!(run(&mut s, "place", json!({"sheet": "A101", "view": "{3D}"}))["scale"], 100.0);
}

// ---------- the title block ----------

/// Whether any pixel of an A3 sheet's picture between paper points `lo` and `hi` (mm) is inked.
fn inked(pm: &tiny_skia::Pixmap, lo: (f64, f64), hi: (f64, f64)) -> bool {
    let d = crate::sheet::DENSITY;
    let (x0, x1) = ((lo.0 * d).ceil() as u32, (hi.0 * d).floor() as u32);
    let (y0, y1) = (((297.0 - hi.1) * d).ceil() as u32, ((297.0 - lo.1) * d).floor() as u32);
    (x0..=x1).flat_map(|x| (y0..=y1).map(move |y| (x, y))).filter_map(|(x, y)| pm.pixel(x, y)).any(|c| c.red() < 200)
}

#[test]
fn a_long_sheet_name_stays_out_of_the_sheet_numbers_field() {
    for name in ["Grundriss EG (Bestand halbtonig / Erweiterung)".to_string(), "Erweiterung".repeat(12)] {
        // Given a sheet with a name far wider than its field
        let mut s = house();
        run(&mut s, "sheet", json!({"number": "A101", "name": name}));
        // When it is drawn
        let pm = picture(&run(&mut s, "render", json!({"view": "A101"})));
        // Then the name is lettered left of the number's field, and nothing of it crosses into it: between the field's
        // rule (50 mm in from the title box's right edge, 410 mm) and where the number begins
        let number = f64::from(crate::font::width("A101", 35.0)) / crate::sheet::DENSITY;
        assert!(inked(&pm, (234.0, 20.0), (300.0, 35.0)), "{name}: its name is lettered");
        assert!(!inked(&pm, (361.0, 11.0), (385.0 - number / 2.0 - 1.0, 35.0)), "{name}: it runs into the number's field");
    }
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
