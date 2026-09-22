//! Leptos components backed by the closed Markdown directive vocabulary.
//!
//! The components in this module deliberately share one CSV loader and one
//! Markdown-fragment renderer.  The page author edits Markdown/CSV; the Rust
//! layer owns validation, URL resolution, SVG geometry, and graceful errors.

use std::fmt::Write as _;

use csv::ReaderBuilder;
use leptos::prelude::*;

use crate::{
    app::SITE_CONFIGURATION,
    components::articles::list::ArticleCard,
    markdown::{
        parse_markdown_list, parse_markdown_table, ArticleIndexSpec, ChartKind, ChartOrientation,
        ChartSpec, ContributorsSpec, DirectiveSpec, DisclaimerSpec, DistanceFieldSpec, FactsSpec,
        FeaturesSpec, HeroSpec, LabsSpec, LinkListSpec, NetworkSpec, PledgeSpec, StatsSpec,
        StepsSpec, ValueFormat,
    },
    models::ArticleIndex,
    utils::MarkdownArticle,
};

#[derive(Debug, Clone, PartialEq)]
pub struct CsvTable {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
struct DistancePoint {
    name: String,
    office: String,
    distance: f64,
    angle: f64,
}

impl CsvTable {
    fn column(&self, name: &str) -> Option<usize> {
        self.headers.iter().position(|header| header == name)
    }

    fn value<'a>(&'a self, row: &'a [String], column: &str) -> Option<&'a str> {
        self.column(column)
            .and_then(|index| row.get(index))
            .map(String::as_str)
    }

    fn require_column(&self, name: &str) -> Result<usize, String> {
        self.column(name)
            .ok_or_else(|| format!("CSV is missing required column '{name}'"))
    }
}

pub async fn fetch_csv(article_id: &str, source: &str) -> Result<CsvTable, String> {
    let site = SITE_CONFIGURATION
        .get()
        .ok_or_else(|| "site configuration is not ready".to_string())?;
    if source.contains("://") || source.starts_with("//") {
        return Err(format!("remote CSV sources are not allowed: {source}"));
    }
    let url = site.resolve_article_asset(article_id, source);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|error| format!("failed to fetch {source}: {error}"))?;
    let content = response
        .text()
        .await
        .map_err(|error| format!("failed to read {source}: {error}"))?;

    let mut reader = ReaderBuilder::new()
        .flexible(false)
        .trim(csv::Trim::All)
        .from_reader(content.as_bytes());
    let headers = reader
        .headers()
        .map_err(|error| format!("failed to parse {source} headers: {error}"))?
        .iter()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if headers.is_empty() || headers.iter().any(String::is_empty) {
        return Err(format!("{source} has an empty CSV header"));
    }

    let mut rows = Vec::new();
    for (index, record) in reader.records().enumerate() {
        let record = record
            .map_err(|error| format!("failed to parse {source} row {}: {error}", index + 2))?;
        if record.len() != headers.len() {
            return Err(format!(
                "failed to parse {source} row {}: expected {} columns, got {}",
                index + 2,
                headers.len(),
                record.len()
            ));
        }
        rows.push(record.iter().map(ToOwned::to_owned).collect());
    }

    Ok(CsvTable { headers, rows })
}

fn render_body(article_id: &str, body: &str) -> String {
    MarkdownArticle::new(body.to_string(), article_id.to_string()).into()
}

fn notice(label: &str, message: impl Into<String>) -> AnyView {
    view! {
        <div class="directive directive-notice" role="status">
            <span class="directive-notice-label">{label.to_string()}</span>
            <span class="directive-notice-message">{message.into()}</span>
        </div>
    }
    .into_any()
}

#[component]
pub fn DirectiveView(article_id: String, spec: DirectiveSpec) -> impl IntoView {
    match spec {
        DirectiveSpec::Hero(spec) => hero_directive(article_id, spec),
        DirectiveSpec::Stats(spec) => stats_directive(spec),
        DirectiveSpec::Facts(spec) => facts_directive(article_id, spec),
        DirectiveSpec::Chart(spec) => chart_directive(article_id, spec),
        DirectiveSpec::DistanceField(spec) => distance_field_directive(article_id, spec),
        DirectiveSpec::Labs(spec) => labs_directive(spec),
        DirectiveSpec::Features(spec) => features_directive(spec),
        DirectiveSpec::ArticleIndex(spec) => article_index_directive(article_id, spec),
        DirectiveSpec::Contributors(spec) => contributors_directive(article_id, spec),
        DirectiveSpec::Network(spec) => network_directive(article_id, spec),
        DirectiveSpec::Steps(spec) => steps_directive(spec),
        DirectiveSpec::LinkList(spec) | DirectiveSpec::RelatedSites(spec) => {
            link_list_directive(article_id, spec)
        }
        DirectiveSpec::Pledge(spec) => pledge_directive(spec),
        DirectiveSpec::Disclaimer(spec) => disclaimer_directive(article_id, spec),
        DirectiveSpec::Unknown { name, .. } => notice("Unknown directive", format!("::{name}")),
        DirectiveSpec::Invalid { name, message } => notice(&format!("::{name}"), message),
    }
}

fn hero_directive(article_id: String, spec: HeroSpec) -> AnyView {
    let body = render_body(&article_id, &spec.body);
    let title = match spec.title.split_once("报考指南") {
        Some((first, _)) => view! { <>{first}<br/><span>报考指南</span></> }.into_any(),
        None => view! { <>{spec.title}</> }.into_any(),
    };
    view! {
        <section class="directive directive-hero">
            <div class="directive-hero-inner">
                <span class="kicker">{spec.kicker}</span>
                <h1 class="directive-hero-title">{title}</h1>
                <p class="directive-hero-subtitle">{spec.subtitle}</p>
                <div class="directive-hero-body" inner_html=body></div>
            </div>
        </section>
    }
    .into_any()
}

