use std::iter::Peekable;

use leptos::prelude::*;

use crate::{
    components::directives::DirectiveView,
    markdown::{DirectiveSpec, DocumentBlock, MarkdownDocument},
    utils::MarkdownArticle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownRenderMode {
    Article,
    Home,
}

impl MarkdownRenderMode {
    fn class(self) -> &'static str {
        match self {
            Self::Article => "rich-markdown rich-markdown-article",
            Self::Home => "rich-markdown rich-markdown-home",
        }
    }
}

/// Mount a document as a sequence of Markdown fragments and typed directives.
/// The Markdown fragments remain content-only HTML islands; directive blocks
/// are real Leptos components and never require a post-mount DOM scanner.
#[component]
pub fn RichMarkdownRenderer(
    article_id: String,
    content: String,
    #[prop(default = MarkdownRenderMode::Article)] mode: MarkdownRenderMode,
) -> impl IntoView {
    let document = MarkdownDocument::parse(&content);
    let headings = document.headings.clone();
    let mut heading_offset = 0;
    let mut rendered_blocks = Vec::new();
    let mut source_blocks = document.blocks.into_iter().peekable();
    while let Some(block) = source_blocks.next() {
        match block {
            DocumentBlock::Markdown(markdown) => {
                let heading_count = MarkdownArticle::new(markdown.clone(), article_id.clone())
                    .headings()
                    .len();
                let end = (heading_offset + heading_count).min(headings.len());
                let fragment_headings = headings[heading_offset..end].to_vec();
                heading_offset = end;
                let html = MarkdownArticle::new(markdown, article_id.clone())
                    .render_fragment(&fragment_headings);
                rendered_blocks.push(
                    view! { <div class="markdown-fragment" inner_html=html></div> }.into_any(),
                );
            }
            DocumentBlock::Directive(DirectiveSpec::Pledge(pledge))
                if has_adjacent_related_sites(&mut source_blocks) =>
            {
                let related = match source_blocks.next() {
                    Some(DocumentBlock::Directive(DirectiveSpec::RelatedSites(spec))) => spec,
                    _ => unreachable!("related-sites lookahead must be consumed"),
                };
                rendered_blocks.push(
                    view! {
                        <div class="directive-utility-grid">
                            <DirectiveView
                                article_id=article_id.clone()
                                spec=DirectiveSpec::Pledge(pledge)
                            />
                            <DirectiveView
                                article_id=article_id.clone()
                                spec=DirectiveSpec::RelatedSites(related)
                            />
                        </div>
                    }
                    .into_any(),
                );
            }
            DocumentBlock::Directive(DirectiveSpec::Chart(chart))
                if has_adjacent_chart_grid(&mut source_blocks, chart.grid.as_deref()) =>
            {
                let second = match source_blocks.next() {
                    Some(DocumentBlock::Directive(DirectiveSpec::Chart(chart))) => chart,
                    _ => unreachable!("chart-grid lookahead must be consumed"),
                };
                rendered_blocks.push(
                    view! {
                        <div class="directive-chart-grid">
                            <DirectiveView
                                article_id=article_id.clone()
                                spec=DirectiveSpec::Chart(chart)
                            />
                            <DirectiveView
                                article_id=article_id.clone()
                                spec=DirectiveSpec::Chart(second)
                            />
                        </div>
                    }
                    .into_any(),
                );
            }
            DocumentBlock::Directive(spec) => {
                rendered_blocks.push(
                    view! { <DirectiveView article_id=article_id.clone() spec=spec /> }.into_any(),
                );
            }
        }
    }
    let blocks = rendered_blocks.into_iter().collect_view();

    view! {
        <div class=mode.class()>
            {blocks}
        </div>
    }
}

fn has_adjacent_related_sites(
    source_blocks: &mut Peekable<impl Iterator<Item = DocumentBlock>>,
) -> bool {
    matches!(
        source_blocks.peek(),
        Some(DocumentBlock::Directive(DirectiveSpec::RelatedSites(_)))
    )
}

fn has_adjacent_chart_grid(
    source_blocks: &mut Peekable<impl Iterator<Item = DocumentBlock>>,
    grid: Option<&str>,
) -> bool {
    let Some(grid) = grid else {
        return false;
    };
    matches!(
        source_blocks.peek(),
        Some(DocumentBlock::Directive(DirectiveSpec::Chart(chart)))
            if chart.grid.as_deref() == Some(grid)
    )
}
