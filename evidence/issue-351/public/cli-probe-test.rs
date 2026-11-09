// Temporary external-binary probe appended to crates/ui-egui/src/tests/control.rs.
// The committed UI tests exercise real canvas actions and TCP without a CLI dependency.
// This probe additionally runs the separately built, actual no-default-feature CLI binary.
#[test]
fn actual_cli_mcp_reads_desktop_context_and_reports_headless_unavailability() {
    use std::process::{Command, Stdio};
    use std::time::Instant;
    fn run(h: &mut Harness<BimApp>, bridge: &std::path::Path, connected: bool, messages: Vec<Value>) -> Vec<Value> {
        let cli = std::env::var("ISSUE23_CLI").unwrap();
        let favorites = bridge.with_file_name("favorites.json");
        let port = entry(h).port.to_string();
        let task = std::thread::spawn(move || {
            let mut command = Command::new(cli);
            command.args(["mcp", "--read-only"]);
            if connected { command.args(["--connect", &port]); }
            let mut child = command.env("BIMCRAFT_FAVORITES", favorites).env("BIMCRAFT_PLUGINS", "")
                .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
            let mut input = child.stdin.take().unwrap();
            for message in messages { writeln!(input, "{message}").unwrap(); }
            drop(input);
            let output = child.wait_with_output().unwrap();
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
            String::from_utf8(output.stdout).unwrap().lines().map(|s| serde_json::from_str(s).unwrap()).collect::<Vec<Value>>()
        });
        let start = Instant::now();
        while !task.is_finished() {
            assert!(start.elapsed() < Duration::from_secs(30), "CLI probe timed out");
            h.step();
            std::thread::sleep(Duration::from_millis(2));
        }
        task.join().unwrap()
    }
    let (mut h, bridge) = controlled("actual-cli-context");
    click(&mut h, P2(4000.0, 0.0));
    let selected = h.state().selection[0];
    let kept = Arc::clone(&h.state().session.model);
    let messages = vec![
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"desktop_context","arguments":{}}}),
        json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"execute","arguments":{"command":"wall","params":{"points":[[0,10000],[4000,10000]]}}}}),
    ];
    let replies = run(&mut h, &bridge, true, messages);
    assert!(replies[0]["result"]["instructions"].as_str().unwrap().contains("desktop_context"));
    assert!(replies[1]["result"]["tools"].as_array().unwrap().iter().any(|t|t["name"]=="desktop_context"));
    let context: Value = serde_json::from_str(replies[2]["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(context["selection"][0]["id"], json!(selected));
    assert_eq!(context["selection"][0]["guid"], json!(h.state().session.model.guid(selected)));
    assert_eq!(context["active_view"]["kind"], "plan");
    assert_eq!(replies[3]["result"]["isError"], true);
    assert!(replies[3]["result"]["content"][0]["text"].as_str().unwrap().contains("read-only"));
    assert!(Arc::ptr_eq(&kept, &h.state().session.model));
    println!("CONNECTED_CONTEXT={context}");
    println!("READ_ONLY_REFUSAL={}", replies[3]);
    browse(&mut h, "Room Schedule");
    let message = json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"desktop_context","arguments":{}}});
    let replies = run(&mut h, &bridge, true, vec![message.clone()]);
    let context: Value = serde_json::from_str(replies[0]["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(context["active_view"]["kind"], "schedule");
    println!("NAVIGATED_CONTEXT={context}");
    let replies = run(&mut h, &bridge, false, vec![message]);
    assert_eq!(replies[0]["result"]["isError"], true);
    assert_eq!(replies[0]["result"]["content"][0]["text"], bimcraft_core::bridge::NO_CONTEXT);
    println!("HEADLESS_UNAVAILABLE={}", replies[0]);
    h.state_mut().control = None;
    let output = Command::new(std::env::var("ISSUE23_CLI").unwrap()).args(["mcp", "--connect"])
        .env("BIMCRAFT_FAVORITES", bridge.with_file_name("favorites.json")).env("BIMCRAFT_PLUGINS", "").output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("no window has opened a control port"));
    println!("NO_WINDOW=connection refused with existing no-control-port diagnostic");
}
