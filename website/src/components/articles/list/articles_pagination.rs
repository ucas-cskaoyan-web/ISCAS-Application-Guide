use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::app::{TranslationContext, SITE_CONFIGURATION};

#[component]
pub fn ArticlesPagination(
    current_page: RwSignal<usize>,
    #[prop(into)] total_pages: Signal<usize>,
    #[prop(into)] total_articles: Signal<usize>,
    #[prop(into, optional)] pagination_visible: Option<Signal<bool>>,
    on_page_change: impl Fn(usize) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let site_config = SITE_CONFIGURATION
        .get()
        .expect("Site configuration should be loaded before rendering pagination");
    let translator = expect_context::<TranslationContext>();
    let pagination_size = site_config.articles.pagination_size;

    let dropdown_open = RwSignal::new(false);
    let dropdown_closing = RwSignal::new(false);

    let pagination_data = Signal::derive(move || {
        let pages = total_pages.get();
        if pages <= 1 {
            return None;
        }

        let current = current_page.get();

        let visible_pages = if pages <= pagination_size {
            (0..pages).collect::<Vec<_>>()
        } else if current < pagination_size {
            (0..pagination_size).collect::<Vec<_>>()
        } else {
            let start = current - pagination_size + 1;
            let end = (start + pagination_size).min(pages);
            (start..end).collect::<Vec<_>>()
        };

        let has_prev = current > 0;
        let has_next = current < pages - 1;
        let last_visible_page = visible_pages.last().copied().unwrap_or(0);
        let show_ellipsis = pages > pagination_size && last_visible_page < pages - 1;

        Some((
            visible_pages,
            has_prev,
            has_next,
            show_ellipsis,
            last_visible_page,
            pages,
            current,
        ))
    });

    let close_dropdown = move || {
        if dropdown_open.get() {
            dropdown_closing.set(true);
            spawn_local(async move {
                TimeoutFuture::new(200).await;
                dropdown_open.set(false);
                dropdown_closing.set(false);
            });
        }
    };

    let toggle_dropdown = move || {
        if dropdown_open.get() {
            close_dropdown();
        } else {
            dropdown_open.set(true);
        }
    };

    let visibility_class = move || {
        if pagination_visible.is_none_or(|v| v.get()) {
            "pagination-visible"
        } else {
            "pagination-hidden"
        }
    };

    view! {
        <Show
            when=move || pagination_data.get().is_some()
            fallback=|| view! { <div class="pagination-container"></div> }
        >
            {move || {
                pagination_data
                    .with(|data| {
                        let (
                            visible_pages,
                            has_prev,
                            has_next,
                            show_ellipsis,
                            last_visible_page,
                            pages,
                            current,
                        ) = data.clone().unwrap();

                        view! {
                            <div class=move || {
                                format!("pagination-container {}", visibility_class())
                            }>
                                <Show when=move || has_prev>
                                    <button
                                        class="pagination-button pagination-button-inactive"
                                        on:click=move |_| on_page_change(current - 1)
                                    >
                                        "<"
                                    </button>
                                </Show>

                                <For
                                    each=move || visible_pages.clone()
                                    key=|page| *page
                                    children=move |page| {
                                        let is_active = current == page;
                                        view! {
                                            <button
                                                class:pagination-button=true
                                                class:pagination-button-active=is_active
                                                class:pagination-button-inactive=!is_active
                                                on:click=move |_| on_page_change(page)
                                            >
                                                {page + 1}
                                            </button>
                                        }
                                    }
                                />

                                <Show when=move || show_ellipsis>
                                    <div class="pagination-dropdown-container">
                                        <button
                                            class="pagination-button pagination-button-inactive"
                                            on:click=move |_| toggle_dropdown()
                                        >
                                            "..."
                                        </button>
                                        <Show when=move || dropdown_open.get()>
                                            <div
                                                class="pagination-dropdown-overlay"
                                                on:click=move |_| close_dropdown()
                                            ></div>
                                            <div class="pagination-dropdown">
                                                <div
                                                    class=move || {
                                                        format!(
                                                            "pagination-dropdown-content {}",
                                                            if dropdown_closing.get() { "hiding" } else { "" },
                                                        )
                                                    }
                                                    on:click=|e| e.stop_propagation()
                                                >
                                                    <For
                                                        each=move || (last_visible_page + 1)..pages
                                                        key=|page| *page
                                                        children=move |page| {
                                                            view! {
                                                                <button
                                                                    class="pagination-dropdown-item"
                                                                    on:click=move |_| {
                                                                        on_page_change(page);
                                                                        close_dropdown();
                                                                    }
                                                                >
                                                                    {page + 1}
                                                                </button>
                                                            }
                                                        }
                                                    />
                                                </div>
                                            </div>
                                        </Show>
                                    </div>
                                </Show>

                                <Show when=move || has_next>
                                    <button
                                        class="pagination-button pagination-button-inactive"
                                        on:click=move |_| on_page_change(current + 1)
                                    >
                                        ">"
                                    </button>
                                </Show>

                                <div class="pagination-info">
                                    {move || {
                                        let current = (current_page.get() + 1).to_string();
                                        let total = total_pages.get().to_string();
                                        let articles = total_articles.get().to_string();
                                        translator.translate_template(
                                            "Page {current} of {total} ({articles} articles)",
                                            &[
                                                ("current", current.as_str()),
                                                ("total", total.as_str()),
                                                ("articles", articles.as_str()),
                                            ],
                                        )
                                    }}
                                </div>
                            </div>
                        }
                    })
            }}
        </Show>
    }
}
