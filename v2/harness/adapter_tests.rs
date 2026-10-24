use super::*;
use crate::native_core::common::CorePaths;
use std::sync::{Arc, Mutex};

type History = Arc<Mutex<Vec<(String, Vec<Value>)>>>;

fn context(response: Result<Value, String>, read: Value) -> (CoreContext, History) {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let capture = calls.clone();
    let ctx = CoreContext::new(
        CorePaths {
            data_dir: "unused-data".into(),
            config_dir: "unused-config".into(),
            resource_dir: "unused-resources".into(),
            temp_dir: "unused-temp".into(),
            home_dir: "unused-home".into(),
        },
        None,
    );
    ctx.set_router(Arc::new(move |route, args| {
        capture.lock().unwrap().push((route.clone(), args.clone()));
        let read = read.clone();
        let response = response.clone();
        Box::pin(async move {
            match route.as_str() {
                "clipboard:read" => {
                    assert!(args.is_empty());
                    Ok(read)
                }
                "clipboard:write" => {
                    assert_eq!(args.len(), 1);
                    assert!(args[0].as_str().unwrap().contains("<!DOCTYPE html>"));
                    response
                }
                _ => panic!("Forbidden OS/focus/browser/selection route {route}"),
            }
        })
    }))
    .unwrap();
    (ctx, calls)
}
#[tokio::test]
async fn both_platforms_use_only_exact_clipboard_routes_and_plain_html() {
    for platform in [Platform::MacOs, Platform::Windows] {
        let (ctx, calls) = context(Ok(json!({"ok":true})), json!("# Қазақ 😀\nnext"));
        let mut state = json!({});
        let result = handle_on(
            &ctx,
            "index",
            "extension:run-command",
            &json!({"preferences":{"preferences":{}},"argumentValues":{}}),
            &mut state,
            platform,
        )
        .await
        .unwrap();
        assert_eq!(result["mode"], "no-view");
        assert!(result.get("sessionId").is_none());
        assert_eq!(state["stage"], "completed");
        let history = calls.lock().unwrap().clone();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0], ("clipboard:read".into(), vec![]));
        assert_eq!(
            history[1],
            (
                "clipboard:write".into(),
                vec![json!(
                    tezbar_markdown_this::convert("# Қазақ 😀\nnext").unwrap()
                )]
            )
        );
        assert!(handle_on(
            &ctx,
            "index",
            "extension:run-command",
            &json!({}),
            &mut state,
            platform
        )
        .await
        .is_err());
        assert_eq!(calls.lock().unwrap().len(), 2);
    }
}
#[tokio::test]
async fn every_bad_or_uncertain_ack_fails_and_cannot_retry() {
    let bad = vec![
        Value::Null,
        json!(true),
        json!(false),
        json!([]),
        json!({}),
        json!({"ok":false}),
        json!({"ok":1}),
        json!({"ok":"true"}),
        json!({"ok":true,"error":"private input"}),
        json!({"ok":true,"error":false}),
        json!({"ok":true,"success":false}),
        json!({"ok":true,"success":null}),
        json!({"success":true}),
    ];
    for platform in [Platform::MacOs, Platform::Windows] {
        for result in bad
            .iter()
            .cloned()
            .map(Ok)
            .chain([Err("private transport clipboard contents".into())])
        {
            let (ctx, calls) = context(result, json!("hello"));
            let mut state = json!({});
            let error = handle_on(
                &ctx,
                "index",
                "extension:run-command",
                &json!({}),
                &mut state,
                platform,
            )
            .await
            .unwrap_err();
            assert!(!error.contains("private"));
            assert_eq!(state["stage"], "copy-attempted");
            assert_eq!(calls.lock().unwrap().len(), 2);
            assert!(handle_on(
                &ctx,
                "index",
                "extension:run-command",
                &json!({}),
                &mut state,
                platform
            )
            .await
            .is_err());
            assert_eq!(calls.lock().unwrap().len(), 2);
        }
    }
    for ack in [
        json!({"ok":true}),
        json!({"ok":true,"error":null,"success":true}),
    ] {
        assert!(acknowledged(&ack));
    }
}
#[tokio::test]
async fn malformed_read_empty_and_resource_errors_never_write() {
    for input in [
        Value::Null,
        json!(false),
        json!({"text":"hi"}),
        json!(""),
        json!(" \n\u{feff}"),
        json!("a".repeat(tezbar_markdown_this::MAX_INPUT_BYTES + 1)),
    ] {
        let (ctx, calls) = context(Ok(json!({"ok":true})), input);
        let mut state = json!({});
        assert!(handle_on(
            &ctx,
            "index",
            "extension:run-command",
            &json!({}),
            &mut state,
            Platform::MacOs
        )
        .await
        .is_err());
        assert_eq!(calls.lock().unwrap().len(), 1);
    }
    let (ctx, calls) = context(Ok(json!({"ok":true})), json!("hi"));
    let mut state = json!({});
    assert!(handle_on(
        &ctx,
        "index",
        "extension:run-command",
        &json!({}),
        &mut state,
        Platform::Unsupported
    )
    .await
    .is_err());
    assert!(calls.lock().unwrap().is_empty());
    assert!(handle_on(
        &ctx,
        "wrong",
        "extension:run-command",
        &json!({}),
        &mut state,
        Platform::MacOs
    )
    .await
    .is_err());
    assert!(handle_on(
        &ctx,
        "index",
        "extension:invoke-action",
        &json!({"actionId":"forged"}),
        &mut state,
        Platform::MacOs
    )
    .await
    .is_err());
    assert!(handle_on(
        &ctx,
        "index",
        "invented",
        &json!({}),
        &mut state,
        Platform::MacOs
    )
    .await
    .is_err());
    assert!(calls.lock().unwrap().is_empty());
}
#[tokio::test]
async fn lifecycle_refreshes_do_not_mint_state_or_perform_effects() {
    let (ctx, calls) = context(Ok(json!({"ok":true})), json!("hi"));
    let state = json!({"stage":"copy-attempted","unknown":{"keep":true}});
    for operation in [
        "extension:refresh-session",
        "extension:search-text-changed",
        "extension:load-more",
    ] {
        let mut cloned = state.clone();
        assert_eq!(
            handle_on(
                &ctx,
                "index",
                operation,
                &json!({}),
                &mut cloned,
                Platform::Windows
            )
            .await
            .unwrap(),
            sdk::unchanged()
        );
        assert_eq!(cloned, state);
    }
    let mut state = json!({});
    handle_on(
        &ctx,
        "index",
        "extension:dispose-session",
        &json!({}),
        &mut state,
        Platform::Windows,
    )
    .await
    .unwrap();
    assert!(handle_on(
        &ctx,
        "index",
        "extension:run-command",
        &json!({}),
        &mut state,
        Platform::Windows
    )
    .await
    .is_err());
    assert!(calls.lock().unwrap().is_empty());
}
#[tokio::test]
async fn read_transport_errors_are_sanitized_and_not_retried() {
    let (ctx, _) = context(Ok(json!({"ok":true})), json!("hi"));
    let ctx = CoreContext::new(ctx.paths.clone(), None);
    let count = Arc::new(Mutex::new(0));
    let c = count.clone();
    ctx.set_router(Arc::new(move |route, args| {
        assert_eq!(route, "clipboard:read");
        assert!(args.is_empty());
        *c.lock().unwrap() += 1;
        Box::pin(async { Err("PRIVATE".into()) })
    }))
    .unwrap();
    let mut state = json!({});
    let e = handle_on(
        &ctx,
        "index",
        "extension:run-command",
        &json!({}),
        &mut state,
        Platform::MacOs,
    )
    .await
    .unwrap_err();
    assert_eq!(e, "Could not read the clipboard");
    assert!(handle_on(
        &ctx,
        "index",
        "extension:run-command",
        &json!({}),
        &mut state,
        Platform::MacOs
    )
    .await
    .is_err());
    assert_eq!(*count.lock().unwrap(), 1);
}
#[tokio::test]
async fn cancelled_uncertain_write_keeps_checkpoint() {
    let (ctx, _) = context(Ok(json!({"ok":true})), json!("hi"));
    let ctx = CoreContext::new(ctx.paths.clone(), None);
    ctx.set_router(Arc::new(move |route, _| {
        Box::pin(async move {
            if route == "clipboard:read" {
                Ok(json!("hello"))
            } else {
                assert_eq!(route, "clipboard:write");
                std::future::pending().await
            }
        })
    }))
    .unwrap();
    let mut state = json!({});
    let result = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        handle_on(
            &ctx,
            "index",
            "extension:run-command",
            &json!({}),
            &mut state,
            Platform::Windows,
        ),
    )
    .await;
    assert!(result.is_err());
    assert_eq!(state["stage"], "copy-attempted");
    assert!(handle_on(
        &ctx,
        "index",
        "extension:run-command",
        &json!({}),
        &mut state,
        Platform::Windows
    )
    .await
    .is_err());
}
