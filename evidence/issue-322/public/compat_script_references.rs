//! Temporary integration evidence for unmerged PR287 plus named script references.
#[cfg(test)]
mod tests {
    use bimcraft_core::bridge::{self, Access, Request};
    use bimcraft_core::engine::Session;
    use serde_json::json;

    #[test]
    fn named_dry_run_previews_hosted_geometry_and_keeps_model_ids_and_history() {
        let mut s = Session::new();
        s.read_only = Some("preview only");
        let before = serde_json::to_value(&s.model).unwrap();
        let text = "@walls = wall {\"points\":[[0,0],[8000,0],[8000,6000]]}\ndoor {\"wall\":\"@walls.ids.1\",\"offset\":3000}";
        let preview = s.script_dry_run(text).unwrap();
        assert_eq!(preview["made"].as_array().unwrap().len(), 3);
        assert!(preview["results"][1]["id"].is_number());
        assert_eq!(serde_json::to_value(&s.model).unwrap(), before);
        assert!(s.history().is_empty());
        assert_eq!(s.read_only, Some("preview only"));
        for access in [Access::Read, Access::Edit] {
            let request = Request::Script { text: text.into(), atomic: true, dry_run: true, read_only: false };
            assert_eq!(bridge::run(&mut s, &request, access).unwrap()["made"].as_array().unwrap().len(), 3);
            assert_eq!(serde_json::to_value(&s.model).unwrap(), before);
        }
    }

    #[test]
    fn named_file_commands_are_refused_before_creating_files_or_mutating() {
        for command in ["save", "render"] {
            let dir = tempfile::tempdir().unwrap();
            let file = dir.path().join(if command == "save" { "forbidden.bimcraft" } else { "forbidden.png" });
            let mut s = Session::new();
            let before = serde_json::to_value(&s.model).unwrap();
            let text = format!("grid {{\"start\":[0,0],\"end\":[0,4000]}}\n@output = {command} {}", json!({"path":file}));
            let result = s.script_dry_run(&text);
            assert!(!file.exists(), "named {command} wrote a file during dry_run: {result:?}");
            let error = result.unwrap_err();
            assert!(error.starts_with("line 2") && error.contains("dry run touches no file"), "{error}");
            assert_eq!(serde_json::to_value(&s.model).unwrap(), before);
            assert!(s.history().is_empty());
        }
    }

    #[test]
    fn named_replacements_are_refused_before_any_line_executes_in_a_dry_run() {
        let mut s = Session::new();
        let before = serde_json::to_value(&s.model).unwrap();
        let text = "grid {\"start\":[0,0],\"end\":[0,4000]}\n@project = new";
        let result = s.script_dry_run(text);
        assert!(result.is_err(), "named new bypassed dry_run preflight: {result:?}");
        assert_eq!(serde_json::to_value(&s.model).unwrap(), before);
        for access in [Access::Read, Access::Edit] {
            let request = Request::Script { text: text.into(), atomic: false, dry_run: true, read_only: false };
            let error = bridge::run(&mut s, &request, access).unwrap_err();
            assert!(error.starts_with("line 2") && error.contains("replace the project"), "{error}");
            assert_eq!(serde_json::to_value(&s.model).unwrap(), before);
            assert!(s.history().is_empty());
        }
    }
}
