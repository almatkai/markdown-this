//! Proposed compiled coverage must agree with the shipped manifest.
use markdown_this_beta_headless::native_core::extensions::markdown_this;
#[test]
fn manifest_matches_compiled_handler_and_keeps_qa_not_run() {
    let m: serde_json::Value =
        serde_json::from_str(include_str!("../engine/extension.json")).unwrap();
    let declared: Vec<_> = m["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert_eq!(declared, markdown_this::commands());
    assert_eq!(declared, ["index"]);
    assert_eq!(m["id"], "com.tezbar.native.markdown-this");
    assert_eq!(m["replaces"], serde_json::json!(["raycast.markdown-this"]));
    assert_eq!(m["platforms"], serde_json::json!(["macOS", "Windows"]));
    assert_eq!(m["preferences"], serde_json::json!([]));
    assert!(m.get("author").is_none());
    for item in m["verification"].as_array().unwrap() {
        assert_eq!(item["state"], "not-run");
    }
}
