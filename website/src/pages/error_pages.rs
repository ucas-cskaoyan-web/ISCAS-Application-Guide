use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::{
    app::TranslationContext,
    components::{progress_bar::stop_progress_bar, ErrorPage},
};

#[component]
pub fn NotFoundPage() -> impl IntoView {
    stop_progress_bar();

    let location = use_location();
    let requested_path = location.pathname.get();
    let translator = expect_context::<TranslationContext>();

    view! {
        <ErrorPage
            title=translator.translate("Page Not Found")
            message=translator.translate_template(
                "Sorry, the page you requested ({path}) does not exist.",
                &[("path", requested_path.as_str())],
            )
            error_type="404".to_string()
            show_navigation=true
        />
    }
}

#[component]
pub fn ServerErrorPage() -> impl IntoView {
    stop_progress_bar();
    let translator = expect_context::<TranslationContext>();

    view! {
        <ErrorPage
            title=translator.translate("Internal Server Error")
            message=translator.translate("An unexpected error occurred on the server.")
            error_type="500".to_string()
            show_navigation=true
        />
    }
}

#[component]
pub fn NetworkErrorPage() -> impl IntoView {
    stop_progress_bar();
    let translator = expect_context::<TranslationContext>();

    view! {
        <ErrorPage
            title=translator.translate("Network Error")
            message=translator.translate(
                "Unable to connect to the server. Please check your internet connection.",
            )
            error_type="network".to_string()
            show_navigation=true
        />
    }
}
