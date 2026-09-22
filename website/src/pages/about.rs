use gloo_timers::future::TimeoutFuture;
use leptos::{prelude::*, task::spawn_local};
use leptos_meta::{Meta, Stylesheet, Title};
use leptos_router::components::A;

use crate::{
    app::{TranslationContext, SITE_CONFIGURATION},
    components::{error_page::ErrorPage, progress_bar::stop_progress_bar},
    models::Article,
    utils::MarkdownArticle,
};

const ABOUT_ARTICLE_ID: &str = "about";

#[component]
pub fn AboutPage() -> impl IntoView {
    let site_config = SITE_CONFIGURATION
        .get()
        .expect("Site configuration should be loaded by AppLayout");
    let translator = expect_context::<TranslationContext>();
    let site_name = site_config.long();
    let github_url = format!("https://github.com/{}", site_config.author.github);
    let article_result =
        LocalResource::new(
            move || async move { Article::fetch(ABOUT_ARTICLE_ID, site_config).await },
        );

    let content_ready = RwSignal::new(false);
    let animation_class = RwSignal::new("page-content".to_string());
    Effect::new(move |_| {
        animation_class.set("page-content".to_string());
        if content_ready.get() {
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
                    result.as_ref().map_or(translator.translate("Error loading About page"), |(article, _)| {
                        format!("{} - {}", article.title, site_name)
                    })
                })
            })
        } />
        <Meta name="description" content=move || {
            article_result.with(|result| {
                result.as_ref().map_or(translator.translate("Loading..."), |result| {
                    result.as_ref().map_or(translator.translate("Error loading About page"), |(article, _)| {
                        article.description.chars().take(150).collect::<String>()
                    })
                })
            })
        } />
        <Stylesheet href="https://cdn.jsdelivr.net/npm/katex@0.16.22/dist/katex.min.css" />
        <Suspense fallback=move || view! { <div class="article-loading" attr:aria-label=translator.translate("Loading About page")></div> }>
            {move || {
                article_result.with(|result| match result {
                    Some(Ok((article, markdown_content))) => {
                        let markdown = MarkdownArticle::new(
                            markdown_content.clone(),
                            ABOUT_ARTICLE_ID.to_string(),
                        );
                        let html_output = markdown.render_about();
                        let title = article.title.clone();
                        let description = article.description.clone();
                        let category = article
                            .category
                            .clone()
                            .unwrap_or_else(|| "About".to_string());
                        let github_button_url = github_url.clone();
                        content_ready.set(true);

                        view! {
                            <div class=move || format!("page-container {}", animation_class.get())>
                                <section class="about-intro shell">
                                    <span class="kicker">{category}</span>
                                    <h1 class="page-title">{title}</h1>
                                    <p class="about-lead">{description}</p>
                                    <div class="about-actions">
                                        <A href="/articles" attr:class="btn btn-tonal">
                                            <span class="material-symbols-outlined" aria-hidden="true">"menu_book"</span>
                                            {translator.translate("Browse articles")}
                                        </A>
                                        <a href=github_button_url class="btn btn-outlined">
                                            <span class="material-symbols-outlined" aria-hidden="true">"open_in_new"</span>
                                            {translator.translate("GitHub")}
                                        </a>
                                    </div>
                                </section>

                                <section class="about-body shell">
                                    <div class="about-markdown markdown-container" inner_html=html_output></div>
                                </section>
                            </div>
                        }
                            .into_any()
                    }
                    Some(Err(error)) => {
                        view! {
                            <div class="page-container">
                                <ErrorPage
                                    title=translator.translate("About Page Unavailable")
                                    message=translator.translate("The About article could not be loaded.")
                                    error_details=error.clone()
                                    error_type="network".to_string()
                                    show_navigation=true
                                />
                            </div>
                        }
                            .into_any()
                    }
                    None => view! { <div class="article-loading" attr:aria-label=translator.translate("Loading About page")></div> }.into_any(),
                })
            }}
        </Suspense>
    }
}
