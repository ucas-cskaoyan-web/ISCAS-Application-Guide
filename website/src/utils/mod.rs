use std::{collections::HashMap, fmt::Debug};

use katex_wasmbind::KaTeXOptions;
use leptos::prelude::*;
use pulldown_cmark::{
    html, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd, TextMergeStream,
};

use crate::{app::SITE_CONFIGURATION, bindgen};

#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownHeading {
    pub id: String,
    pub title: String,
    pub level: u8,
}

pub struct MarkdownArticle {
    id: String,
    content: String,
}

impl MarkdownArticle {
    pub fn new(content: String, id: String) -> Self {
        Self { id, content }
    }

    pub fn headings(&self) -> Vec<MarkdownHeading> {
        let mut headings = Vec::new();
        let mut counts = HashMap::<String, usize>::new();
        let mut current: Option<(u8, String)> = None;

        for event in Parser::new_ext(&self.content, Options::all()) {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    current = Some((heading_level(level), String::new()));
                }
                Event::Text(text) | Event::Code(text) => {
                    if let Some((_, title)) = current.as_mut() {
                        title.push_str(&text);
                    }
                }
                Event::End(TagEnd::Heading(_)) => {
                    if let Some((level, title)) = current.take() {
                        let base_id = slugify(&title);
                        let count = counts.entry(base_id.clone()).or_insert(0);
                        let id = if *count == 0 {
                            base_id
                        } else {
                            format!("{base_id}-{}", *count + 1)
                        };
                        *count += 1;
                        headings.push(MarkdownHeading { id, title, level });
                    }
                }
                _ => {}
            }
        }

        headings
    }

    /// Render a special article using the section layout of the About page.
    ///
    /// The section boundaries still come from Markdown headings; this only adds
    /// the layout wrappers needed by the existing About presentation.
    pub fn render_about(&self) -> String {
        let headings = self.headings();
        self.render_markdown_with_headings(true, &headings)
    }

    /// Render one Markdown fragment using heading IDs allocated by the parent
    /// document.  Rich Markdown uses this to keep duplicate heading suffixes
    /// stable even when directives split one article into several fragments.
    pub fn render_fragment(&self, headings: &[MarkdownHeading]) -> String {
        self.render_markdown_with_headings(false, headings)
    }

    fn render_markdown(&self, wrap_about_sections: bool) -> String {
        let headings = self.headings();
        self.render_markdown_with_headings(wrap_about_sections, &headings)
    }

    fn render_markdown_with_headings(
        &self,
        wrap_about_sections: bool,
        headings: &[MarkdownHeading],
    ) -> String {
        let mut html_output = String::new();
        let mut in_code_block = false;
        let mut lang = String::new();
        let mut iterator = Vec::new();
        let section_ids = if wrap_about_sections {
            headings
                .iter()
                .filter(|heading| heading.level == 2)
                .map(|heading| heading.id.clone())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let mut section_index = 0;
        let mut about_section_open = false;
        let events = Parser::new_ext(&self.content, Options::all());
        for e in TextMergeStream::new(events) {
            match e {
                Event::Start(Tag::Heading {
                    level: HeadingLevel::H2,
                    id,
                    classes,
                    attrs,
                }) if wrap_about_sections => {
                    if about_section_open {
                        iterator.push(Event::Html("</div>".into()));
                    }
                    let section_id = section_ids
                        .get(section_index)
                        .map(String::as_str)
                        .unwrap_or("section");
                    iterator.push(Event::Html(
                        format!(r#"<div class="about-section" data-section="{section_id}">"#)
                            .into(),
                    ));
                    about_section_open = true;
                    section_index += 1;
                    iterator.push(Event::Start(Tag::Heading {
                        level: HeadingLevel::H2,
                        id,
                        classes,
                        attrs,
                    }));
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    match kind {
                        CodeBlockKind::Fenced(lang_str) => {
                            lang = lang_str.to_string();
                        }
                        CodeBlockKind::Indented => {
                            lang.clear(); // Clear language marker for indented
                                          // code blocks without language info
                        }
                    }
                    in_code_block = true;
                }
                Event::End(TagEnd::CodeBlock) => {
                    in_code_block = false;
                }
                Event::End(TagEnd::Heading(HeadingLevel::H2)) if wrap_about_sections => {
                    iterator.push(Event::End(TagEnd::Heading(HeadingLevel::H2)));
                }
                Event::Text(text) => {
                    if in_code_block {
                        let output =
                            format!("<pre>{}</pre>", bindgen::highlight_code(&text, &lang));
                        iterator.push(Event::Html(output.into()));
                    } else {
                        iterator.push(Event::Text(text));
                    }
                }
                Event::DisplayMath(equation) => {
                    let d = KaTeXOptions::display_mode();
                    iterator.push(Event::Html(d.render(&equation).into()));
                }
                Event::InlineMath(equation) => {
                    let i = KaTeXOptions::inline_mode();
                    iterator.push(Event::Html(i.render(&equation).into()));
                }
                Event::Start(Tag::Link {
                    link_type,
                    dest_url,
                    title,
                    id,
                }) => {
                    if let Some(rewritten_url) = self.try_rewrite_assets_link(&dest_url) {
                        // Rewrite asset links to point to the correct assets
                        // directory
                        iterator.push(Event::Start(Tag::Link {
                            link_type,
                            dest_url: rewritten_url.into(),
                            title,
                            id,
                        }));
                    } else {
                        // Fallback to original link
                        iterator.push(Event::Start(Tag::Link {
                            link_type,
                            dest_url,
                            title,
                            id,
                        }));
                    }
                }
                Event::Start(Tag::Image {
                    link_type,
                    dest_url,
                    title,
                    id,
                }) => {
                    // Handle image links
                    if let Some(rewritten_url) = self.try_rewrite_assets_link(&dest_url) {
                        // Rewrite asset links to point to the correct assets
                        // directory
                        iterator.push(Event::Start(Tag::Image {
                            link_type,
                            dest_url: rewritten_url.into(),
                            title,
                            id,
                        }));
                    } else {
                        // Fallback to original image link
                        iterator.push(Event::Start(Tag::Image {
                            link_type,
                            dest_url,
                            title,
                            id,
                        }));
                    }
                }
                _ => iterator.push(e),
            }
        }

        if about_section_open {
            iterator.push(Event::Html("</div>".into()));
        }

        html::push_html(&mut html_output, iterator.into_iter());
        let html_output = inject_heading_ids(html_output, headings);
        let html_output = inject_heading_kickers(html_output);

        format!(r#"<div class="markdown-body">{html_output}</div>"#)
    }

    fn try_rewrite_assets_link(&self, link: &str) -> Option<String> {
        let site_config = SITE_CONFIGURATION
            .get()
            .expect("Site configuration should be loaded by AppLayout");
        if let Some((_, asset_path)) = link.split_once("$ASSETS/") {
            if !asset_path.is_empty() {
                return Some(site_config.resolve_article_asset(&self.id, asset_path));
            }
        }
        None
    }
}

fn heading_level(level: pulldown_cmark::HeadingLevel) -> u8 {
    match level {
        pulldown_cmark::HeadingLevel::H1 => 1,
        pulldown_cmark::HeadingLevel::H2 => 2,
        pulldown_cmark::HeadingLevel::H3 => 3,
        pulldown_cmark::HeadingLevel::H4 => 4,
        pulldown_cmark::HeadingLevel::H5 => 5,
        pulldown_cmark::HeadingLevel::H6 => 6,
    }
}

fn slugify(title: &str) -> String {
    let mut slug = String::new();
    for character in title.chars() {
        if character.is_alphanumeric() {
            slug.extend(character.to_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }

    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "section".to_string()
    } else {
        slug.to_string()
    }
}

fn inject_heading_ids(mut html: String, headings: &[MarkdownHeading]) -> String {
    let mut search_start = 0;
    for heading in headings {
        let marker = format!("<h{}>", heading.level);
        let Some(relative_position) = html[search_start..].find(&marker) else {
            continue;
        };
        let position = search_start + relative_position;
        let replacement = format!("<h{} id=\"{}\">", heading.level, heading.id);
        html.replace_range(position..position + marker.len(), &replacement);
        search_start = position + replacement.len();
    }
    html
}

fn inject_heading_kickers(mut html: String) -> String {
    const MARKER: &str = "<!-- kicker:";
    let mut search_start = 0;

    while let Some(relative_start) = html[search_start..].find(MARKER) {
        let start = search_start + relative_start;
        let Some(relative_end) = html[start..].find("-->") else {
            break;
        };
        let end = start + relative_end;
        let kicker = html[start + MARKER.len()..end].trim();
        if kicker.is_empty() {
            search_start = end + 3;
            continue;
        }

        let replacement = format!(
            r#"<span class="markdown-heading-kicker">{}</span>"#,
            escape_html(kicker)
        );
        html.replace_range(start..end + 3, &replacement);
        search_start = start + replacement.len();
    }

    html
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

impl From<MarkdownArticle> for String {
    fn from(val: MarkdownArticle) -> Self {
        val.render_markdown(false)
    }
}

impl Debug for MarkdownArticle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MarkdownArticle(id: {}, content: ...)", self.id)
    }
}

pub trait ToHtml: Mountable {
    fn to_html(&self) -> String {
        self.elements()
            .iter()
            .map(|el| el.outer_html())
            .collect::<Vec<_>>()
            .join("")
    }
}

impl<T: Mountable> ToHtml for T {}

#[cfg(test)]
mod tests {
    use super::MarkdownArticle;

    #[test]
    fn special_article_sections_follow_markdown_headings() {
        let article = MarkdownArticle::new(
            "## Stack\n\n- Rust\n\n## Links\n\n- Articles".to_string(),
            "about".to_string(),
        );
        let about_html = article.render_about();

        assert!(about_html.contains(r#"<div class="about-section" data-section="stack">"#));
        assert!(about_html.contains(r#"<div class="about-section" data-section="links">"#));
        assert!(about_html.contains(r#"<h2 id="stack">Stack</h2>"#));

        let regular_html: String = MarkdownArticle::new(
            "## Stack\n\n- Rust\n\n## Links\n\n- Articles".to_string(),
            "article".to_string(),
        )
        .into();
        assert!(!regular_html.contains("about-section"));
    }

    #[test]
    fn markdown_heading_kicker_markers_render_as_themeable_labels() {
        let html: String = MarkdownArticle::new(
            "<!-- kicker: DATA · GENERATED -->\n\n## 录取数据分析".to_string(),
            "home".to_string(),
        )
        .into();

        assert!(html.contains(r#"<span class="markdown-heading-kicker">DATA · GENERATED</span>"#));
        assert!(html.contains(r#"<h2 id="录取数据分析">录取数据分析</h2>"#));
        assert!(!html.contains("<!-- kicker:"));
    }
}
