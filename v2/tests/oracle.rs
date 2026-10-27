use serde_json::Value;
#[test]
fn unchanged_source_and_locked_library_clipboard_oracle() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("../oracle/cases.json")).unwrap();
    let mut compared = 0;
    for case in &cases {
        let Some(input) = case["input"].as_str() else {
            continue;
        };
        let expected = case["calls"]
            .as_array()
            .unwrap()
            .iter()
            .find(|call| call[0] == "copy");
        let actual = tezbar_markdown_this::convert(input);
        if let Some(copy) = expected {
            assert_eq!(
                actual.as_deref(),
                Ok(copy[1].as_str().unwrap()),
                "oracle case {} input {:?}",
                case["id"],
                input
            );
            compared += 1;
        } else {
            assert!(actual.is_err(), "empty oracle case {}", case["id"]);
        }
    }
    assert!(compared >= 2400);
}
#[test]
fn explicit_limits_and_javascript_whitespace() {
    assert_eq!(
        tezbar_markdown_this::convert("\u{feff}\u{00a0}\u{3000}").unwrap_err(),
        "The clipboard has no content"
    );
    assert!(tezbar_markdown_this::convert("\u{0085}").is_ok());
    assert!(
        tezbar_markdown_this::convert(&"x".repeat(tezbar_markdown_this::MAX_INPUT_BYTES + 1))
            .unwrap_err()
            .contains("limit")
    );
    assert!(
        tezbar_markdown_this::convert(&format!("{} hi", ">".repeat(65)))
            .unwrap_err()
            .contains("limit")
    );
}
