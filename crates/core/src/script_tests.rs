//! Scripts that point at what they made: a line named `@name = command {…}`, and `@name`, `$last` and `$-n` in a later
//! line's params standing for what that line returned. Driven as the CLI's `--script`, MCP's `script` and the window's
//! control port drive them, through `Session::script`.

use serde_json::json;

use crate::engine::Session;
use crate::model::Element;

/// A hall with a mono-pitch roof, and one wall attached to it by name, in one script.
const HALL: &str = r#"
level {"elevation": 6000, "name": "Dach"}
@hall = roof {"level": "Dach", "boundary": [[0,0],[10000,0],[10000,20000],[0,20000]], "slope": 10, "gables": [0,1,2], "overhang": 0}
wall {"points": [[0,0],[10000,0]], "height": 3000, "attach_top": "@hall"}
"#;

#[test]
fn a_wall_attaches_to_a_roof_made_earlier_in_the_same_script_by_its_name() {
    let mut s = Session::new();
    let out = s.script(HALL).unwrap();
    let roof = out[1]["id"].as_u64().unwrap();
    let wall = out[2]["ids"][0].as_u64().unwrap();
    let Some(Element::Wall(w)) = s.model.elements.get(wall) else { panic!("no wall {wall}") };
    assert!(w.attach_top.is_some_and(|g| s.model.same(g, roof)), "the wall's top follows roof {roof}");
}

#[test]
fn last_and_back_references_stand_for_the_lines_before() {
    let mut s = Session::new();
    let out = s
        .script(
            "grid {\"start\": [0,0], \"end\": [0,10000], \"name\": \"A\"}\n\
             # a comment is no line to count back over\n\
             grid {\"start\": [5000,0], \"end\": [5000,10000], \"name\": \"B\"}\n\
             modify {\"id\": \"$last\", \"name\": \"B1\"}\n\
             modify {\"id\": \"$-3\", \"name\": \"A1\"}",
        )
        .unwrap();
    let names: Vec<(u64, String)> = s
        .model
        .elements
        .iter()
        .filter_map(|(id, e)| match e {
            Element::Grid(g) => Some((*id, g.name.clone())),
            _ => None,
        })
        .collect();
    let (a, b) = (out[0]["id"].as_u64().unwrap(), out[1]["id"].as_u64().unwrap());
    assert_eq!(names, vec![(a, "A1".to_string()), (b, "B1".to_string())]);
}

#[test]
fn a_reference_names_a_field_of_what_a_line_returned() {
    let mut s = Session::new();
    // `wall` returns its walls as `ids`: the second of them is `@box.ids.1`.
    let out = s
        .script(
            "@box = wall {\"points\": [[0,0],[8000,0],[8000,6000],[0,6000]], \"closed\": true}\n\
             door {\"wall\": \"@box.ids.1\", \"offset\": 3000}",
        )
        .unwrap();
    let second = &out[0]["ids"][1];
    let host = s.execute("elements", &json!({"category": "doors"})).unwrap()["elements"][0]["host"].clone();
    assert_eq!(serde_json::from_value::<crate::model::Guid>(host).ok().and_then(|g| s.model.elements.id(g)), second.as_u64());
}

#[test]
fn a_reference_to_nothing_stops_the_script_at_its_line_and_says_why() {
    let mut s = Session::new();
    let e = s.script("grid {\"start\": [0,0], \"end\": [0,10000]}\nmodify {\"id\": \"@roof\", \"name\": \"X\"}").unwrap_err();
    assert!(e.starts_with("line 2") && e.contains("@roof") && e.contains("no line before it is named"), "{e}");
    let e = s.script("modify {\"id\": \"$-5\", \"name\": \"X\"}").unwrap_err();
    assert!(e.starts_with("line 1") && e.contains("$-5"), "{e}");
    let e = s.script("@g = grid {\"start\": [0,0], \"end\": [0,10000]}\nmodify {\"id\": \"@g.nope\", \"name\": \"X\"}").unwrap_err();
    assert!(e.starts_with("line 2") && e.contains("nope"), "{e}");
    let e = s.script("@g = grid {\"start\": [0,0], \"end\": [0,10000]}\n@g = grid {\"start\": [1,0], \"end\": [1,10000]}").unwrap_err();
    assert!(e.starts_with("line 2") && e.contains("@g") && e.contains("already"), "{e}");
    let e = s.script("@ = grid {\"start\": [0,0], \"end\": [0,10000]}").unwrap_err();
    assert!(e.starts_with("line 1") && e.contains("name"), "{e}");
}

#[test]
fn a_text_that_starts_with_at_or_dollar_is_written_doubled() {
    let mut s = Session::new();
    let out = s
        .script(
            "text {\"text\": \"@@home\", \"at\": [0, 0]}\n\
             text {\"text\": \"$$last\", \"at\": [0, 0]}\n\
             text {\"text\": \"a @b $c $5\", \"at\": [0, 0]}",
        )
        .unwrap();
    let text = |i: usize| match s.model.elements.get(out[i]["id"].as_u64().unwrap()) {
        Some(Element::Text(n)) => n.text.clone(),
        _ => panic!("no note"),
    };
    assert_eq!((text(0).as_str(), text(1).as_str()), ("@home", "$last"), "a doubled @ or $ at the start is the character itself");
    assert_eq!(text(2), "a @b $c $5", "only a whole string is a reference");
}

#[test]
fn an_atomic_script_with_references_is_one_undo_step() {
    let mut s = Session::new();
    s.script_atomic(HALL).unwrap();
    assert_eq!(s.history().last(), Some(&"Script"));
    let e = s.script_atomic("@r = level {\"elevation\": 9000}\nmodify {\"id\": \"@x\", \"name\": \"X\"}").unwrap_err();
    assert!(e.contains("@x"), "{e}");
    assert!(!s.model.levels().iter().any(|(_, l)| l.elevation == 9000.0), "the failed script took back its first line");
}

#[test]
fn a_named_line_that_would_replace_the_project_is_refused_through_the_control_port() {
    let mut s = Session::new();
    let request = crate::bridge::Request::Script { text: "level {\"elevation\": 6000}\n@p = new".into(), atomic: false };
    let e = crate::bridge::run(&mut s, &request).unwrap_err();
    assert!(e.starts_with("line 2") && e.contains("`new`"), "{e}");
    assert_eq!(s.model.levels().len(), 2, "no line of it ran");
}

#[test]
fn the_readme_s_example_of_a_gable_attached_to_its_roof_runs() {
    let mut s = Session::new();
    let out = s
        .script(
            r#"@gable = wall {"points": [[19290,0],[0,0]], "height": 7000}
level {"elevation": 7200, "name": "Roof"}
@hall = roof {"level": "Roof", "boundary": [[0,0],[19290,0],[19290,40480],[0,40480]], "slope": 5, "gables": [0,1,2]}
modify {"id": "@gable.ids.0", "attach_top": "@hall"}"#,
        )
        .unwrap();
    let Some(Element::Wall(w)) = s.model.elements.get(out[0]["ids"][0].as_u64().unwrap()) else { panic!("no wall") };
    assert!(w.attach_top.is_some_and(|g| s.model.same(g, out[2]["id"].as_u64().unwrap())));
}
