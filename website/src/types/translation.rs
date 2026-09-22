use std::{collections::HashMap, sync::Arc};

use gloo_net::http::Request;

use super::site::public_url;

/// Runtime translations keyed by the canonical English UI copy.
///
/// English is intentionally kept as the source of truth: a missing key, a
/// malformed file, or an unavailable translation asset all fall back to the
/// key itself so the interface remains usable.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Translator {
    entries: Arc<HashMap<String, String>>,
}

impl Translator {
    pub fn from_entries(entries: HashMap<String, String>) -> Self {
        Self {
            entries: Arc::new(entries),
        }
    }

    pub async fn fetch() -> Self {
        let Ok(response) = Request::get(&public_url("translation.json")).send().await else {
            return Self::default();
        };
        let Ok(text) = response.text().await else {
            return Self::default();
        };
        let Ok(entries) = serde_json_wasm::from_str::<HashMap<String, String>>(&text) else {
            return Self::default();
        };

        Self::from_entries(entries)
    }

    pub fn translate(&self, english: &str) -> String {
        self.entries
            .get(english)
            .cloned()
            .unwrap_or_else(|| english.to_string())
    }

    pub fn translate_template(&self, english: &str, replacements: &[(&str, &str)]) -> String {
        let mut translated = self.translate(english);
        for (name, value) in replacements {
            translated = translated.replace(&format!("{{{name}}}"), value);
        }
        translated
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::Translator;

    #[test]
    fn missing_keys_fall_back_to_english() {
        let translator =
            Translator::from_entries(HashMap::from([("Home".to_string(), "首页".to_string())]));

        assert_eq!(translator.translate("Home"), "首页");
        assert_eq!(translator.translate("Articles"), "Articles");
    }

    #[test]
    fn templates_replace_named_placeholders() {
        let translator = Translator::from_entries(HashMap::from([(
            "Page {path} not found".to_string(),
            "找不到页面：{path}".to_string(),
        )]));

        assert_eq!(
            translator.translate_template("Page {path} not found", &[("path", "/missing")]),
            "找不到页面：/missing"
        );
    }
}
