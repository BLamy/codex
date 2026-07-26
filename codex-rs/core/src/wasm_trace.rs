#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(message: &str);
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn stage(message: &str) {
    if trace_enabled() {
        console_error(&format!("[codex-core wasm] {message}"));
    }
}

#[cfg(target_arch = "wasm32")]
fn trace_enabled() -> bool {
    js_sys::Reflect::get(
        &js_sys::global(),
        &JsValue::from_str("__ALMOSTNODE_CODEX_WASM_TRACE"),
    )
    .ok()
    .and_then(|value| value.as_bool())
    .unwrap_or(false)
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn stage(_message: &str) {}
