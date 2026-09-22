//! Markdown document structure and the small, deliberately closed directive
//! language used by special articles such as the ISCAS Guide home page.
//!
//! Directives are parsed before Markdown rendering so the renderer can mount
//! real Leptos components between normal Markdown fragments.  This keeps the
//! Markdown/HTML boundary local to a fragment and, importantly, does not need
//! a second DOM pass after the page has mounted.

use std::collections::HashMap;

use crate::utils::{MarkdownArticle, MarkdownHeading};

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentBlock {
    Markdown(String),
    Directive(DirectiveSpec),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownDocument {
    pub blocks: Vec<DocumentBlock>,
    pub headings: Vec<MarkdownHeading>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartKind {
    Line,
    Bar,
    Pie,
    Donut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueFormat {
    Integer,
    Decimal1,
    Raw,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeroSpec {
    pub kicker: String,
    pub title: String,
    pub subtitle: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatsSpec {
    pub columns: usize,
    pub kicker: Option<String>,
    pub title: Option<String>,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FactsSpec {
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartSeriesSpec {
    pub column: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartSpec {
    pub kind: ChartKind,
    pub orientation: ChartOrientation,
    pub grid: Option<String>,
    pub source: String,
    pub x: Option<String>,
    pub series: Vec<ChartSeriesSpec>,
    pub name: Option<String>,
    pub value: Option<String>,
    pub unit: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub center: Option<String>,
    pub value_format: ValueFormat,
    pub show_values: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DistanceFieldSpec {
    pub source: String,
    pub center: String,
    pub max_distance: f64,
    pub unit: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabsSpec {
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FeaturesSpec {
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArticleIndexSpec {
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub limit: usize,
    pub show_description: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContributorsSpec {
    pub source: String,
    pub title: Option<String>,
    pub avatar_size: u16,
    pub label_size: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkSpec {
    pub source: String,
    pub center: String,
    pub mode: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepsSpec {
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinkListSpec {
    pub variant: String,
    pub title: Option<String>,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PledgeSpec {
    pub title: Option<String>,
    pub label: String,
    pub storage_key: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisclaimerSpec {
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectiveSpec {
    Hero(HeroSpec),
    Stats(StatsSpec),
    Facts(FactsSpec),
    Chart(ChartSpec),
    DistanceField(DistanceFieldSpec),
    Labs(LabsSpec),
    Features(FeaturesSpec),
    ArticleIndex(ArticleIndexSpec),
    Contributors(ContributorsSpec),
    Network(NetworkSpec),
    Steps(StepsSpec),
    LinkList(LinkListSpec),
    RelatedSites(LinkListSpec),
    Pledge(PledgeSpec),
    Disclaimer(DisclaimerSpec),
    Unknown { name: String, body: String },
    Invalid { name: String, message: String },
}

impl MarkdownDocument {
    pub fn parse(content: &str) -> Self {
        let blocks = split_blocks(content);
        let mut headings = Vec::new();
        let mut counts = HashMap::<String, usize>::new();

        for block in &blocks {
            let DocumentBlock::Markdown(markdown) = block else {
                continue;
            };

            for heading in MarkdownArticle::new(markdown.clone(), String::new()).headings() {
                let base_id = slugify(&heading.title);
                let count = counts.entry(base_id.clone()).or_insert(0);
                let id = if *count == 0 {
                    base_id
                } else {
                    format!("{base_id}-{}", *count + 1)
                };
                *count += 1;
                headings.push(MarkdownHeading {
                    id,
                    title: heading.title,
                    level: heading.level,
                });
            }
        }

        Self { blocks, headings }
    }
}

fn split_blocks(content: &str) -> Vec<DocumentBlock> {
    let lines = content.lines().collect::<Vec<_>>();
    let mut blocks = Vec::new();
    let mut markdown = Vec::new();
    let mut index = 0;

    let flush_markdown = |blocks: &mut Vec<DocumentBlock>, markdown: &mut Vec<&str>| {
        let text = markdown.join("\n");
        if !text.trim().is_empty() {
            blocks.push(DocumentBlock::Markdown(text));
        }
        markdown.clear();
    };

    while index < lines.len() {
        let Some(name) = directive_name(lines[index]) else {
            markdown.push(lines[index]);
            index += 1;
            continue;
        };

        flush_markdown(&mut blocks, &mut markdown);
        index += 1;
        let mut option_lines = Vec::new();
        let mut body_lines = Vec::new();
        let mut in_body = false;
        let mut closed = false;

        while index < lines.len() {
            let line = lines[index];
            if line.trim() == ":::" {
                closed = true;
                index += 1;
                break;
            }

            if !in_body && line.trim().is_empty() {
                in_body = true;
            } else if !in_body && is_option_line(line) {
                option_lines.push(line);
            } else {
                in_body = true;
                body_lines.push(line);
            }
            index += 1;
        }

        if !closed {
            blocks.push(DocumentBlock::Directive(DirectiveSpec::Invalid {
                name,
                message: "directive is missing its closing ::: line".to_string(),
            }));
            break;
        }

        let body = body_lines.join("\n");
        let spec = parse_directive(&name, &option_lines, body);
        blocks.push(DocumentBlock::Directive(spec));
    }

    flush_markdown(&mut blocks, &mut markdown);
    blocks
}

fn is_option_line(line: &str) -> bool {
    let Some((key, _)) = line.split_once(':') else {
        return false;
    };
    let key = key.trim();
    !key.is_empty()
        && key
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
}

fn directive_name(line: &str) -> Option<String> {
    let line = line.trim();
    if !line.starts_with(":::") || line == ":::" {
        return None;
    }

    let name = line[3..].trim();
    if name.is_empty()
        || !name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return None;
    }
    Some(name.to_ascii_lowercase())
}

fn options(lines: &[&str]) -> HashMap<String, String> {
    lines
        .iter()
        .filter_map(|line| {
            let (key, value) = line.split_once(':')?;
            let key = key.trim().to_ascii_lowercase();
            if key.is_empty() {
                None
            } else {
                Some((key, value.trim().to_string()))
            }
        })
        .collect()
}

fn option<'a>(values: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    values.get(key).map(String::as_str)
}

fn required_option(values: &HashMap<String, String>, key: &str) -> Result<String, String> {
    option(values, key)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("missing required option '{key}'"))
}

fn parse_directive(name: &str, option_lines: &[&str], body: String) -> DirectiveSpec {
    let values = options(option_lines);
    let invalid = |message: String| DirectiveSpec::Invalid {
        name: name.to_string(),
        message,
    };

    if body.lines().any(|line| {
        let trimmed = line.trim();
        (trimmed.starts_with(":::") && trimmed != ":::") || trimmed == ":::"
    }) {
        return invalid("nested directives are not supported inside a directive body".to_string());
    }

    match name {
        "hero" => DirectiveSpec::Hero(HeroSpec {
            kicker: option(&values, "kicker").unwrap_or_default().to_string(),
            title: option(&values, "title").unwrap_or_default().to_string(),
            subtitle: option(&values, "subtitle").unwrap_or_default().to_string(),
            body,
        }),
        "stats" => DirectiveSpec::Stats(StatsSpec {
            columns: option(&values, "columns")
                .and_then(|value| value.parse().ok())
                .filter(|columns| *columns > 0)
                .unwrap_or(6),
            kicker: option(&values, "kicker").map(ToOwned::to_owned),
            title: option(&values, "title").map(ToOwned::to_owned),
            body,
        }),
        "facts" => DirectiveSpec::Facts(FactsSpec { body }),
        "labs" => DirectiveSpec::Labs(LabsSpec { body }),
        "features" => DirectiveSpec::Features(FeaturesSpec { body }),
        "steps" => DirectiveSpec::Steps(StepsSpec { body }),
        "disclaimer" => DirectiveSpec::Disclaimer(DisclaimerSpec { body }),
        "chart" => parse_chart(&values)
            .map(DirectiveSpec::Chart)
            .unwrap_or_else(invalid),
        "distance-field" => parse_distance_field(&values)
            .map(DirectiveSpec::DistanceField)
            .unwrap_or_else(invalid),
        "article-index" => DirectiveSpec::ArticleIndex(ArticleIndexSpec {
            category: option(&values, "category").map(ToOwned::to_owned),
            tags: option(&values, "tags")
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .filter(|tag| !tag.is_empty())
                .map(ToOwned::to_owned)
                .collect(),
            limit: option(&values, "limit")
                .and_then(|value| value.parse().ok())
                .filter(|limit| *limit > 0)
                .unwrap_or(16),
            show_description: parse_bool(option(&values, "description"), true),
        }),
        "contributors" => required_option(&values, "src")
            .map(|source| {
                DirectiveSpec::Contributors(ContributorsSpec {
                    source,
                    title: option(&values, "title").map(ToOwned::to_owned),
                    avatar_size: option(&values, "avatar-size")
                        .and_then(|value| value.parse().ok())
                        .filter(|size| *size > 0)
                        .unwrap_or(70),
                    label_size: option(&values, "label-size")
                        .filter(|size| !size.trim().is_empty())
                        .unwrap_or("0.64rem")
                        .to_string(),
                    body,
                })
            })
            .unwrap_or_else(invalid),
        "network" => required_option(&values, "src")
            .map(|source| {
                DirectiveSpec::Network(NetworkSpec {
                    source,
                    center: option(&values, "center").unwrap_or("ISCAS").to_string(),
                    mode: option(&values, "mode").unwrap_or("qualitative").to_string(),
                })
            })
            .unwrap_or_else(invalid),
        "link-list" => DirectiveSpec::LinkList(LinkListSpec {
            variant: option(&values, "variant").unwrap_or("default").to_string(),
            title: option(&values, "title").map(ToOwned::to_owned),
            body,
        }),
        "related-sites" => DirectiveSpec::RelatedSites(LinkListSpec {
            variant: "related-sites".to_string(),
            title: option(&values, "title").map(ToOwned::to_owned),
            body,
        }),
        "pledge" => DirectiveSpec::Pledge(PledgeSpec {
            title: option(&values, "title").map(ToOwned::to_owned),
            label: option(&values, "label").unwrap_or("对软所效忠").to_string(),
            storage_key: option(&values, "storage-key")
                .unwrap_or("iscas-pledged")
                .to_string(),
            body,
        }),
        _ => DirectiveSpec::Unknown {
            name: name.to_string(),
            body,
        },
    }
}

fn parse_chart(values: &HashMap<String, String>) -> Result<ChartSpec, String> {
    let kind = match option(values, "type")
        .unwrap_or("bar")
        .to_ascii_lowercase()
        .as_str()
    {
        "line" => ChartKind::Line,
        "bar" => ChartKind::Bar,
        "pie" => ChartKind::Pie,
        "donut" => ChartKind::Donut,
        value => return Err(format!("unsupported chart type '{value}'")),
    };
    let source = required_option(values, "src")?;
    let orientation = match option(values, "orientation")
        .unwrap_or("vertical")
        .to_ascii_lowercase()
        .as_str()
    {
        "horizontal" => ChartOrientation::Horizontal,
        "vertical" => ChartOrientation::Vertical,
        value => return Err(format!("unsupported chart orientation '{value}'")),
    };
    let series = option(values, "series")
        .unwrap_or_default()
        .split(',')
        .filter_map(|entry| {
            let (column, label) = entry.split_once('=')?;
            Some(ChartSeriesSpec {
                column: column.trim().to_string(),
                label: label.trim().to_string(),
            })
        })
        .filter(|series| !series.column.is_empty())
        .collect::<Vec<_>>();
    if matches!(kind, ChartKind::Line | ChartKind::Bar) && series.is_empty() {
        return Err("line and bar charts need at least one series: column=label".to_string());
    }
    if matches!(kind, ChartKind::Line | ChartKind::Bar) && option(values, "x").is_none() {
        return Err("line and bar charts need an x column".to_string());
    }
    if matches!(kind, ChartKind::Pie | ChartKind::Donut)
        && (option(values, "name").is_none() || option(values, "value").is_none())
    {
        return Err("pie and donut charts need name and value columns".to_string());
    }

    let parse_number = |key: &str| -> Result<Option<f64>, String> {
        option(values, key)
            .map(|value| {
                value
                    .parse::<f64>()
                    .map(Some)
                    .map_err(|_| format!("option '{key}' must be a number"))
            })
            .unwrap_or(Ok(None))
    };
    let value_format = match option(values, "value-format")
        .unwrap_or("raw")
        .to_ascii_lowercase()
        .as_str()
    {
        "integer" => ValueFormat::Integer,
        "decimal-1" => ValueFormat::Decimal1,
        "raw" => ValueFormat::Raw,
        value => return Err(format!("unsupported value format '{value}'")),
    };

    Ok(ChartSpec {
        kind,
        orientation,
        grid: option(values, "grid")
            .filter(|value| !value.trim().is_empty())
            .map(ToOwned::to_owned),
        source,
        x: option(values, "x").map(ToOwned::to_owned),
        series,
        name: option(values, "name").map(ToOwned::to_owned),
        value: option(values, "value").map(ToOwned::to_owned),
        unit: option(values, "unit").map(ToOwned::to_owned),
        min: parse_number("min")?,
        max: parse_number("max")?,
        title: option(values, "title").map(ToOwned::to_owned),
        subtitle: option(values, "subtitle").map(ToOwned::to_owned),
        center: option(values, "center").map(ToOwned::to_owned),
        value_format,
        show_values: parse_bool(option(values, "show-values"), true),
    })
}

fn parse_distance_field(values: &HashMap<String, String>) -> Result<DistanceFieldSpec, String> {
    let source = required_option(values, "src")?;
    let max_distance = option(values, "max-distance")
        .unwrap_or("2.5")
        .parse::<f64>()
        .map_err(|_| "option 'max-distance' must be a number".to_string())?;
    if max_distance <= 0.0 {
        return Err("option 'max-distance' must be positive".to_string());
    }
    Ok(DistanceFieldSpec {
        source,
        center: option(values, "center").unwrap_or("ISCAS").to_string(),
        max_distance,
        unit: option(values, "unit").unwrap_or("km").to_string(),
        title: option(values, "title")
            .unwrap_or("Zhongguancun / Tech field")
            .to_string(),
    })
}

fn parse_bool(value: Option<&str>, default: bool) -> bool {
    match value.map(|value| value.to_ascii_lowercase()).as_deref() {
        Some("true" | "yes" | "1") => true,
        Some("false" | "no" | "0") => false,
        _ => default,
    }
}

pub fn parse_markdown_table(body: &str) -> Result<MarkdownTable, String> {
    let mut rows = body
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with('|') && line.ends_with('|'))
        .map(|line| {
            line.trim_matches('|')
                .split('|')
                .map(|cell| cell.trim().to_string())
                .collect::<Vec<_>>()
        });
    let headers = rows
        .next()
        .ok_or_else(|| "expected a Markdown table".to_string())?;
    let separator = rows
        .next()
        .ok_or_else(|| "expected a Markdown table separator".to_string())?;
    if separator.is_empty()
        || separator.iter().any(|cell| {
            !cell
                .chars()
                .all(|character| character == '-' || character == ':' || character.is_whitespace())
        })
    {
        return Err("expected a Markdown table separator row".to_string());
    }
    let rows = rows
        .map(|row| {
            let mut row = row;
            row.resize(headers.len(), String::new());
            row.truncate(headers.len());
            row
        })
        .collect();
    Ok(MarkdownTable { headers, rows })
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownTable {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub fn parse_markdown_list(body: &str) -> Vec<String> {
    body.lines()
        .filter_map(|line| {
            let line = line.trim();
            let line = line
                .strip_prefix("- ")
                .or_else(|| line.strip_prefix("* "))
                .or_else(|| line.strip_prefix("+ "))?;
            let line = line.trim();
            (!line.is_empty()).then(|| line.to_string())
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_markdown_directive_markdown_blocks() {
        let document = MarkdownDocument::parse(
            "# Intro\n\nText\n\n:::stats\ncolumns: 2\n\n| value | label |\n| --- | --- |\n| 1 | \
             One |\n:::\n\n## Outro",
        );
        assert_eq!(document.blocks.len(), 3);
        assert_eq!(document.headings[0].id, "intro");
        assert_eq!(document.headings[1].id, "outro");
        assert!(matches!(
            document.blocks[1],
            DocumentBlock::Directive(DirectiveSpec::Stats(_))
        ));
    }

    #[test]
    fn duplicate_heading_ids_are_document_scoped() {
        let document =
            MarkdownDocument::parse("## 数据\n\n:::labs\n- one\n:::\n\n## 数据\n\n## 数据");
        let ids = document
            .headings
            .iter()
            .map(|heading| heading.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["数据", "数据-2", "数据-3"]);
    }

    #[test]
    fn malformed_and_unknown_directives_are_non_fatal() {
        let document = MarkdownDocument::parse(":::missing\nkey: value\n");
        assert!(matches!(
            document.blocks.first(),
            Some(DocumentBlock::Directive(DirectiveSpec::Invalid { .. }))
        ));

        let unknown = MarkdownDocument::parse(":::future\n\nhello\n:::");
        assert!(matches!(
            unknown.blocks.first(),
            Some(DocumentBlock::Directive(DirectiveSpec::Unknown { .. }))
        ));
    }

    #[test]
    fn chart_options_are_typed_and_validated() {
        let document = MarkdownDocument::parse(
            ":::chart\ntype: line\nsrc: /$ASSETS/data/trend.csv\nx: year\nseries: \
             count=Count\nvalue-format: integer\nshow-values: false\n:::",
        );
        let Some(DocumentBlock::Directive(DirectiveSpec::Chart(chart))) = document.blocks.first()
        else {
            panic!("expected chart directive");
        };
        assert_eq!(chart.kind, ChartKind::Line);
        assert_eq!(chart.series[0].column, "count");
        assert_eq!(chart.value_format, ValueFormat::Integer);
        assert!(!chart.show_values);
    }

    #[test]
    fn chart_grid_option_is_preserved_for_adjacent_layouts() {
        let document = MarkdownDocument::parse(
            ":::chart\ntype: line\ngrid: recommendation\nsrc: /$ASSETS/data/trend.csv\nx: \
             year\nseries: count=Count\n:::",
        );
        let Some(DocumentBlock::Directive(DirectiveSpec::Chart(chart))) = document.blocks.first()
        else {
            panic!("expected chart directive");
        };
        assert_eq!(chart.grid.as_deref(), Some("recommendation"));
    }

    #[test]
    fn tables_and_lists_have_small_typed_helpers() {
        let table =
            parse_markdown_table("| value | label |\n| :--- | ---: |\n| 269 | 学硕 | ").unwrap();
        assert_eq!(table.rows[0][1], "学硕");
        assert_eq!(
            parse_markdown_list("- one\n* two\nnot a list"),
            vec!["one", "two"]
        );
    }

    #[test]
    fn nested_directive_bodies_are_rejected() {
        let document = MarkdownDocument::parse(":::hero\n\n:::chart\n:::\n:::");
        assert!(matches!(
            document.blocks.first(),
            Some(DocumentBlock::Directive(DirectiveSpec::Invalid { .. }))
        ));
    }

    #[test]
    fn directive_bodies_can_start_without_a_blank_line() {
        let document = MarkdownDocument::parse(":::labs\n- Systems\n- Security\n:::");
        assert!(matches!(
            document.blocks.first(),
            Some(DocumentBlock::Directive(DirectiveSpec::Labs(LabsSpec { body })))
                if body.contains("Systems") && body.contains("Security")
        ));
    }

    #[test]
    fn home_article_uses_only_valid_directives() {
        let document =
            MarkdownDocument::parse(include_str!("../assets/_assets/articles/home/index.md"));
        assert!(!document.blocks.iter().any(|block| matches!(
            block,
            DocumentBlock::Directive(DirectiveSpec::Invalid { .. })
        )));
        assert!(document.blocks.iter().any(|block| {
            matches!(
                block,
                DocumentBlock::Directive(DirectiveSpec::RelatedSites(spec))
                    if !spec.body.trim().is_empty()
            )
        }));
    }
}
