use log::warn;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

/// JavaScript binding declarations
#[wasm_bindgen(module = "/src/scripts/highlighter.js")]
extern "C" {
    #[wasm_bindgen(js_name = highlightCode)]
    fn highlight_code_inner(input: &str, lang: &str) -> JsValue;
}

pub fn highlight_code(code: &str, lang: &str) -> String {
    highlight_code_inner(code, lang)
        .as_string()
        .unwrap_or_else(|| {
            warn!("Failed to convert JsValue to String when highlighting code");
            String::from("<div class=\"error\">Error highlighting code</div>")
        })
}
