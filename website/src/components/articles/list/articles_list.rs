use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::{components::articles::list::article_card::ArticleCard, models::SearchableArticle};

#[component]
pub fn ArticlesList(
    #[prop(into)] articles: Signal<Vec<SearchableArticle>>,
    #[prop(into)] empty_message: Signal<String>,
    #[prop(into, optional)] pagination_visible: Option<RwSignal<bool>>,
) -> impl IntoView {
    let init_signal = RwSignal::new(true);
    let show_group = RwSignal::new(true);
    let update_msg = RwSignal::new(String::new());
    let update_group = RwSignal::new(Vec::<SearchableArticle>::new()); // Create
                                                                       // a reactive
                                                                       // effect
                                                                       // that responds
                                                                       // to prop
                                                                       // changes
    Effect::new(move |prev| {
        let current_articles = articles.get();
        let current_message = empty_message.get();
        let reactive_bundle = (current_articles.clone(), current_message.clone());

        // Check if this is the first run or if the bundle has actually changed
        if let Some(prev_bundle) = prev {
            if prev_bundle == reactive_bundle {
                return reactive_bundle; // No change, don't animate
            }
        }

        spawn_local(async move {
            if init_signal.get_untracked() {
                // No load animation on initial load
                init_signal.set(false);
                update_msg.set(current_message);
                update_group.set(current_articles);
                return;
            }

            // Hide both articles list and pagination
            show_group.set(false);
            if let Some(pagination_signal) = pagination_visible {
                pagination_signal.set(false);
            }

            // Wait for hide animation to complete
            TimeoutFuture::new(400).await;

            // Update content
            update_msg.set(current_message);
            update_group.set(current_articles);

            // Show both articles list and pagination
            show_group.set(true);
            if let Some(pagination_signal) = pagination_visible {
                pagination_signal.set(true);
            }
        });

        reactive_bundle
    });

    view! {
        // Container with transition for opacity
        <div class=move || {
            format!(
                "articles-list-container {}",
                if show_group.get() { "articles-list-visible" } else { "articles-list-hidden" },
            )
        }>
            {move || {
                let articles = update_group.get();
                let message = update_msg.get();
                if articles.is_empty() {

                    view! {
                        // "No articles" message
                        <div class="articles-list-empty">
                            <p class="articles-list-empty-text">{message}</p>
                        </div>
                    }
                        .into_any()
                } else {
                    // List of articles
                    view! {
                        <ul class="articles-list">
                            {move || {
                                articles
                                    .iter()
                                    .map(|article| {
                                        let article = article.clone();
                                        view! { <ArticleCard article=article /> }
                                    })
                                    .collect::<Vec<_>>()
                            }}
                        </ul>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}
