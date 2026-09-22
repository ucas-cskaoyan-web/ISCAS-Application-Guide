use gloo_timers::future::TimeoutFuture;
use leptos::{prelude::*, reactive::spawn_local};
use leptos_meta::Title;

use crate::{
    app::{TranslationContext, SITE_CONFIGURATION},
    components::{
        articles::list::{ArticleTitleBar, ArticlesList, ArticlesPagination},
        error_page::ErrorPage,
        progress_bar::stop_progress_bar,
    },
    models::{ArticleIndex, ArticleSearchIndex, SearchCriteria},
};

fn initial_search_query() -> String {
    let Some(window) = web_sys::window() else {
        return String::new();
    };
    let Ok(search) = window.location().search() else {
        return String::new();
    };
    let Some(tag) = web_sys::UrlSearchParams::new_with_str(&search)
        .ok()
        .and_then(|params| params.get("tag"))
        .filter(|tag| !tag.trim().is_empty())
    else {
        return String::new();
    };
    format!("tag:{tag}")
}

#[component]
pub fn ArticlesListPage() -> impl IntoView {
    let site = SITE_CONFIGURATION
        .get()
        .expect("Site configuration not initialized");
    let translator = expect_context::<TranslationContext>();
    let search_query = RwSignal::new(initial_search_query());
    let search_expanded = RwSignal::new(false);
    let current_page = RwSignal::new(0usize);
    let active_category = RwSignal::new(String::new());

    let site_clone = site.clone();
    let articles_index = LocalResource::new(move || {
        let site = site_clone.clone();
        async move {
            ArticleIndex::fetch(&site)
                .await
                .map(|index| index.to_search_index())
        }
    });
    let animation_class = RwSignal::new("page-content");
    let pagination_visible = RwSignal::new(true);

    // Page animation - trigger only once when component mounts
    let content_ready = RwSignal::new(false);
    Effect::new(move |_| {
        if content_ready.get() {
            animation_class.set("page-content animate-fade-in-up");
            stop_progress_bar();
        }
    });

    // Event handlers for child components
    let handle_search_change = move |query: String| {
        search_query.set(query);
        current_page.set(0);
    };

    let handle_page_change = move |page: usize| {
        // First scroll to top smoothly, then change page after scroll completes
        spawn_local(async move {
            // Scroll to top
            if let Some(window) = web_sys::window() {
                let options = web_sys::ScrollToOptions::new();
                options.set_top(0.0);
                options.set_behavior(web_sys::ScrollBehavior::Smooth);
                window.scroll_to_with_scroll_to_options(&options);
            }

            // Wait for smooth scroll to complete
            TimeoutFuture::new(400).await;

            // Now update the page
            current_page.set(page);
        });
    };

    view! {
        <Title text=format!("{} - {}", translator.translate("Articles"), site.long()) />
        <Suspense fallback=move || {
            view! { <div></div> }
        }>
            {move || {
                articles_index
                    .get()
                    .map(|result| {
                        match result {
                            Ok(search_index) => {
                                content_ready.set(true);

                                view! {
                                    <ArticlesListPageContent
                                        search_index=search_index
                                        search_query=search_query
                                        search_expanded=search_expanded
                                        current_page=current_page
                                        active_category=active_category
                                        animation_class=animation_class
                                        pagination_visible=pagination_visible
                                        handle_search_change=handle_search_change
                                        handle_page_change=handle_page_change
                                    />
                                }
                                    .into_any()
                            }
                            Err(_) => {
                                view! {
                                    <ErrorPage
                                        title=translator.translate("Unexpected Error")
                                        message=translator.translate("An unexpected error occurred while fetching articles.")
                                        error_type="500".to_string()
                                        show_navigation=true
                                    />
                                }
                                    .into_any()
                            }
                        }
                    })
            }}
        </Suspense>
    }
}

#[component]
fn ArticlesListPageContent(
    search_index: crate::models::ArticleSearchIndex,
    search_query: RwSignal<String>,
    search_expanded: RwSignal<bool>,
    current_page: RwSignal<usize>,
    active_category: RwSignal<String>,
    animation_class: RwSignal<&'static str>,
    pagination_visible: RwSignal<bool>,
    handle_search_change: impl Fn(String) + 'static + Copy + Send + Sync,
    handle_page_change: impl Fn(usize) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let site_config = SITE_CONFIGURATION
        .get()
        .expect("Site configuration not initialized");
    let translator = expect_context::<TranslationContext>();
    let articles_per_page = site_config.articles.maximum_number_per_page;
    let mut categories = search_index.categories.clone();
    categories.sort();
    let filtered_articles = Memo::new(move |_| {
        let criteria = SearchCriteria::parse(&search_query.get());
        let active_category_value = active_category.get();
        let criteria = if active_category_value.is_empty() {
            criteria
        } else {
            SearchCriteria {
                categories: vec![active_category_value],
                ..criteria
            }
        };
        search_index
            .search_with_criteria(&criteria)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>()
    });
    let current_page_articles = Memo::new(move |_| {
        let articles = filtered_articles.get();
        let articles_refs: Vec<&_> = articles.iter().collect();
        ArticleSearchIndex::paginate(&articles_refs, current_page.get(), articles_per_page)
            .iter()
            .map(|&article| article.clone())
            .collect::<Vec<_>>()
    });

    let empty_message = Memo::new(move |_| {
        let articles = filtered_articles.get();
        let criteria = SearchCriteria::parse(&search_query.get());

        if articles.is_empty() {
            if criteria.is_empty() {
                translator.translate("No articles yet!")
            } else {
                translator.translate("No articles found matching your search criteria.")
            }
        } else {
            translator.translate("No articles on this page.")
        }
    });

    let total_pages = Memo::new(move |_| {
        let articles = filtered_articles.get();
        ArticleSearchIndex::total_pages(articles.len(), articles_per_page)
    });

    let total_articles = Memo::new(move |_| filtered_articles.get().len());

    view! {
        // Main container.
        <div class=move || format!("page-container {}", animation_class.get())>
            <div class="shell">
                <ArticleTitleBar
                    search_query=search_query
                    search_expanded=search_expanded
                    search_results=filtered_articles
                    on_search_change=handle_search_change
                />
                <div class="archive-filters" attr:aria-label=translator.translate("Filter articles by category")>
                    <button
                        type="button"
                        class=move || if active_category.get().is_empty() {
                            "filter-chip filter-chip-active"
                        } else {
                            "filter-chip"
                        }
                        on:click=move |_| {
                            active_category.set(String::new());
                            current_page.set(0);
                        }
                    >
                        {translator.translate("All articles")}
                    </button>
                    {categories
                        .into_iter()
                        .map(|category| {
                            let category_for_class = category.clone();
                            let category_for_click = category.clone();
                            let category_label = category.clone();
                            view! {
                                <button
                                    type="button"
                                    class=move || if active_category.get() == category_for_class {
                                        "filter-chip filter-chip-active"
                                    } else {
                                        "filter-chip"
                                    }
                                    on:click=move |_| {
                                        active_category.set(category_for_click.clone());
                                        current_page.set(0);
                                    }
                                >
                                    {category_label}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
                <ArticlesList
                    articles=current_page_articles
                    empty_message=empty_message
                    pagination_visible=pagination_visible
                />
                <ArticlesPagination
                    current_page=current_page
                    total_pages=total_pages
                    total_articles=total_articles
                    pagination_visible=Signal::from(pagination_visible)
                    on_page_change=handle_page_change
                />
            </div>
        </div>
    }
}
