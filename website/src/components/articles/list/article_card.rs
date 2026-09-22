use leptos::prelude::*;
use leptos_router::components::A;

use crate::{
    app::{TranslationContext, SITE_CONFIGURATION},
    models::SearchableArticle,
};

fn category_icon(category: &str) -> &'static str {
    match category.to_ascii_lowercase().as_str() {
        "technology" | "tech" => "terminal",
        "general" => "edit_note",
        _ => "auto_stories",
    }
}

fn tag_href(tag: &str) -> String {
    format!(
        "/articles?tag={}",
        tag.replace('&', "%26").replace(' ', "%20")
    )
}

#[component]
pub fn ArticleCard(
    article: SearchableArticle,
    #[prop(default = true)] show_description: bool,
) -> impl IntoView {
    let translator = expect_context::<TranslationContext>();
    let category = article
        .article
        .category
        .clone()
        .unwrap_or_else(|| "Notes".to_string());
    let is_experience = category == "上岸经验";
    let date = article.article.date.clone().unwrap_or_default();
    let title = article.article.title.clone();
    let first_tag = article.article.tags.first().cloned();
    let icon = category_icon(&category);
    let article_path = format!("/articles/{}", article.id);
    let first_tag_href = first_tag.as_deref().map(tag_href);
    let has_first_tag = first_tag.is_some();
    let category_label = first_tag.clone().unwrap_or_else(|| category.clone());
    let secondary_category = first_tag
        .as_ref()
        .filter(|tag| *tag != &category)
        .map(|_| category.clone());
    let cover_url = SITE_CONFIGURATION.get().and_then(|site| {
        article
            .article
            .cover
            .as_ref()
            .map(|cover| site.article_asset_url(&article.id, cover))
    });
    let cover_title = title.clone();

    view! {
        <li class="article-card article-row">
            <div class=move || {
                format!(
                    "article-card-link{}",
                    if is_experience { " article-card-link-no-thumb" } else { "" }
                )
            }>
                {(!is_experience).then(|| view! {
                    <A href=article_path.clone() attr:class="article-card-thumb-link">
                        {cover_url
                            .map(|url| {
                                let alt = cover_title.clone();
                                view! { <div class="article-card-thumb"><img src=url alt=alt /></div> }
                                    .into_any()
                            })
                            .unwrap_or_else(|| {
                                view! {
                                    <div class="article-card-thumb" aria-hidden="true">
                                        <span class="material-symbols-outlined">{icon}</span>
                                    </div>
                                }
                                .into_any()
                            })}
                    </A>
                })}
                <div class="article-card-body">
                    <div class="article-card-meta">
                        {first_tag_href.clone().map(|href| {
                            let tag = first_tag.clone().unwrap_or_default();
                            view! {
                                <A href=href attr:class="article-card-category article-card-tag-link">
                                    {format!("#{tag}")}
                                </A>
                            }
                        })}
                        {secondary_category.map(|category| view! {
                            <span class="article-card-tag">{category}</span>
                        })}
                        {(!has_first_tag).then(|| view! {
                            <span class="article-card-category">{category_label.clone()}</span>
                        })}
                    </div>
                    <A href=article_path.clone() attr:class="article-card-primary-link">
                        <h2 class="article-card-title">{title.clone()}</h2>
                        {show_description.then(|| view! {
                            <p class="article-card-description">{article.article.description.clone()}</p>
                        })}
                        <div class="article-card-extra">
                            <span>{format!(
                                "{} {}",
                                article.article.tags.len(),
                                translator.translate("tags")
                            )}</span>
                            <span class="article-card-arrow" aria-hidden="true">"↗"</span>
                        </div>
                    </A>
                </div>
                <time class="article-card-date">{date}</time>
            </div>
        </li>
    }
}

#[component]
pub fn ArticleSearchResult(
    article: SearchableArticle,
    on_select: Callback<String>,
) -> impl IntoView {
    let translator = expect_context::<TranslationContext>();
    let category = article
        .article
        .category
        .clone()
        .unwrap_or_else(|| "Notes".to_string());
    let metadata = format!(
        "{} · {} {}",
        category,
        article.article.tags.len(),
        translator.translate("tags")
    );
    let icon = category_icon(&category);
    let article_path = format!("/articles/{}", article.id);

    view! {
        <A
            href=article_path.clone()
            attr:class="search-result"
            on:click=move |event| {
                if event.button() != 0
                    || event.meta_key()
                    || event.alt_key()
                    || event.ctrl_key()
                    || event.shift_key()
                {
                    return;
                }
                event.prevent_default();
                on_select.run(article_path.clone());
            }
        >
            <div class="search-result-thumb" aria-hidden="true">
                <span class="material-symbols-outlined">{icon}</span>
            </div>
            <div>
                <h2 class="search-result-title">{article.article.title.clone()}</h2>
                <div class="search-result-meta">{metadata}</div>
            </div>
        </A>
    }
}
