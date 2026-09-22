use gloo_timers::future::TimeoutFuture;
use leptos::{prelude::*, task::spawn_local};
use leptos_meta::{Meta, Title};

use crate::{
    app::{TranslationContext, SITE_CONFIGURATION},
    components::{
        error_page::ErrorPage,
        progress_bar::stop_progress_bar,
        rich_markdown::{MarkdownRenderMode, RichMarkdownRenderer},
    },
    models::Article,
};

/// The home page is intentionally a projection of the special `home` article.
/// Its content, guide links, data directives, and prose live in Markdown/CSV;
/// this page only owns loading, metadata, animation, and failure handling.
#[component]
pub fn HomePage() -> impl IntoView {
    let site = SITE_CONFIGURATION
        .get()
        .expect("Site configuration must be initialized before HomePage");
    let translator = expect_context::<TranslationContext>();
    let site_name = site.long();
    let article_id = site.home.article_id.clone();
    let article_id_for_resource = article_id.clone();
    let site_for_article = site.clone();
    let article_result = LocalResource::new(move || {
        let article_id = article_id_for_resource.clone();
        let site = site_for_article.clone();
        async move { Article::fetch(&article_id, &site).await }
    });

    let content_ready = RwSignal::new(false);
    let animation_class = RwSignal::new("page-content".to_string());
    Effect::new(move |_| {
        if content_ready.get() {
            animation_class.set("page-content".to_string());
            spawn_local(async move {
                TimeoutFuture::new(10).await;
                animation_class.set("page-content animate-fade-in-up".to_string());
            });
            stop_progress_bar();
        }
    });

    view! {
        <Title text=move || {
            article_result.with(|result| {
                result.as_ref().map_or(translator.translate("Loading..."), |result| {
                    result.as_ref().map_or(translator.translate("Error loading home"), |(article, _)| {
                        format!("{} - {}", article.title, site_name)
                    })
                })
            })
        } />
        <Meta name="description" content=move || {
            article_result.with(|result| {
                result.as_ref().map_or(String::new(), |result| {
                    result
                        .as_ref()
                        .map(|(article, _)| article.description.chars().take(160).collect())
                        .unwrap_or_default()
                })
            })
        } />
        <Suspense fallback=move || view! {
            <div class="article-loading" attr:aria-label=translator.translate("Loading home page")></div>
        }>
            {move || {
                article_result.with(|result| match result {
                    Some(Ok((_, markdown_content))) => {
                        content_ready.set(true);
                        view! {
                            <div class=move || format!("page-container home-page {}", animation_class.get())>
                                <RichMarkdownRenderer
                                    article_id=article_id.clone()
                                    content=markdown_content.clone()
                                    mode=MarkdownRenderMode::Home
                                />
                            </div>
                        }
                            .into_any()
                    }
                    Some(Err(error)) => view! {
                        <div class="page-container">
                            <ErrorPage
                                title=translator.translate("Home Guide Unavailable")
                                message=translator.translate("The ISCAS Guide could not be loaded. Please try again later.")
                                error_details=error.clone()
                                error_type="network".to_string()
                                show_navigation=false
                            />
                        </div>
                    }.into_any(),
                    None => view! {
                        <div class="article-loading" attr:aria-label=translator.translate("Loading home page")></div>
                    }.into_any(),
                })
            }}
        </Suspense>
    }
}