fn stats_directive(spec: StatsSpec) -> AnyView {
    let StatsSpec {
        columns,
        kicker,
        title,
        body,
    } = spec;
    let table = match parse_markdown_table(&body) {
        Ok(table) if table.headers.len() >= 2 => table,
        Ok(_) => {
            return notice(
                "Stats unavailable",
                "The stats table needs value and label columns.",
            )
        }
        Err(error) => return notice("Stats unavailable", error),
    };
    let grid_class = format!(
        "directive-stat-grid directive-grid-columns-{}",
        columns.clamp(1, 6)
    );
    let section_head = match (kicker, title) {
        (Some(kicker), Some(title)) => view! {
            <div class="directive-section-head">
                <div>
                    <span class="kicker">{kicker}</span>
                    <h2 class="directive-section-title">{title}</h2>
                </div>
                <a class="directive-section-link" href="/#data">"查看完整录取分析 →"</a>
            </div>
        }
        .into_any(),
        (Some(kicker), None) => view! {
            <div class="directive-section-head">
                <span class="kicker">{kicker}</span>
            </div>
        }
        .into_any(),
        (None, Some(title)) => view! {
            <div class="directive-section-head">
                <h2 class="directive-section-title">{title}</h2>
            </div>
        }
        .into_any(),
        (None, None) => view! { <></> }.into_any(),
    };
    view! {
        <section class="directive directive-stats" aria-label="Statistics">
            {section_head}
            <div class=grid_class>
                {table.rows.into_iter().map(|row| {
                    let value = row.first().cloned().unwrap_or_default();
                    let label = row.get(1).cloned().unwrap_or_default();
                    view! {
                        <div class="directive-stat">
                            <strong>{value}</strong>
                            <span>{label}</span>
                        </div>
                    }
                }).collect_view()}
            </div>
        </section>
    }
    .into_any()
}

