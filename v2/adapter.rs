//! Compiled in as a sibling of the unchanged beta v2_sdk module.
use super::v2_sdk as sdk;
use crate::native_core::common::{CoreContext, CoreResult};
use serde_json::{json, Value};

pub fn commands() -> &'static [&'static str] {
    &["index"]
}

#[derive(Clone, Copy)]
enum Platform {
    MacOs,
    Windows,
    Unsupported,
}
fn platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::MacOs
    } else if cfg!(target_os = "windows") {
        Platform::Windows
    } else {
        Platform::Unsupported
    }
}
/// A write is successful only with the beta's explicit object acknowledgement.
/// Never forward route errors: they may contain private clipboard data.
fn acknowledged(value: &Value) -> bool {
    value.is_object()
        && value.get("ok") == Some(&json!(true))
        && value.get("error").is_none_or(Value::is_null)
        && value.get("success").is_none_or(|v| v == &json!(true))
}

pub async fn handle(
    ctx: &CoreContext,
    command: &str,
    operation: &str,
    payload: &Value,
    state: &mut Value,
) -> CoreResult {
    handle_on(ctx, command, operation, payload, state, platform()).await
}
async fn handle_on(
    ctx: &CoreContext,
    command: &str,
    operation: &str,
    _payload: &Value,
    state: &mut Value,
    platform: Platform,
) -> CoreResult {
    if command != "index" {
        return Err("Unknown Markdown This command".into());
    }
    match operation {
        "extension:refresh-session" | "extension:search-text-changed" | "extension:load-more" => {
            Ok(sdk::unchanged())
        }
        "extension:dispose-session" => {
            *state = json!({"stage":"disposed"});
            Ok(sdk::unchanged())
        }
        "extension:invoke-action" => Err("Markdown This has no interactive actions".into()),
        "extension:run-command" => {
            if matches!(platform, Platform::Unsupported) {
                return Err("Markdown This Native V2 supports macOS and Windows".into());
            }
            if !state.is_object() {
                return Err("Invalid Markdown This command state".into());
            }
            if state.get("stage").is_some() {
                return Err("This command already ran or was interrupted; launch a new command to convert again".into());
            }
            state["stage"] = json!("reading");
            let read = ctx
                .request("clipboard:read", vec![])
                .await
                .map_err(|_| "Could not read the clipboard".to_string())?;
            let text = read
                .as_str()
                .ok_or("Clipboard returned an invalid text response")?
                .to_owned();
            state["stage"] = json!("converting");
            let html = tokio::task::spawn_blocking(move || tezbar_markdown_this::convert(&text))
                .await
                .map_err(|_| "Native Markdown conversion did not complete".to_string())??;
            // Record the attempted non-idempotent effect before awaiting it.
            // Even a transport error may mean the clipboard was changed.
            state["stage"] = json!("copy-attempted");
            let result=sdk::copy(ctx,&html).await.map_err(|_|"Could not confirm the clipboard write; it may already have changed. Launch a new command to try again".to_string())?;
            if !acknowledged(&result) {
                return Err("Clipboard write was not acknowledged; it may already have changed. Launch a new command to try again".into());
            }
            state["stage"] = json!("completed");
            Ok(sdk::no_view(
                "Conversion successful — HTML copied to clipboard",
            ))
        }
        _ => Err("Unsupported Markdown This operation".into()),
    }
}

#[cfg(test)]
#[path = "harness/adapter_tests.rs"]
mod tests;
