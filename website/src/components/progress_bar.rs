use gloo_timers::future::TimeoutFuture;
use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::use_location;
use wasm_bindgen::{closure::Closure, JsCast};

use crate::components::layout::ProgressContext;

#[component]
pub fn ProgressBar() -> impl IntoView {
    let location = use_location();
    let progress_context =
        use_context::<ProgressContext>().expect("ProgressContext must be provided");

    // Keep one app-lifetime listener and update the shared track only while an
    // article page is present. This avoids leaking one listener per route
    // visit.
    Effect::new(move |_| {
        let Some(window) = web_sys::window() else {
            return;
        };

        progress_context.update_progress();

        let callback =
            Closure::wrap(
                Box::new(move |_event: web_sys::Event| progress_context.update_progress())
                    as Box<dyn FnMut(web_sys::Event)>,
            );
        let _ =
            window.add_event_listener_with_callback("scroll", callback.as_ref().unchecked_ref());
        callback.forget();
    });

    // Monitor route changes to trigger the progress bar animation
    Effect::new(move |_| {
        let _pathname = location.pathname.get();
        spawn_local(async move {
            if progress_context.navigation_active.get_untracked() {
                // If animation was somehow stuck true, reset
                progress_context.navigation_active.set(false);
                // Brief pause to ensure CSS can pick up the change if
                // re-triggering fast
                TimeoutFuture::new(10).await;
            }
            progress_context.navigation_active.set(true); // Activate progress
                                                          // bar
        });
    });

    // Reuse the same fixed track for route loading and article reading
    // progress.
    let progress_class = move || {
        if progress_context.navigation_active.get() {
            // Active state: full width with a longer ease-out transition
            "progress-bar-active"
        } else if progress_context.reading_progress.get().is_some() {
            "progress-bar-reading"
        } else {
            // Inactive state: zero width with a shorter ease-out transition
            "progress-bar-inactive"
        }
    };
    let progress_style = move || {
        if progress_context.navigation_active.get() {
            String::new()
        } else {
            progress_context
                .reading_progress
                .get()
                .map(|value| format!("width: {value:.2}%"))
                .unwrap_or_default()
        }
    };

    view! {
        <div class="progress-bar-container">
            <div class=progress_class style=progress_style></div>
        </div>
    }
}

pub fn stop_progress_bar() {
    // Try to get the context, but don't panic if it's not available
    if let Some(progress_context) = use_context::<ProgressContext>() {
        let nav_progress_active = progress_context.navigation_active;
        spawn_local(async move {
            TimeoutFuture::new(400).await; // Wait for 400ms before hiding
            nav_progress_active.set(false); // Deactivate progress bar
        });
    }
}