fn facts_directive(_article_id: String, spec: FactsSpec) -> AnyView {
    let table = match parse_markdown_table(&spec.body) {
        Ok(table) if table.headers.len() >= 2 => table,
        Ok(_) => {
            return notice(
                "Facts unavailable",
                "The facts table needs label and value columns.",
            )
        }
        Err(error) => return notice("Facts unavailable", error),
    };
    view! {
        <div class="directive directive-facts">
            <div class="directive-facts-grid">
                {table.rows.into_iter().map(|row| {
                    let label = row.first().cloned().unwrap_or_default();
                    let value = row.get(1).cloned().unwrap_or_default();
                    view! {
                        <div class="directive-fact">
                            <dt>{label}</dt>
                            <dd>{value}</dd>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
    .into_any()
}

fn labs_directive(spec: LabsSpec) -> AnyView {
    let labs = parse_markdown_list(&spec.body);
    if labs.is_empty() {
        return notice(
            "Labs unavailable",
            "Add one Markdown list item per laboratory.",
        );
    }
    view! {
        <div class="directive directive-labs">
            {labs.into_iter().map(|lab| view! { <span>{lab}</span> }).collect_view()}
        </div>
    }
    .into_any()
}

fn features_directive(spec: FeaturesSpec) -> AnyView {
    let table = match parse_markdown_table(&spec.body) {
        Ok(table) if table.headers.len() >= 2 => table,
        Ok(_) => {
            return notice(
                "Features unavailable",
                "The features table needs title and description columns.",
            )
        }
        Err(error) => return notice("Features unavailable", error),
    };
    view! {
        <div class="directive directive-features">
            {table.rows.into_iter().enumerate().map(|(index, row)| {
                let title = row.first().cloned().unwrap_or_default();
                let description = row.get(1).cloned().unwrap_or_default();
                view! {
                    <div class="directive-feature">
                        <span class="directive-feature-key">{format!("{:02}", index + 1)}</span>
                        <div>
                            <strong>{title}</strong>
                            <p>{description}</p>
                        </div>
                    </div>
                }
            }).collect_view()}
        </div>
    }
    .into_any()
}

fn link_list_directive(article_id: String, spec: LinkListSpec) -> AnyView {
    let body = render_body(&article_id, &spec.body);
    let class = format!("directive directive-link-list {}", spec.variant);
    let title = spec.title;
    view! {
        <div class=class>
            {title.map(|title| view! { <h2 class="directive-section-title directive-link-list-title">{title}</h2> })}
            <div class="directive-link-list-body" inner_html=body></div>
        </div>
    }
    .into_any()
}

fn disclaimer_directive(article_id: String, spec: DisclaimerSpec) -> AnyView {
    let body = render_body(&article_id, &spec.body);
    view! { <aside class="directive directive-disclaimer" inner_html=body></aside> }.into_any()
}

fn steps_directive(spec: StepsSpec) -> AnyView {
    let steps = parse_steps(&spec.body);
    if steps.is_empty() {
        return notice(
            "Steps unavailable",
            "Add an ordered Markdown list to the steps directive.",
        );
    }
    view! {
        <ol class="directive directive-steps">
            {steps.into_iter().enumerate().map(|(index, (title, description))| view! {
                <li>
                    <span class="directive-step-number">{format!("{:02}", index + 1)}</span>
                    <span class="directive-step-content">
                        <strong>{title}</strong>
                        <span>{description}</span>
                    </span>
                </li>
            }).collect_view()}
        </ol>
    }
    .into_any()
}

fn parse_steps(body: &str) -> Vec<(String, String)> {
    body.lines()
        .filter_map(|line| {
            let line = line.trim();
            let (_, rest) = line.split_once('.')?;
            let rest = rest.trim();
            if rest.is_empty() {
                return None;
            }
            let (title, description) = rest
                .split_once('—')
                .or_else(|| rest.split_once(" - "))
                .map(|(title, description)| (title, description))
                .unwrap_or((rest, ""));
            Some((strip_emphasis(title.trim()), description.trim().to_string()))
        })
        .collect()
}

fn strip_emphasis(value: &str) -> String {
    value.trim_matches('*').trim_matches('_').trim().to_string()
}

fn article_index_directive(_article_id: String, spec: ArticleIndexSpec) -> AnyView {
    let site = match SITE_CONFIGURATION.get() {
        Some(site) => site.clone(),
        None => return notice("Articles unavailable", "site configuration is not ready"),
    };
    let resource = LocalResource::new(move || {
        let site = site.clone();
        async move {
            ArticleIndex::fetch(&site)
                .await
                .map(|index| index.to_search_index())
        }
    });
    let category = spec.category.clone();
    let tags = spec.tags.clone();
    let limit = spec.limit;
    let show_description = spec.show_description;

    view! {
        <section class="directive directive-article-index">
            <Suspense fallback=|| view! { <div class="directive-loading">"Loading articles…"</div> }>
                {move || resource.get().map(|result| match result {
                    Ok(index) => {
                        let articles = index.articles.into_iter().filter(|article| {
                            let category_matches = category.as_ref().is_none_or(|expected| {
                                article.article.category.as_ref().is_some_and(|actual| {
                                    actual.to_lowercase().contains(&expected.to_lowercase())
                                })
                            });
                            let tags_match = tags.is_empty() || tags.iter().any(|expected| {
                                article.article.tags.iter().any(|actual| {
                                    actual.to_lowercase().contains(&expected.to_lowercase())
                                })
                            });
                            category_matches && tags_match
                        }).take(limit).collect::<Vec<_>>();
                        if articles.is_empty() {
                            view! { <p class="muted">"No matching experience articles yet."</p> }.into_any()
                        } else {
                            view! {
                                <ul class="directive-article-index-list">
                                    {articles.into_iter().map(|article| {
                                        view! { <ArticleCard article=article show_description=show_description /> }
                                    }).collect_view()}
                                </ul>
                            }.into_any()
                        }
                    }
                    Err(error) => notice("Articles unavailable", error),
                })}
            </Suspense>
        </section>
    }
    .into_any()
}

fn contributors_directive(article_id: String, spec: ContributorsSpec) -> AnyView {
    let source = spec.source.clone();
    let title = spec.title.clone();
    let avatar_size = spec.avatar_size;
    let label_size = spec.label_size.clone();
    let body = render_body(&article_id, &spec.body);
    let resource = LocalResource::new(move || {
        let source = source.clone();
        let article_id = article_id.clone();
        async move { fetch_csv(&article_id, &source).await }
    });

    view! {
        <section class="directive directive-contributors">
            {title.map(|title| view! { <h3 class="directive-subtitle">{title}</h3> })}
            <Suspense fallback=|| view! { <div class="directive-loading">"Loading contributors…"</div> }>
                {move || resource.get().map(|result| match result {
                    Ok(table) => match contributor_rows(&table, avatar_size) {
                        Ok(rows) if !rows.is_empty() => view! {
                            <div
                                class="directive-contributor-grid"
                                style=format!("--contributor-avatar-size: {avatar_size}px; --contributor-label-size: {label_size};")
                            >
                                {rows.into_iter().map(|(login, display_name)| view! {
                                    <a
                                        class="directive-contributor"
                                        href=format!("https://github.com/{login}")
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        title=display_name.clone()
                                    >
                                        <img
                                            width=avatar_size.to_string()
                                            height=avatar_size.to_string()
                                            src=format!("https://github.com/{login}.png?size={avatar_size}")
                                            alt=display_name.clone()
                                            loading="lazy"
                                        />
                                        <span>{display_name.clone()}</span>
                                    </a>
                                }).collect_view()}
                            </div>
                        }.into_any(),
                        Ok(_) => notice("Contributors unavailable", "The contributor CSV is empty."),
                        Err(error) => notice("Contributors unavailable", error),
                    },
                    Err(error) => notice("Contributors unavailable", error),
                })}
            </Suspense>
            <div class="directive-contributor-note" inner_html=body></div>
        </section>
    }
    .into_any()
}

fn contributor_rows(table: &CsvTable, _avatar_size: u16) -> Result<Vec<(String, String)>, String> {
    table.require_column("login")?;
    table.require_column("display_name")?;
    table
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let login = table
                .value(row, "login")
                .filter(|value| !value.is_empty())
                .ok_or_else(|| format!("contributors.csv row {} has no login", index + 2))?;
            let display_name = table
                .value(row, "display_name")
                .filter(|value| !value.is_empty())
                .unwrap_or(login);
            Ok((login.to_string(), display_name.to_string()))
        })
        .collect()
}

fn network_directive(article_id: String, spec: NetworkSpec) -> AnyView {
    let source = spec.source.clone();
    let center = spec.center.clone();
    let mode = spec.mode.clone();
    let article_id_for_resource = article_id.clone();
    let resource = LocalResource::new(move || {
        let source = source.clone();
        let article_id = article_id_for_resource.clone();
        async move { fetch_csv(&article_id, &source).await }
    });

    view! {
        <section class="directive directive-network">
            <Suspense fallback=|| view! { <div class="directive-loading">"Loading network…"</div> }>
                {move || resource.get().map(|result| match result {
                    Ok(table) => match network_svg(&table, &center) {
                        Ok(svg) => view! {
                            <div class="directive-svg-host" inner_html=svg></div>
                            <p class="directive-caption">{format!("QUALITATIVE ONLY · NODE SIZE DOES NOT ENCODE COUNT · {mode}")}</p>
                        }.into_any(),
                        Err(error) => notice("Network unavailable", error),
                    },
                    Err(error) => notice("Network unavailable", error),
                })}
            </Suspense>
        </section>
    }
    .into_any()
}

fn pledge_directive(spec: PledgeSpec) -> AnyView {
    let key = spec.storage_key.clone();
    let effect_key = key.clone();
    let title = spec.title.clone();
    let label = spec.label.clone();
    let button_label = label.clone();
    let body = spec.body.clone();
    let (pledged, set_pledged) = signal(false);
    Effect::new(move |_| {
        let value = web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten())
            .and_then(|storage| storage.get_item(&effect_key).ok().flatten())
            .is_some_and(|value| value == "true");
        set_pledged.set(value);
    });
    let toggle = move |_| {
        let next = !pledged.get_untracked();
        set_pledged.set(next);
        if let Some(storage) =
            web_sys::window().and_then(|window| window.local_storage().ok().flatten())
        {
            let _ = storage.set_item(&key, if next { "true" } else { "false" });
        }
    };

    view! {
        <section class="directive directive-pledge">
            {title.map(|title| view! { <h3 class="directive-utility-title">{title}</h3> })}
            <div class="directive-pledge-row">
                <button type="button" on:click=toggle aria-pressed=move || pledged.get().to_string()>
                    {move || {
                        if pledged.get() {
                            "✓ 已效忠".to_string()
                        } else {
                            button_label.clone()
                        }
                    }}
                </button>
                <div>
                    <p>{body}</p>
                </div>
            </div>
        </section>
    }
    .into_any()
}

fn chart_directive(article_id: String, spec: ChartSpec) -> AnyView {
    let source = spec.source.clone();
    let article_id_for_resource = article_id.clone();
    let resource = LocalResource::new(move || {
        let source = source.clone();
        let article_id = article_id_for_resource.clone();
        async move { fetch_csv(&article_id, &source).await }
    });
    let title = spec.title.clone();
    let subtitle = spec.subtitle.clone();
    let legend = spec.series.clone();
    let legend_is_pie = matches!(spec.kind, ChartKind::Pie | ChartKind::Donut);

    view! {
        <figure class="directive directive-chart">
            {title.map(|title| view! { <h3 class="directive-chart-title">{title}</h3> })}
            {subtitle.map(|subtitle| view! { <p class="directive-chart-subtitle">{subtitle}</p> })}
            <Suspense fallback=|| view! { <div class="directive-loading">"Loading chart…"</div> }>
                {move || resource.get().map(|result| match result {
                    Ok(table) => match chart_svg(&spec, &table) {
                        Ok(svg) => view! {
                            {(!legend_is_pie).then(|| view! {
                                <div class="directive-chart-legend">
                                    {legend.iter().enumerate().map(|(index, series)| view! {
                                        <span><i class=format!("chart-legend-swatch chart-series-{index}")></i>{series.label.clone()}</span>
                                    }).collect_view()}
                                </div>
                            })}
                            <div class="directive-svg-host" inner_html=svg></div>
                        }.into_any(),
                        Err(error) => notice("Chart unavailable", error),
                    },
                    Err(error) => notice("Chart unavailable", error),
                })}
            </Suspense>
        </figure>
    }
    .into_any()
}

fn chart_svg(spec: &ChartSpec, table: &CsvTable) -> Result<String, String> {
    match spec.kind {
        ChartKind::Line | ChartKind::Bar => cartesian_chart_svg(spec, table),
        ChartKind::Pie | ChartKind::Donut => circular_chart_svg(spec, table),
    }
}

fn cartesian_chart_svg(spec: &ChartSpec, table: &CsvTable) -> Result<String, String> {
    let x_column = spec
        .x
        .as_deref()
        .ok_or_else(|| "chart is missing x column".to_string())?;
    let x_index = table.require_column(x_column)?;
    let series_indices = spec
        .series
        .iter()
        .map(|series| table.require_column(&series.column))
        .collect::<Result<Vec<_>, _>>()?;
    if table.rows.is_empty() {
        return Err("CSV contains no data rows".to_string());
    }

    let mut values = Vec::with_capacity(table.rows.len());
    for (row_index, row) in table.rows.iter().enumerate() {
        let mut row_values = Vec::with_capacity(series_indices.len());
        for (series, index) in spec.series.iter().zip(&series_indices) {
            let raw = row
                .get(*index)
                .ok_or_else(|| format!("missing {} at row {}", series.column, row_index + 2))?;
            let value = raw.parse::<f64>().map_err(|_| {
                format!(
                    "invalid number in {} at row {}: {raw}",
                    series.column,
                    row_index + 2
                )
            })?;
            if !value.is_finite() {
                return Err(format!(
                    "non-finite number in {} at row {}",
                    series.column,
                    row_index + 2
                ));
            }
            row_values.push(value);
        }
        values.push((row.get(x_index).cloned().unwrap_or_default(), row_values));
    }

    let all_values = values
        .iter()
        .flat_map(|(_, row)| row.iter().copied())
        .collect::<Vec<_>>();
    let data_min = all_values.iter().copied().fold(f64::INFINITY, f64::min);
    let data_max = all_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut min = spec
        .min
        .unwrap_or(if data_min > 0.0 { 0.0 } else { data_min });
    let mut max = spec.max.unwrap_or(data_max);
    if !min.is_finite() || !max.is_finite() {
        return Err("chart range is not finite".to_string());
    }
    if (max - min).abs() < f64::EPSILON {
        let padding = max.abs().max(1.0) * 0.1;
        min -= padding;
        max += padding;
    }
    if min >= max {
        return Err("chart min must be smaller than max".to_string());
    }

    const WIDTH: f64 = 1000.0;
    const HEIGHT: f64 = 360.0;
    const RIGHT: f64 = 26.0;
    const TOP: f64 = 24.0;
    const BOTTOM: f64 = 62.0;
    let left = if spec.orientation == ChartOrientation::Horizontal {
        190.0
    } else {
        64.0
    };
    let plot_width = WIDTH - left - RIGHT;
    let plot_height = HEIGHT - TOP - BOTTOM;
    let x = |index: usize| {
        if values.len() <= 1 {
            left + plot_width / 2.0
        } else {
            left + index as f64 * plot_width / (values.len() - 1) as f64
        }
    };
    let y = |value: f64| TOP + plot_height - (value - min) / (max - min) * plot_height;
    let axis_ticks = (0..=4)
        .map(|tick| min + (max - min) * f64::from(tick) / 4.0)
        .collect::<Vec<_>>();
    let mut svg = svg_start(
        WIDTH,
        HEIGHT,
        spec.title.as_deref().unwrap_or("Chart"),
        "Data-driven chart",
    );

    for tick in &axis_ticks {
        if spec.orientation == ChartOrientation::Horizontal {
            let x = left + (*tick - min) / (max - min) * plot_width;
            let _ = write!(
                svg,
                r#"<line class="chart-grid-line chart-grid-line-vertical" x1="{x:.2}" y1="{TOP:.2}" x2="{x:.2}" y2="{y2:.2}"/><text class="chart-axis-label chart-axis-label-x" x="{x:.2}" y="{label_y:.2}" text-anchor="middle">{label}</text>"#,
                y2 = TOP + plot_height,
                label_y = HEIGHT - 20.0,
                label = escape_html(&format_axis_tick(*tick, spec)),
            );
        } else {
            let y = y(*tick);
            let _ = write!(
                svg,
                r#"<line class="chart-grid-line" x1="{left:.2}" y1="{y:.2}" x2="{x2:.2}" y2="{y:.2}"/><text class="chart-axis-label chart-axis-label-y" x="{label_x:.2}" y="{label_y:.2}" text-anchor="end">{label}</text>"#,
                x2 = WIDTH - RIGHT,
                label_x = left - 12.0,
                label_y = y + 4.0,
                label = escape_html(&format_axis_tick(*tick, spec)),
            );
        }
    }

    match spec.kind {
        ChartKind::Line => {
            for (series_index, _series) in spec.series.iter().enumerate() {
                let path = values
                    .iter()
                    .enumerate()
                    .map(|(index, (_, row))| {
                        let command = if index == 0 { "M" } else { "L" };
                        format!("{command} {:.2} {:.2}", x(index), y(row[series_index]))
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                let _ = write!(
                    svg,
                    r#"<path class="chart-series chart-line chart-series-{series_index}" d="{path}"/><g class="chart-points">"#
                );
                for (index, (_, row)) in values.iter().enumerate() {
                    let _ = write!(
                        svg,
                        r#"<circle class="chart-series chart-point chart-series-{series_index}" cx="{x:.2}" cy="{y:.2}" r="4"/>"#,
                        x = x(index),
                        y = y(row[series_index]),
                    );
                    if spec.show_values && (spec.series.len() == 1 || series_index == 0) {
                        let _ = write!(
                            svg,
                            r#"<text class="chart-value chart-series-{series_index}" x="{x:.2}" y="{label_y:.2}" text-anchor="middle">{value}</text>"#,
                            x = x(index),
                            label_y = y(row[series_index]) - 11.0,
                            value = escape_html(&format_value(row[series_index], spec)),
                        );
                    }
                }
                svg.push_str("</g>");
            }
        }
        ChartKind::Bar => {
            let group_width = plot_width / values.len() as f64;
            if spec.orientation == ChartOrientation::Vertical {
                let bar_width = (group_width * 0.72 / spec.series.len() as f64).max(3.0);
                for (row_index, (_, row)) in values.iter().enumerate() {
                    for (series_index, value) in row.iter().enumerate() {
                        let bar_x = left
                            + group_width * row_index as f64
                            + group_width * 0.14
                            + bar_width * series_index as f64;
                        let zero_y = y(0.0_f64.clamp(min, max));
                        let value_y = y(*value);
                        let top = value_y.min(zero_y);
                        let height = (value_y - zero_y).abs().max(1.0);
                        let _ = write!(
                            svg,
                            r#"<rect class="chart-series chart-bar chart-series-{series_index}" x="{bar_x:.2}" y="{top:.2}" width="{bar_width:.2}" height="{height:.2}" rx="4"/>"#
                        );
                        if spec.show_values && (spec.series.len() == 1 || series_index == 0) {
                            let _ = write!(
                                svg,
                                r#"<text class="chart-value chart-series-{series_index}" x="{x:.2}" y="{label_y:.2}" text-anchor="middle">{value}</text>"#,
                                x = bar_x + bar_width / 2.0,
                                label_y = top - 7.0,
                                value = escape_html(&format_value(*value, spec)),
                            );
                        }
                    }
                }
            } else {
                let row_height = plot_height / values.len() as f64;
                for (row_index, (label, row)) in values.iter().enumerate() {
                    let group_height = row_height * 0.68;
                    let base_y =
                        TOP + row_height * row_index as f64 + (row_height - group_height) / 2.0;
                    let bar_height = (group_height / spec.series.len() as f64).max(3.0);
                    let _ = write!(
                        svg,
                        r#"<text class="chart-axis-label chart-axis-label-left" x="{label_x:.2}" y="{label_y:.2}" text-anchor="end">{label}</text>"#,
                        label_x = left - 12.0,
                        label_y = TOP + row_height * row_index as f64 + row_height / 2.0 + 4.0,
                        label = escape_html(label),
                    );
                    for (series_index, value) in row.iter().enumerate() {
                        let width = ((*value - min) / (max - min) * plot_width).max(0.0);
                        let bar_y = base_y + bar_height * series_index as f64;
                        let _ = write!(
                            svg,
                            r#"<rect class="chart-series chart-bar chart-series-{series_index}" x="{left:.2}" y="{bar_y:.2}" width="{width:.2}" height="{bar_height:.2}" rx="4"/>"#
                        );
                        if spec.show_values && (spec.series.len() == 1 || series_index == 0) {
                            let _ = write!(
                                svg,
                                r#"<text class="chart-value chart-series-{series_index}" x="{x:.2}" y="{label_y:.2}" dominant-baseline="middle">{value}</text>"#,
                                x = left + width + 8.0,
                                label_y = bar_y + bar_height / 2.0,
                                value = escape_html(&format_value(*value, spec)),
                            );
                        }
                    }
                }
            }
        }
        ChartKind::Pie | ChartKind::Donut => unreachable!("handled by circular_chart_svg"),
    }

    if spec.orientation == ChartOrientation::Vertical {
        for (index, (label, _)) in values.iter().enumerate() {
            let _ = write!(
                svg,
                r#"<text class="chart-axis-label chart-axis-label-x" x="{x:.2}" y="{y:.2}" text-anchor="middle">{label}</text>"#,
                x = x(index),
                y = HEIGHT - 20.0,
                label = escape_html(label),
            );
        }
    }
    if let Some(unit) = &spec.unit {
        let _ = write!(
            svg,
            r#"<text class="chart-unit" x="{x:.2}" y="{y:.2}">{unit}</text>"#,
            x = left,
            y = 13.0,
            unit = escape_html(unit)
        );
    }
    svg.push_str("</svg>");
    Ok(svg)
}

fn circular_chart_svg(spec: &ChartSpec, table: &CsvTable) -> Result<String, String> {
    let name_column = spec
        .name
        .as_deref()
        .ok_or_else(|| "pie chart is missing name column".to_string())?;
    let value_column = spec
        .value
        .as_deref()
        .ok_or_else(|| "pie chart is missing value column".to_string())?;
    table.require_column(name_column)?;
    table.require_column(value_column)?;
    let mut entries = Vec::new();
    for (index, row) in table.rows.iter().enumerate() {
        let name = table
            .value(row, name_column)
            .unwrap_or_default()
            .to_string();
        let raw = table.value(row, value_column).unwrap_or_default();
        let value = raw.parse::<f64>().map_err(|_| {
            format!(
                "invalid number in {value_column} at row {}: {raw}",
                index + 2
            )
        })?;
        if !value.is_finite() || value < 0.0 {
            return Err(format!(
                "invalid non-negative value in {value_column} at row {}",
                index + 2
            ));
        }
        entries.push((name, value));
    }
    let total = entries.iter().map(|(_, value)| value).sum::<f64>();
    if total <= 0.0 {
        return Err("pie chart contains no positive values".to_string());
    }

    const WIDTH: f64 = 720.0;
    const HEIGHT: f64 = 360.0;
    const CX: f64 = 220.0;
    const CY: f64 = 180.0;
    const RADIUS: f64 = 112.0;
    let mut svg = svg_start(
        WIDTH,
        HEIGHT,
        spec.title.as_deref().unwrap_or("Chart"),
        "Composition chart",
    );
    let mut angle = -std::f64::consts::FRAC_PI_2;
    for (index, (name, value)) in entries.iter().enumerate() {
        let sweep = value / total * std::f64::consts::TAU;
        let end = angle + sweep;
        let class = format!("chart-series chart-series-{}", index % 6);
        if spec.kind == ChartKind::Donut {
            let circumference = std::f64::consts::TAU * RADIUS;
            let dash = circumference * value / total;
            let offset =
                circumference * (angle + std::f64::consts::FRAC_PI_2) / std::f64::consts::TAU;
            let _ = write!(
                svg,
                r#"<circle class="{class} chart-donut" cx="{CX}" cy="{CY}" r="{RADIUS}" stroke-dasharray="{dash:.2} {rest:.2}" stroke-dashoffset="-{offset:.2}"/>"#,
                rest = circumference - dash,
            );
        } else {
            let path = pie_slice_path(CX, CY, RADIUS, angle, end);
            let _ = write!(svg, r#"<path class="{class} chart-pie" d="{path}"/>"#);
        }
        let legend_y = 92.0 + index as f64 * 35.0;
        let _ = write!(
            svg,
            r#"<circle class="{class} chart-pie-legend-swatch" cx="{x}" cy="{y}" r="5"/><text class="{class} chart-pie-legend-label" x="{text_x}" y="{text_y}">{name} · {value}</text>"#,
            x = 420.0,
            y = legend_y - 4.0,
            text_x = 434.0,
            text_y = legend_y,
            name = escape_html(name),
            value = escape_html(&format_value(*value, spec)),
        );
        angle = end;
    }
    if spec.kind == ChartKind::Donut {
        let center_label = spec.center.as_deref().unwrap_or("Total").replace(' ', "");
        let center_value = format!("{} 人", format_number(total, ValueFormat::Integer));
        let _ = write!(
            svg,
            r#"<text class="chart-center-label" x="{CX}" y="{y1}">{label}</text><text class="chart-center-value" x="{CX}" y="{y2}">{value}</text>"#,
            y1 = CY - 8.0,
            y2 = CY + 16.0,
            label = escape_html(&center_label),
            value = escape_html(&center_value),
        );
    }
    svg.push_str("</svg>");
    Ok(svg)
}

fn pie_slice_path(cx: f64, cy: f64, radius: f64, start: f64, end: f64) -> String {
    let start_x = cx + radius * start.cos();
    let start_y = cy + radius * start.sin();
    let end_x = cx + radius * end.cos();
    let end_y = cy + radius * end.sin();
    let large_arc = if end - start > std::f64::consts::PI {
        1
    } else {
        0
    };
    format!(
        "M {cx:.2} {cy:.2} L {start_x:.2} {start_y:.2} A {radius:.2} {radius:.2} 0 {large_arc} 1 \
         {end_x:.2} {end_y:.2} Z"
    )
}

fn svg_start(width: f64, height: f64, title: &str, description: &str) -> String {
    format!(
        r#"<svg class="directive-svg" viewBox="0 0 {width:.0} {height:.0}" role="img" aria-label="{title}"><title>{title}</title><desc>{description}</desc>"#,
        title = escape_html(title),
        description = escape_html(description),
    )
}

fn format_value(value: f64, spec: &ChartSpec) -> String {
    let number = format_number(value, spec.value_format);
    match spec.unit.as_deref() {
        Some(unit) => format!("{number}{unit}"),
        None => number,
    }
}

fn format_axis_tick(value: f64, spec: &ChartSpec) -> String {
    let number = format_number(
        value,
        if spec.unit.is_some() {
            ValueFormat::Integer
        } else {
            spec.value_format
        },
    );
    match spec.unit.as_deref() {
        Some(unit) => format!("{number}{unit}"),
        None => number,
    }
}

fn format_number(value: f64, format: ValueFormat) -> String {
    match format {
        ValueFormat::Integer => format!("{value:.0}"),
        ValueFormat::Decimal1 => format!("{value:.1}"),
        ValueFormat::Raw => {
            let text = format!("{value:.3}");
            text.trim_end_matches('0').trim_end_matches('.').to_string()
        }
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn distance_field_directive(article_id: String, spec: DistanceFieldSpec) -> AnyView {
    let source = spec.source.clone();
    let article_id_for_resource = article_id.clone();
    let resource = LocalResource::new(move || {
        let source = source.clone();
        let article_id = article_id_for_resource.clone();
        async move { fetch_csv(&article_id, &source).await }
    });
    let title = spec.title.clone();
    let center = spec.center.clone();
    let max_distance = spec.max_distance;
    let unit = spec.unit.clone();

    view! {
        <section class="directive directive-distance-field">
            <Suspense fallback=|| view! { <div class="directive-loading">"Loading distance field…"</div> }>
                {move || resource.get().map(|result| match result {
                    Ok(table) => match distance_points(&table).and_then(|points| {
                        let svg = distance_svg(&points, &center, max_distance, &unit, &title)?;
                        Ok((svg, points))
                    }) {
                        Ok((svg, points)) => {
                            let mut bar_points = points.clone();
                            bar_points.sort_by(|left, right| {
                                left.distance
                                    .partial_cmp(&right.distance)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            });
                            let bar_max = bar_points
                                .iter()
                                .map(|point| point.distance)
                                .fold(0.0_f64, f64::max)
                                .max(f64::EPSILON);
                            let bars = bar_points
                                .into_iter()
                                .enumerate()
                                .map(|(index, point)| {
                                    let width = point.distance / bar_max * 100.0;
                                    let aria_label = format!(
                                        "{} {:.2} {}",
                                        point.name, point.distance, unit
                                    );
                                    view! {
                                        <div class="directive-distance-bar">
                                            <span class="directive-distance-rank">
                                                {format!("{:02}", index + 1)}
                                            </span>
                                            <span class="directive-distance-name">{point.name}</span>
                                            <span
                                                class="directive-distance-track"
                                                role="img"
                                                aria-label=aria_label
                                            >
                                                <i
                                                    aria-hidden="true"
                                                    style=format!("--distance-bar-width: {width:.1}%")
                                                ></i>
                                            </span>
                                            <span class="directive-distance-value">
                                                {format!("{:.2} {}", point.distance, unit)}
                                            </span>
                                        </div>
                                    }
                                })
                                .collect_view();
                            view! {
                                <div class="directive-distance-layout">
                                    <div class="directive-distance-radial">
                                        <div class="directive-svg-host" inner_html=svg></div>
                                        <p class="directive-caption">
                                            "DISTANCE FIELD · SCHEMATIC / NOT GEOGRAPHIC BEARING"
                                        </p>
                                    </div>
                                    <div class="directive-distance-bars">
                                        <div class="directive-distance-bar-list">{bars}</div>
                                    </div>
                                </div>
                            }
                            .into_any()
                        }
                        Err(error) => notice("Distance field unavailable", error),
                    },
                    Err(error) => notice("Distance field unavailable", error),
                })}
            </Suspense>
        </section>
    }
    .into_any()
}

fn distance_points(table: &CsvTable) -> Result<Vec<DistancePoint>, String> {
    table.require_column("name")?;
    table.require_column("distance_km")?;
    let has_angle = table.column("angle_deg").is_some();
    let count = table.rows.len().max(1);
    table
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let name = table.value(row, "name").unwrap_or_default().to_string();
            let office = table.value(row, "office").unwrap_or_default().to_string();
            let raw_distance = table.value(row, "distance_km").unwrap_or_default();
            let distance = raw_distance
                .parse::<f64>()
                .map_err(|_| format!("invalid distance_km at row {}: {raw_distance}", index + 2))?;
            if !distance.is_finite() || distance < 0.0 {
                return Err(format!(
                    "distance_km must be a non-negative finite number at row {}",
                    index + 2
                ));
            }
            let angle = if has_angle {
                table
                    .value(row, "angle_deg")
                    .unwrap_or_default()
                    .parse::<f64>()
                    .map_err(|_| format!("invalid angle_deg at row {}", index + 2))?
            } else {
                index as f64 / count as f64 * 360.0
            };
            if !angle.is_finite() {
                return Err(format!("angle_deg must be finite at row {}", index + 2));
            }
            Ok(DistancePoint {
                name,
                office,
                distance,
                angle,
            })
        })
        .collect()
}

fn distance_svg(
    points: &[DistancePoint],
    center: &str,
    max_distance: f64,
    unit: &str,
    title: &str,
) -> Result<String, String> {
    const WIDTH: f64 = 900.0;
    const HEIGHT: f64 = 470.0;
    const CX: f64 = 450.0;
    const CY: f64 = 235.0;
    const INNER_RADIUS: f64 = 48.0;
    const OUTER_RADIUS: f64 = 193.0;
    let mut svg = svg_start(WIDTH, HEIGHT, title, "Schematic distance field");
    for ring in [0.5_f64, 1.0, 1.5, 2.0, 2.5] {
        let radius =
            INNER_RADIUS + (ring.min(max_distance) / max_distance) * (OUTER_RADIUS - INNER_RADIUS);
        let _ = write!(
            svg,
            r#"<circle class="distance-ring" cx="{CX}" cy="{CY}" r="{radius:.2}"/><text class="distance-ring-label" x="{CX:.2}" y="{label_y:.2}" text-anchor="middle">{ring:.1} {unit}</text>"#,
            label_y = CY - radius - 8.0,
        );
    }

    let label_y = distance_label_positions(points, CX, CY, 74.0, 396.0);
    for (index, point) in points.iter().enumerate() {
        let radius = INNER_RADIUS
            + (point.distance.min(max_distance) / max_distance) * (OUTER_RADIUS - INNER_RADIUS);
        let radians = point.angle.to_radians();
        let x = CX + radians.cos() * radius;
        let y = CY + radians.sin() * radius;
        let anchor = if x >= CX { "start" } else { "end" };
        let text_x = x + if x >= CX { 12.0 } else { -12.0 };
        let leader_x = text_x + if x >= CX { -5.0 } else { 5.0 };
        let _ = write!(
            svg,
            r#"<line class="distance-spoke" x1="{CX}" y1="{CY}" x2="{x:.2}" y2="{y:.2}"/><line class="distance-leader" x1="{x:.2}" y1="{y:.2}" x2="{leader_x:.2}" y2="{label_y:.2}"/><circle class="distance-point" cx="{x:.2}" cy="{y:.2}" r="5"/><text class="distance-name" text-anchor="{anchor}" x="{text_x:.2}" y="{label_y:.2}">{name}</text><text class="distance-office" text-anchor="{anchor}" x="{text_x:.2}" y="{office_y:.2}">{distance:.2} {unit} · {office}</text>"#,
            label_y = label_y[index],
            office_y = label_y[index] + 15.0,
            name = escape_html(&point.name),
            distance = point.distance,
            office = escape_html(&point.office),
        );
    }
    let _ = write!(
        svg,
        r#"<circle class="distance-center" cx="{CX}" cy="{CY}" r="28"/><text class="distance-center-label" x="{CX}" y="{label_y}">{center}</text>"#,
        label_y = CY + 4.0,
        center = escape_html(center)
    );
    svg.push_str("</svg>");
    Ok(svg)
}

fn distance_label_positions(
    points: &[DistancePoint],
    cx: f64,
    cy: f64,
    min_y: f64,
    max_y: f64,
) -> Vec<f64> {
    let mut positions = vec![cy; points.len()];
    for right_side in [false, true] {
        let mut indices = points
            .iter()
            .enumerate()
            .filter(|(_, point)| {
                let radians = point.angle.to_radians();
                let x = cx + radians.cos() * point.distance;
                (x >= cx) == right_side
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        indices.sort_by(|left, right| {
            let left_y = cy + points[*left].angle.to_radians().sin() * points[*left].distance;
            let right_y = cy + points[*right].angle.to_radians().sin() * points[*right].distance;
            left_y
                .partial_cmp(&right_y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let last = indices.len().saturating_sub(1) as f64;
        for (slot, index) in indices.into_iter().enumerate() {
            positions[index] = if last == 0.0 {
                (min_y + max_y) / 2.0
            } else {
                min_y + (max_y - min_y) * slot as f64 / last
            };
        }
    }
    positions
}

fn network_svg(table: &CsvTable, center: &str) -> Result<String, String> {
    table.require_column("target")?;
    table.require_column("label")?;
    table.require_column("type")?;
    if table.rows.is_empty() {
        return Err("CSV contains no network nodes".to_string());
    }
    const WIDTH: f64 = 820.0;
    const HEIGHT: f64 = 430.0;
    const CX: f64 = 400.0;
    const CY: f64 = 215.0;
    let mut svg = svg_start(WIDTH, HEIGHT, center, "Qualitative destination network");
    for (index, row) in table.rows.iter().enumerate() {
        let label = table.value(row, "label").unwrap_or_default();
        let kind = table.value(row, "type").unwrap_or("other");
        let angle = index as f64 / table.rows.len() as f64 * std::f64::consts::TAU;
        let radius = 150.0;
        let x = CX + angle.cos() * radius;
        let y = CY + angle.sin() * radius;
        let _ = write!(
            svg,
            r#"<path class="network-edge" d="M {CX} {CY} Q {mid_x:.2} {mid_y:.2} {x:.2} {y:.2}"/><circle class="network-node network-node-{kind}" cx="{x:.2}" cy="{y:.2}" r="6"/><text class="network-label" text-anchor="{anchor}" x="{text_x:.2}" y="{text_y:.2}">{label}</text>"#,
            mid_x = (CX + x) / 2.0,
            mid_y = (CY + y) / 2.0 + if index % 2 == 0 { -18.0 } else { 18.0 },
            anchor = if x >= CX { "start" } else { "end" },
            text_x = x + if x >= CX { 13.0 } else { -13.0 },
            text_y = y + 4.0,
            label = escape_html(label),
            kind = css_token(kind),
        );
    }
    let _ = write!(
        svg,
        r#"<circle class="network-center" cx="{CX}" cy="{CY}" r="40"/><text class="network-center-label" x="{CX}" y="{label_y}">{center}</text>"#,
        label_y = CY + 5.0,
        center = escape_html(center)
    );
    svg.push_str("</svg>");
    Ok(svg)
}

fn css_token(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' {
                character
            } else {
                '-'
            }
        })
        .collect()
}
