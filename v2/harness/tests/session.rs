//! Exercises unchanged beta native.rs (library compiled without its unrelated unit tests).
use beta_session_fixture::native_core::{
    common::{CoreContext, CorePaths},
    extensions::SessionHarness,
};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
#[tokio::test]
async fn actual_beta_no_view_dispatch_rejects_stale_ids_without_os_effects() {
    let root = tempfile::tempdir().unwrap();
    let ctx = CoreContext::new(
        CorePaths {
            data_dir: root.path().join("data"),
            config_dir: root.path().join("config"),
            resource_dir: root.path().join("resources"),
            temp_dir: root.path().join("tmp"),
            home_dir: root.path().to_owned(),
        },
        None,
    );
    let history = Arc::new(Mutex::new(Vec::new()));
    let calls = history.clone();
    ctx.set_router(Arc::new(move |route, args| {
        calls.lock().unwrap().push((route.clone(), args));
        Box::pin(async move {
            match route.as_str() {
                "clipboard:read" => Ok(json!("# Real session")),
                "clipboard:write" => Ok(json!({"ok":true})),
                _ => panic!("Unexpected OS route {route}"),
            }
        })
    }))
    .unwrap();
    let session = SessionHarness::default();
    let body = json!({"extensionId":"com.tezbar.native.markdown-this","commandName":"index","argumentValues":{}});
    let result = session
        .dispatch(&ctx, "extension:run-command", body.clone())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result["mode"], "no-view");
    assert!(result.get("sessionId").is_none());
    assert_eq!(history.lock().unwrap().len(), 2);
    for route in [
        "extension:run-command",
        "extension:invoke-action",
        "extension:refresh-session",
        "extension:search-text-changed",
        "extension:load-more",
    ] {
        let mut stale = body.clone();
        stale["sessionId"] = json!("native-extension:forged");
        stale["actionId"] = json!("forged");
        assert_eq!(
            session.dispatch(&ctx, route, stale).await.unwrap().unwrap()["ok"],
            false
        );
    }
    assert_eq!(history.lock().unwrap().len(), 2);
    assert_eq!(
        session
            .dispatch(
                &ctx,
                "extension:dispose-session",
                json!({"sessionId":"native-extension:forged"})
            )
            .await
            .unwrap()
            .unwrap(),
        Value::Bool(false)
    );
    let mut wrong = body;
    wrong["commandName"] = json!("invented");
    assert_eq!(
        session
            .dispatch(&ctx, "extension:run-command", wrong)
            .await
            .unwrap()
            .unwrap()["ok"],
        false
    );
    session.clear_slug().await;
    session.clear().await;
    assert_eq!(history.lock().unwrap().len(), 2);
}
