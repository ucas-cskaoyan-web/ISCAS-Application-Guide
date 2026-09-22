use leptos::{attr::global::ClassAttribute, prelude::*};
use leptos_router::components::A;

use crate::{app::TranslationContext, components::progress_bar::stop_progress_bar};

#[component]
pub fn ErrorPage(
    title: String,
    message: String,
    #[prop(optional)] error_details: Option<String>,
    #[prop(optional)] on_retry: Option<Callback<()>>,
    #[prop(default = "error".into())] error_type: String,
    #[prop(default = false)] show_navigation: bool,
) -> impl IntoView {
    stop_progress_bar();
    let translator = expect_context::<TranslationContext>();

    let (icon, icon_color_class) = match error_type.as_str() {
        "404" => ("unknown_document", "error-page-icon-404"),
        "500" => ("error", "error-page-icon-500"),
        "network" => ("wifi_off", "error-page-icon-network"),
        _ => ("error", "error-page-icon-default"),
    };

    view! {
        // Main container for the error page
        <div class="error-page-container">
            // Content box
            <div class="error-page-content">
                // Icon
                <div class="error-page-icon-container">
                    <span class=format!(
                        "material-symbols-outlined error-page-icon {}",
                        icon_color_class,
                    )>{icon}</span>
                </div>

                // Title and Message
                <h2 class="error-page-title">{title}</h2>
                <p class="error-page-message">{message}</p>

                // Error Details (collapsible)
                {error_details
                    .map(|details| {
                        view! {
                            <details class="error-page-details">
                                <summary class="error-page-details-summary">
                                    {translator.translate("Error Details")}
                                </summary>
                                <p class="error-page-details-content">{details}</p>
                            </details>
                        }
                    })}

                // Action Buttons
                <div class="error-page-actions">
                    {on_retry
                        .map(|retry_cb| {
                            view! {
                                <button
                                    class="error-page-button error-page-button-primary"
                                    on:click=move |_| retry_cb.run(())
                                >
                                    <span class="material-symbols-outlined error-page-button-icon">
                                        "refresh"
                                    </span>
                                    {translator.translate("Retry")}
                                </button>
                            }
                        })}
                    {show_navigation
                        .then(|| {
                            view! {
                                <A
                                    href="/"
                                    attr:class="error-page-button error-page-button-secondary"
                                >
                                    <span class="material-symbols-outlined error-page-button-icon">
                                        "home"
                                    </span>
                                    {translator.translate("Home")}
                                </A>
                                {(error_type == "404")
                                    .then(|| {
                                        view! {
                                            <A
                                                href="/articles"
                                                attr:class="error-page-button error-page-button-tertiary"
                                            >
                                                <span class="material-symbols-outlined error-page-button-icon">
                                                    "article"
                                                </span>
                                                {translator.translate("Articles")}
                                            </A>
                                        }
                                    })}
                            }
                        })}
                </div>
            </div>
        </div>
    }
}
