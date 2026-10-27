//! HEADLESS STUB: type compatibility only. No window, OS or event side effects.
#[derive(Clone)]
pub struct AppHandle;
pub trait Emitter {
    fn emit(&self, _channel: &str, _value: serde_json::Value) -> Result<(), String> {
        Ok(())
    }
}
impl Emitter for AppHandle {}
