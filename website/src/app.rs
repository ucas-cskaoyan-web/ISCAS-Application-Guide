use leptos::{prelude::*, task::spawn_local};
use leptos_meta::provide_meta_context;
use once_cell::sync::OnceCell;

use crate::{
    router::AppRouter,
    types::{site::Site, translation::Translator},
};

pub static SITE_CONFIGURATION: OnceCell<Site> = OnceCell::new();

#[derive(Clone, Copy, PartialEq)]
pub struct ThemeContext(pub RwSignal<bool>); // true for dark mode

#[derive(Clone, Copy, PartialEq)]
pub struct TranslationContext(pub RwSignal<Translator>);

impl TranslationContext {
    pub fn translate(&self, english: &str) -> String {
        self.0.get_untracked().translate(english)
    }

    pub fn translate_template(&self, english: &str, replacements: &[(&str, &str)]) -> String {
        self.0
            .get_untracked()
            .translate_template(english, replacements)
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Read saved theme state from localStorage, use system preference as
    // fallback
    let is_dark_mode = RwSignal::new({
        let window = web_sys::window().expect("no global `window` exists");
        let storage = window.local_storage().unwrap().unwrap();

        // First try to read from localStorage
        if let Ok(Some(saved_theme)) = storage.get_item("theme") {
            saved_theme == "dark"
        } else {
            // If no saved theme exists, use system preference
            window
                .match_media("(prefers-color-scheme: dark)")
                .ok()
                .flatten()
                .is_some_and(|mql| mql.matches())
        }
    });
    provide_context(ThemeContext(is_dark_mode));

    let translations = RwSignal::new(Translator::default());
    let translation_ready = RwSignal::new(false);
    provide_context(TranslationContext(translations));
    spawn_local(async move {
        translations.set(Translator::fetch().await);
        translation_ready.set(true);
    });

    // Set body class for global styling
    Effect::new(move |_| {
        let body = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap();
        let document = web_sys::window().unwrap().document().unwrap();
        let root = document.document_element().unwrap();

        let current_is_dark = is_dark_mode.get();

        // Save theme state to localStorage
        let window = web_sys::window().unwrap();
        let storage = window.local_storage().unwrap().unwrap();
        let theme_value = if current_is_dark { "dark" } else { "light" };
        let _ = storage.set_item("theme", theme_value);
        let _ = root.set_attribute("data-theme", theme_value);

        if current_is_dark {
            body.class_list().add_1("dark").unwrap();
            body.class_list().remove_1("light").unwrap();
        } else {
            body.class_list().add_1("light").unwrap();
            body.class_list().remove_1("dark").unwrap();
        }
    });

    view! {
        <Show
            when=move || translation_ready.get()
            fallback=|| view! { <div class="app-loading"></div> }
        >
            <AppRouter />
        </Show>
    }
}
