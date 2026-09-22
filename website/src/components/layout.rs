use std::rc::Rc;

use gloo_timers::future::TimeoutFuture;
use leptos::{prelude::*, task::spawn_local};
use leptos_router::components::Outlet;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{HtmlElement, HtmlImageElement};

use crate::{
    app::{TranslationContext, SITE_CONFIGURATION},
    components::{
        error_page::ErrorPage, footer::Footer, navbar::Navbar, progress_bar::ProgressBar,
    },
    types::site::{BackgroundFocusOptions, Site},
};

#[derive(Clone, Copy, PartialEq)]
pub struct ProgressContext {
    pub navigation_active: RwSignal<bool>,
    pub reading_progress: RwSignal<Option<f64>>,
}

impl ProgressContext {
    pub fn refresh_progress() {
        if let Some(progress_context) = use_context::<ProgressContext>() {
            progress_context.update_progress();
        }
    }

    pub fn update_progress(&self) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let has_article_page = document
            .query_selector(".article-page")
            .ok()
            .flatten()
            .is_some();
        if !has_article_page {
            self.reading_progress.set(None);
            return;
        }
        let Some(root) = document.document_element() else {
            return;
        };
        let viewport = window
            .inner_height()
            .ok()
            .and_then(|height| height.as_f64())
            .unwrap_or(0.0);
        let total = (root.scroll_height() as f64 - viewport).max(1.0);
        let value = (window.scroll_y().unwrap_or(0.0) / total * 100.0).clamp(0.0, 100.0);
        self.reading_progress.set(Some(value));
    }
}

fn apply_background_focus(
    body: &HtmlElement,
    focus: Option<&BackgroundFocusOptions>,
    image_width: f64,
    image_height: f64,
) {
    let position = focus
        .map(|focus| {
            let window = web_sys::window();
            let viewport_width = window
                .as_ref()
                .and_then(|window| window.inner_width().ok())
                .and_then(|width| width.as_f64())
                .unwrap_or_default();
            let viewport_height = window
                .as_ref()
                .and_then(|window| window.inner_height().ok())
                .and_then(|height| height.as_f64())
                .unwrap_or_default();
            focus.position(image_width, image_height, viewport_width, viewport_height)
        })
        .unwrap_or_default();
    let value = format!("{:.4}% {:.4}%", position.x_percent, position.y_percent);
    let _ = body
        .style()
        .set_property("--site-wallpaper-position", &value);
}

fn install_background_focus(body: HtmlElement, site: &Site) {
    let focus = site
        .theme
        .as_ref()
        .and_then(|theme| theme.background_focus)
        .filter(BackgroundFocusOptions::is_enabled);

    let Some(focus) = focus else {
        apply_background_focus(&body, None, 0.0, 0.0);
        return;
    };
    let Some(background_url) = site.background_url() else {
        apply_background_focus(&body, None, 0.0, 0.0);
        return;
    };
    let Ok(image) = HtmlImageElement::new() else {
        apply_background_focus(&body, None, 0.0, 0.0);
        return;
    };

    let focus = Rc::new(focus);
    apply_background_focus(&body, Some(focus.as_ref()), 0.0, 0.0);

    let Some(window) = web_sys::window() else {
        return;
    };

    let resize_body = body.clone();
    let resize_focus = Rc::clone(&focus);
    let resize_image = image.clone();
    let resize = Closure::wrap(Box::new(move || {
        apply_background_focus(
            &resize_body,
            Some(resize_focus.as_ref()),
            resize_image.natural_width() as f64,
            resize_image.natural_height() as f64,
        );
    }) as Box<dyn FnMut()>);
    let _ = window.add_event_listener_with_callback("resize", resize.as_ref().unchecked_ref());
    resize.forget();

    let load_body = body.clone();
    let load_focus = Rc::clone(&focus);
    let load_image = image.clone();
    let load = Closure::wrap(Box::new(move || {
        apply_background_focus(
            &load_body,
            Some(load_focus.as_ref()),
            load_image.natural_width() as f64,
            load_image.natural_height() as f64,
        );
    }) as Box<dyn FnMut()>);
    let _ = image.add_event_listener_with_callback("load", load.as_ref().unchecked_ref());
    load.forget();

    image.set_src(&background_url);
}

// This component is used in the router to wrap all pages and provide navbar
#[component]
pub fn AppLayout() -> impl IntoView {
    let nav_progress_active = RwSignal::new(false);
    let reading_progress = RwSignal::new(None::<f64>);

    // Provide ProgressContext to all child components
    provide_context(ProgressContext {
        navigation_active: nav_progress_active,
        reading_progress,
    });

    // Load site configuration
    let (site_signal, set_site_signal) = signal(None::<Result<Site, String>>);

    // Use spawn_local for client-side only operation
    spawn_local(async move {
        let result = Site::fetch().await;
        set_site_signal.set(Some(result));
    });

    // Disable transition animation on init
    Effect::new(move |_| {
        let body = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap();

        // Use `no-transition` class defined in CSS to disable transitions
        // initially
        body.class_list().add_1("no-transition").unwrap();

        spawn_local(async move {
            TimeoutFuture::new(50).await;
            let body = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .body()
                .unwrap();
            body.class_list().remove_1("no-transition").unwrap();
        });
    });

    view! {
        <div class="app-layout">
            <ProgressBar />
            <Suspense fallback=move || {
                view! { <div class="app-layout"></div> }
            }>
                {move || {
                    site_signal
                        .get()
                        .map(|site_result| {
                            match site_result {
                                Ok(site) => {
                                    if SITE_CONFIGURATION.get().is_none() {
                                        let _ = SITE_CONFIGURATION.set(site.clone());
                                    }
                                    if let Some(body) = web_sys::window()
                                        .and_then(|window| window.document())
                                        .and_then(|document| document.body())
                                    {
                                        let wallpaper = site
                                            .background_url()
                                            .map(|url| format!("url(\"{url}\")"))
                                            .unwrap_or_else(|| "none".to_string());
                                        let _ = body.style().set_property("--site-wallpaper", &wallpaper);
                                        install_background_focus(body, &site);
                                    }
                                    // Site config loaded successfully, set global config and render app
                                    view! {
                                        <Navbar />
                                        <main class="main-content">
                                            <Outlet />
                                        </main>
                                        <Footer />
                                    }
                                        .into_any()
                                }
                                Err(e) => {
                                    // Loading error, display error message using ErrorPage component
                                    let translator = expect_context::<TranslationContext>();
                                    view! {
                                        <ErrorPage
                                            title=translator.translate("Failed to Load Configuration")
                                            message=translator.translate("Unable to load site configuration. Please check your network connection and try again.")
                                            error_details=e.to_string()
                                            error_type="network".to_string()
                                            show_navigation=false
                                        />
                                    }
                                        .into_any()
                                }
                            }
                        })
                }}
            </Suspense>
        </div>
    }
}
