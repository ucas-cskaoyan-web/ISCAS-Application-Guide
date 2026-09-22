use leptos::prelude::*;

use crate::app::{ThemeContext, TranslationContext};

#[component]
pub fn ThemeToggle() -> impl IntoView {
    let theme_ctx = expect_context::<ThemeContext>();
    let translator = expect_context::<TranslationContext>();
    let is_dark = theme_ctx.0; // is_dark_mode signal

    let toggle_theme = move |_| {
        let current_is_dark = is_dark.get();
        is_dark.set(!current_is_dark);
    };

    let thumb_classes = move || {
        if is_dark.get() {
            ("theme-toggle-thumb-dark", "theme-toggle-icon-dark")
        } else {
            ("theme-toggle-thumb-light", "theme-toggle-icon-light")
        }
    };

    view! {
        <button
            attr:aria-label=translator.translate("Toggle theme")
            aria-pressed=move || is_dark.get().to_string()
            class="theme-toggle-button"
            on:click=toggle_theme
        >
            <div class=move || {
                let (thumb_position, _) = thumb_classes();
                format!("theme-toggle-thumb {thumb_position}")
            }>
                // Light Mode Icon
                <span class=move || {
                    let current_is_dark = is_dark.get();
                    format!(
                        "theme-toggle-icon-container {}",
                        if current_is_dark {
                            "theme-toggle-icon-hidden"
                        } else {
                            "theme-toggle-icon-visible"
                        },
                    )
                }>
                    <span class=move || {
                        let (_, thumb_icon_color) = thumb_classes();
                        format!("material-symbols-outlined theme-toggle-icon {}", thumb_icon_color)
                    }>"light_mode"</span>
                </span>

                // Dark Mode Icon
                <span class=move || {
                    let current_is_dark = is_dark.get();
                    format!(
                        "theme-toggle-icon-container {}",
                        if current_is_dark {
                            "theme-toggle-icon-visible"
                        } else {
                            "theme-toggle-icon-hidden"
                        },
                    )
                }>
                    <span class=move || {
                        let (_, thumb_icon_color) = thumb_classes();
                        format!("material-symbols-outlined theme-toggle-icon {}", thumb_icon_color)
                    }>"dark_mode"</span>
                </span>
            </div>
        </button>
    }
}
