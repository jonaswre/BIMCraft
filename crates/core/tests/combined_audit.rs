//! Interactions between independently reviewed factory workflow contributions.
#[cfg(test)]
mod tests {
    use bimcraft_core::{
        engine::Session,
        model::{FORMAT, Model},
    };
    use serde_json::{Value, json};

    fn run(s: &mut Session, command: &str, params: Value) -> Value {
        s.execute(command, &params).unwrap_or_else(|e| panic!("{command} {params}: {e}"))
    }

    #[test]
    fn combined_native_history_advances_beyond_both_independent_format19_readers() {
        assert_eq!(FORMAT, 20, "steel19 and stair19 readers must refuse combined construction");
        for (command, params) in
            [("beam", json!({"points":[[0,0],[6000,0]],"type":"IPE 300"})), ("stair", json!({"start":[0,4000],"height":2880,"open":true}))]
        {
            let mut s = Session::new();
            run(&mut s, command, params);
            let mut old = serde_json::to_value(&s.model).unwrap();
            old["format"] = json!(19);
            let (m, left) = Model::read(&old.to_string()).unwrap();
            assert!(left.is_empty());
            assert_eq!(m.format, 20);
            assert_eq!(m.elements, s.model.elements);
            assert_eq!(m.types, s.model.types);
            assert_eq!(m.library, s.model.library);
        }
    }

    #[test]
    fn implicit_opening_prefers_any_cut_host_then_unique_default_level_raised_host() {
        let mut s = Session::new();
        let raised = run(&mut s, "wall", json!({"points":[[0,0],[8000,0]],"base_offset":6000,"height":3000}))["ids"][0].clone();
        let upper = run(&mut s, "wall", json!({"points":[[0,0],[8000,0]],"level":"Level 2","height":3000}))["ids"][0].clone();
        assert_eq!(run(&mut s, "door", json!({"at":[2000,0]}))["host"], upper);
        assert_eq!(run(&mut s, "door", json!({"at":[4000,0],"level":"Level 1"}))["host"], raised);
        let mut s = Session::new();
        let raised = run(&mut s, "wall", json!({"points":[[0,0],[8000,0]],"base_offset":6000,"height":3000}))["ids"][0].clone();
        assert_eq!(run(&mut s, "door", json!({"at":[2000,0]}))["host"], raised);
        run(&mut s, "wall", json!({"points":[[0,0],[8000,0]],"base_offset":10000,"height":3000}));
        let before = run(&mut s, "elements", json!({}));
        let error = s.execute("window", &json!({"at":[4000,0]})).unwrap_err();
        assert!(error.contains("ambiguous"), "{error}");
        assert_eq!(run(&mut s, "elements", json!({})), before);
    }

    #[test]
    fn property_boundary_level_edit_updates_warm_plans_undo_and_native_reopen() {
        use bimcraft_core::render::{self, View};
        let mut s = Session::new();
        let id = run(&mut s, "property_line", json!({"points":[[0,0],[6000,0],[6000,4000],[0,4000]]}))["id"].as_u64().unwrap();
        let levels = s.model.levels();
        let (old, new) = (levels[0].0, levels[1].0);
        let view = View { cx: 3000.0, cy: 2000.0, scale: 0.05, width: 400, height: 300 };
        let before = render::png(&render::draw(&s.model, old, view)).unwrap();
        assert!(render::draw(&s.model, old, view).owners.contains(&Some(id)));
        assert!(!render::draw(&s.model, new, view).owners.contains(&Some(id)));
        run(&mut s, "modify", json!({"id":id,"level":"Level 2"}));
        assert!(!render::draw(&s.model, old, view).owners.contains(&Some(id)));
        assert!(render::draw(&s.model, new, view).owners.contains(&Some(id)));
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("boundary.bimcraft");
        run(&mut s, "save", json!({"path":path}));
        let mut reopened = Session::new();
        run(&mut reopened, "open", json!({"path":path}));
        assert!(!render::draw(&reopened.model, old, view).owners.contains(&Some(id)));
        assert!(render::draw(&reopened.model, new, view).owners.contains(&Some(id)));
        run(&mut s, "undo", json!({}));
        assert_eq!(render::png(&render::draw(&s.model, old, view)).unwrap(), before);
        assert!(!render::draw(&s.model, new, view).owners.contains(&Some(id)));
        run(&mut s, "save", json!({"path":path}));
        run(&mut reopened, "open", json!({"path":path}));
        assert_eq!(render::png(&render::draw(&reopened.model, old, view)).unwrap(), before);
        assert_eq!(reopened.model.guid(id), s.model.guid(id));
    }

    #[test]
    fn industrial_gates_draw_in_section_and_elevation_after_native_reopen() {
        use bimcraft_core::render;
        for operation in ["sectional", "roller"] {
            let mut s = Session::new();
            run(&mut s, "wall", json!({"points":[[0,0],[12000,0]],"height":4500}));
            run(&mut s, "type", json!({"category":"door","name":"Gate","width":3000,"height":3200,"operation":operation}));
            let door = run(&mut s, "door", json!({"at":[3500,0],"type":"Gate"}))["id"].as_u64().unwrap();
            let section =
                run(&mut s, "section", json!({"start":[3500,-1000],"end":[3500,4500],"name":"Gate Section"}))["id"].as_u64().unwrap();
            run(&mut s, "elevation", json!({"facing":"south","name":"Gate South"}));
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("gate.bimcraft");
            run(&mut s, "save", json!({"path":path}));
            let original = run(&mut s, "render", json!({"view":"Gate South","width":500,"height":300}))["pngBase64"].clone();
            let cut = run(&mut s, "render", json!({"view":"Gate Section","width":500,"height":300}))["pngBase64"].clone();
            assert!(original.is_string() && cut.is_string());
            let picture = render::section_picture(&s.model, section, render::section_fit(&s.model, section, 500, 300), &[]).unwrap();
            assert!(
                (0..picture.height).any(|y| (0..picture.width).any(|x| picture.pick(f64::from(x) + 0.5, f64::from(y) + 0.5) == Some(door))),
                "gate must contribute visible section pixels: {operation}"
            );
            let mut reopened = Session::new();
            run(&mut reopened, "open", json!({"path":path}));
            assert_eq!(run(&mut reopened, "render", json!({"view":"Gate South","width":500,"height":300}))["pngBase64"], original);
            assert_eq!(run(&mut reopened, "render", json!({"view":"Gate Section","width":500,"height":300}))["pngBase64"], cut);
            run(&mut reopened, "delete", json!({"ids":[door]}));
            assert_ne!(run(&mut reopened, "render", json!({"view":"Gate South","width":500,"height":300}))["pngBase64"], original);
        }
    }
}
