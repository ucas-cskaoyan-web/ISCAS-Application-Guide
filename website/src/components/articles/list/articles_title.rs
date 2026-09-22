use gloo_timers::future::TimeoutFuture;
use leptos::{portal::Portal, prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;
use wasm_bindgen::JsCast;

use super::article_card::ArticleSearchResult;
use crate::{app::TranslationContext, models::SearchableArticle};

#[component]
pub fn ArticleTitleBar(
    search_query: RwSignal<String>,
    search_expanded: RwSignal<bool>,
    #[prop(into)] search_results: Signal<Vec<SearchableArticle>>,
    on_search_change: impl Fn(String) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let search_closing = RwSignal::new(false);
    let navigate = use_navigate();
    let translator = expect_context::<TranslationContext>();

    Effect::new(move |_| {
        let is_open = search_expanded.get();
        if let Some(body) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.body())
        {
            if is_open {
                let _ = body.class_list().add_1("search-open");
            } else {
                let _ = body.class_list().remove_1("search-open");
            }
        }
    });

    on_cleanup(|| {
        if let Some(body) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.body())
        {
            let _ = body.class_list().remove_1("search-open");
        }
    });

    let close_search = move || {
        if search_expanded.get_untracked() {
            search_closing.set(true);
            spawn_local(async move {
                TimeoutFuture::new(180).await;
                search_expanded.set(false);
                search_closing.set(false);
            });
        }
    };

    let select_article = Callback::new(move |path: String| {
        if !search_expanded.get_untracked() || search_closing.get_untracked() {
            return;
        }

        search_closing.set(true);
        let navigate = navigate.clone();
        spawn_local(async move {
            TimeoutFuture::new(180).await;
            search_expanded.set(false);
            search_closing.set(false);
            navigate(&path, Default::default());
        });
    });

    // Auto-focus the command input after its enter animation has mounted.
    Effect::new(move |_| {
        if search_expanded.get() {
            spawn_local(async move {
                TimeoutFuture::new(50).await;
                if let Some(document) = web_sys::window().and_then(|window| window.document()) {
                    if let Some(input) = document
                        .query_selector(".articles-search-input")
                        .ok()
                        .flatten()
                    {
                        if let Ok(input) = input.dyn_into::<web_sys::HtmlInputElement>() {
                            let _ = input.focus();
                        }
                    }
                }
            });
        }
    });

    view! {
        <div class="archive-head">
            <div>
                <span class="kicker">{translator.translate("Archive")}</span>
                <h1 class="page-title">{translator.translate("Articles")}</h1>
                <p>{translator.translate("Search the ISCAS Guide by category, tag, and experience.")}</p>
            </div>
            <button
                type="button"
                class="icon-button search-button"
                on:click=move |_| {
                    search_closing.set(false);
                    search_expanded.set(true);
                }
                attr:aria-label=translator.translate("Open article search")
            >
                <span class="material-symbols-outlined" aria-hidden="true">"search"</span>
            </button>
        </div>

        <Show when=move || search_expanded.get()>
            <Portal>
                <div
                    class=move || {
                        if search_closing.get() {
                            "search-dialog search-dialog-closing"
                        } else {
                            "search-dialog search-dialog-open"
                        }
                    }
                    role="dialog"
                    aria-modal="true"
                    attr:aria-label=translator.translate("Search articles")
                    on:click=move |_| close_search()
                >
                    <div class="search-panel" on:click=|event| event.stop_propagation()>
                        <div class="search-row">
                            <span class="material-symbols-outlined search-row-icon" aria-hidden="true">"search"</span>
                            <input
                                class="search-input articles-search-input"
                                type="search"
                                autocomplete="off"
                                attr:aria-label=translator.translate("Search articles")
                                placeholder=translator.translate("Search: category:<any> tag:<any> keywords")
                                prop:value=move || search_query.get()
                                on:input=move |event| on_search_change(event_target_value(&event))
                                on:blur=move |_| {
                                    if search_query.get().is_empty() {
                                        close_search();
                                    }
                                }
                                on:keydown=move |event| {
                                    if event.key() == "Escape" {
                                        close_search();
                                    }
                                }
                            />
                            <button
                                type="button"
                                class="icon-button search-close-button"
                                on:click=move |_| close_search()
                                attr:aria-label=translator.translate("Close article search")
                            >
                                <span class="material-symbols-outlined" aria-hidden="true">"close"</span>
                            </button>
                        </div>
                        <div class="search-results" aria-live="polite">
                            {move || {
                                let query = search_query.get();
                                let results = if query.trim().is_empty() {
                                    Vec::new()
                                } else {
                                    search_results.get()
                                };
                                if results.is_empty() {
                                    let message = if query.trim().is_empty() {
                                        translator.translate("Type to search articles.")
                                    } else {
                                        translator.translate("No matching articles.")
                                    };
                                    view! { <div class="search-empty">{message}</div> }.into_any()
                                } else {
                                    results
                                        .into_iter()
                                        .map(|article| {
                                            view! {
                                                <ArticleSearchResult
                                                    article=article
                                                    on_select=select_article
                                                />
                                            }
                                        })
                                        .collect_view()
                                        .into_any()
                                }
                            }}
                        </div>
                        <div class="search-footer">
                            <span><kbd>"/"</kbd> {format!(" {}", translator.translate("Search"))}</span>
                            <span><kbd>"Esc"</kbd> {format!(" {}", translator.translate("Close"))}</span>
                        </div>
                    </div>
                </div>
            </Portal>
        </Show>
    }
}
