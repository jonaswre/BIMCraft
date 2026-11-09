//! Temporary visual probe, installed and removed by capture-ui.ps1; no product source is changed permanently.
use super::*;

#[test]
fn capture_industrial_operation_choices() {
    let output = std::path::PathBuf::from(std::env::var("TASK10_UI_OUTPUT_DIR").unwrap());
    for profile in crate::style::Profile::ALL {
        for (look, dark, light_canvas) in [("light",false,false),("dark",true,false),("dark-light-canvas",true,true)] {
            let mut h = harness(&format!("{BOX}\ndoor {{\"at\":[2000,0],\"family\":\"Single Door\"}}"), 0.25, profile);
            h.set_size(vec2(1700.0,1000.0));
            h.ctx.set_theme(if dark {egui::ThemePreference::Dark} else {egui::ThemePreference::Light});
            h.state_mut().light_canvas = light_canvas;
            h.state_mut().edit_family("Single Door");
            h.run();
            pick(&mut h, Role::ComboBox, "Operation");
            h.get_by_label("Roller").scroll_to_me();
            h.run();
            assert!(h.query_by_label("Sectional").is_some() && h.query_by_label("Roller").is_some());
            let path=output.join(format!("ui-{}-{}.png",profile.code(),look));
            h.render().unwrap().save(path).unwrap();
        }
    }
}
