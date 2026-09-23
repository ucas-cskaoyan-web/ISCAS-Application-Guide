use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_location};
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::spawn_local;

use crate::{
    app::{TranslationContext, SITE_CONFIGURATION},
    components::theme_toggle::ThemeToggle,
};

#[derive(Clone, Copy, PartialEq)]
enum CurrentActiveLink {
    Home,
    Articles,
    About,
    None,
}

impl From<String> for CurrentActiveLink {
    fn from(path: String) -> Self {
        match path.as_str() {
            "/" => CurrentActiveLink::Home,
            "/about" | "/articles/about" => CurrentActiveLink::About,
            "/articles" => CurrentActiveLink::Articles,
            path if path.starts_with("/articles/") => CurrentActiveLink::Articles,
            _ => CurrentActiveLink::None,
        }
    }
}

fn nav_link_class(active: bool) -> &'static str {
    if active {
        "nav-link nav-link-active"
    } else {
        "nav-link"
    }
}

#[component]
pub fn Navbar() -> impl IntoView {
    let (is_mobile_menu_open, set_is_mobile_menu_open) = signal(false);
    let (is_closing, set_is_closing) = signal(false);
    let (is_scrolled, set_is_scrolled) = signal(false);
    let current_active_link = RwSignal::new(CurrentActiveLink::None);

    let site = SITE_CONFIGURATION
        .get()
        .expect("SITE_CONFIGURATION must be initialized before Navbar is rendered");
    let translator = expect_context::<TranslationContext>();
    let legacy_site_url = site.links.legacy_site_url.clone();

    let close_mobile_menu = move || {
        if is_mobile_menu_open.get_untracked() {
            set_is_closing.set(true);
            spawn_local(async move {
                TimeoutFuture::new(180).await;
                set_is_mobile_menu_open.set(false);
                set_is_closing.set(false);
            });
        }
    };

    let location = use_location();
    Effect::new(move |_| {
        let path = location.pathname.get();
        current_active_link.set(CurrentActiveLink::from(path));
        close_mobile_menu();
    });

    // The app bar is transparent over the hero and gains its scrim only after
    // the user starts scrolling. The closure is intentionally installed once.
    Effect::new(move |_| {
        let Some(window) = web_sys::window() else {
            return;
        };

        let update = move || {
            let scrolled = web_sys::window()
                .and_then(|window| window.scroll_y().ok())
                .is_some_and(|offset| offset > 24.0);
            set_is_scrolled.set(scrolled);
        };
        update();

        let callback = Closure::wrap(
            Box::new(move |_event: web_sys::Event| update()) as Box<dyn FnMut(web_sys::Event)>
        );
        let _ =
            window.add_event_listener_with_callback("scroll", callback.as_ref().unchecked_ref());
        callback.forget();
    });

    let toggle_mobile_menu = move |_| {
        set_is_mobile_menu_open.set(true);
        set_is_closing.set(false);
    };

    view! {
        <header class=move || {
            format!("site-header {}", if is_scrolled.get() { "scrolled" } else { "" })
        }>
            <nav class="shell nav" attr:aria-label=translator.translate("Primary navigation")>
                <A href="/" attr:class="brand">
                    <span class="navbar-brand-long">{site.long()}</span>
                    <span class="navbar-brand-short">{site.short()}</span>
                </A>

                <div class="navbar-desktop">
                    <A
                        href="/"
                        attr:class=move || nav_link_class(current_active_link.get() == CurrentActiveLink::Home)
                        attr:aria-current=move || {
                            (current_active_link.get() == CurrentActiveLink::Home).then_some("page")
                        }
                    >
                        <span class="material-symbols-outlined" aria-hidden="true">"home"</span>
                        <span>{translator.translate("Home")}</span>
                    </A>
                    <A
                        href="/articles"
                        attr:class=move || nav_link_class(current_active_link.get() == CurrentActiveLink::Articles)
                        attr:aria-current=move || {
                            (current_active_link.get() == CurrentActiveLink::Articles).then_some("page")
                        }
                    >
                        <span class="material-symbols-outlined" aria-hidden="true">"description"</span>
                        <span>{translator.translate("Articles")}</span>
                    </A>
                    <A
                        href="/about"
                        attr:class=move || nav_link_class(current_active_link.get() == CurrentActiveLink::About)
                        attr:aria-current=move || {
                            (current_active_link.get() == CurrentActiveLink::About).then_some("page")
                        }
                    >
                        <span class="material-symbols-outlined" aria-hidden="true">"info"</span>
                        <span>{translator.translate("About")}</span>
                    </A>
                    <a
                        class="nav-link"
                        href=legacy_site_url.clone()
                        target="_blank"
                        rel="noopener noreferrer"
                        title=translator.translate("Legacy site")
                    >
                        <span class="material-symbols-outlined" aria-hidden="true">"history"</span>
                        <span>{translator.translate("Legacy site")}</span>
                    </a>
                    <a
                        class="nav-link nav-login-link"
                        href="http://106.53.55.161/"
                        title=translator.translate("Score entry")
                    >
                        <span class="material-symbols-outlined" aria-hidden="true">"edit_note"</span>
                        <span>{translator.translate("Score entry")}</span>
                    </a>
                    <ThemeToggle />
                </div>

                <button
                    type="button"
                    class="icon-button navbar-mobile-button"
                    on:click=toggle_mobile_menu
                    attr:aria-label=translator.translate("Open navigation menu")
                    aria-controls="mobile-navigation"
                    aria-expanded=move || is_mobile_menu_open.get().to_string()
                >
                    <span class="material-symbols-outlined" aria-hidden="true">"menu"</span>
                </button>
            </nav>

            <Show when=move || is_mobile_menu_open.get()>
                <div
                    class=move || {
                        format!(
                            "mobile-menu-overlay {}",
                            if is_closing.get() {
                                "mobile-menu-overlay-fade-out"
                            } else {
                                "mobile-menu-overlay-fade-in"
                            },
                        )
                    }
                    on:click=move |_| close_mobile_menu()
                ></div>
                <aside
                    id="mobile-navigation"
                    class=move || {
                        format!(
                            "mobile-menu-panel {}",
                            if is_closing.get() {
                                "mobile-menu-panel-slide-out"
                            } else {
                                "mobile-menu-panel-slide-in"
                            },
                        )
                    }
                    attr:aria-label=translator.translate("Mobile navigation")
                >
                    <div class="mobile-menu-header">
                        <h2 class="mobile-menu-title">{translator.translate("Navigation")}</h2>
                        <button
                            type="button"
                            class="icon-button mobile-menu-close-button"
                            on:click=move |_| close_mobile_menu()
                            attr:aria-label=translator.translate("Close navigation menu")
                        >
                            <span class="material-symbols-outlined" aria-hidden="true">"close"</span>
                        </button>
                    </div>
                    <div class="mobile-menu-links">
                        <A
                            href="/"
                            attr:class=move || nav_link_class(current_active_link.get() == CurrentActiveLink::Home)
                            on:click=move |_| close_mobile_menu()
                        >
                            <span class="material-symbols-outlined" aria-hidden="true">"home"</span>
                            <span>{translator.translate("Home")}</span>
                        </A>
                        <A
                            href="/articles"
                            attr:class=move || nav_link_class(current_active_link.get() == CurrentActiveLink::Articles)
                            on:click=move |_| close_mobile_menu()
                        >
                            <span class="material-symbols-outlined" aria-hidden="true">"description"</span>
                            <span>{translator.translate("Articles")}</span>
                        </A>
                        <A
                            href="/about"
                            attr:class=move || nav_link_class(current_active_link.get() == CurrentActiveLink::About)
                            on:click=move |_| close_mobile_menu()
                        >
                            <span class="material-symbols-outlined" aria-hidden="true">"info"</span>
                            <span>{translator.translate("About")}</span>
                        </A>
                        <a
                            class="nav-link"
                            href=legacy_site_url.clone()
                            target="_blank"
                            rel="noopener noreferrer"
                            on:click=move |_| close_mobile_menu()
                        >
                            <span class="material-symbols-outlined" aria-hidden="true">"history"</span>
                            <span>{translator.translate("Legacy site")}</span>
                        </a>
                        <a
                            class="nav-link nav-login-link"
                            href="http://106.53.55.161/"
                            title=translator.translate("Score entry")
                        >
                            <span class="material-symbols-outlined" aria-hidden="true">"edit_note"</span>
                            <span>{translator.translate("Score entry")}</span>
                        </a>
                    </div>
                    <div class="mobile-menu-footer">
                        <ThemeToggle />
                    </div>
                </aside>
            </Show>
        </header>
    }
}
