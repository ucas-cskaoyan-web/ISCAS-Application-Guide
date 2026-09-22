use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ThemeOptions {
    pub background: String,
    #[serde(default, alias = "focus")]
    pub background_focus: Option<BackgroundFocusOptions>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub struct BackgroundFocusRectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum BackgroundFocusOverflow {
    Center,
    Left,
    Right,
    #[default]
    None,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub struct BackgroundFocusOptions {
    #[serde(alias = "rect")]
    pub rectangle: BackgroundFocusRectangle,
    #[serde(default, alias = "alignment")]
    pub overflow: BackgroundFocusOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackgroundFocusPosition {
    pub x_percent: f64,
    pub y_percent: f64,
}

impl Default for BackgroundFocusPosition {
    fn default() -> Self {
        Self {
            x_percent: 50.0,
            y_percent: 50.0,
        }
    }
}

impl BackgroundFocusOptions {
    pub fn is_enabled(&self) -> bool {
        !matches!(self.overflow, BackgroundFocusOverflow::None)
    }

    /// Calculate the CSS `background-position` percentages for a cover-sized
    /// image.
    ///
    /// Coordinates in the configuration are source-image pixels. The focus is
    /// centered vertically, while the overflow mode controls the horizontal
    /// relation between the focus rectangle and the viewport.
    pub fn position(
        &self,
        image_width: f64,
        image_height: f64,
        viewport_width: f64,
        viewport_height: f64,
    ) -> BackgroundFocusPosition {
        const EPSILON: f64 = 0.0001;
        let default_position = BackgroundFocusPosition::default();

        if !self.is_enabled()
            || !image_width.is_finite()
            || !image_height.is_finite()
            || !viewport_width.is_finite()
            || !viewport_height.is_finite()
            || image_width <= EPSILON
            || image_height <= EPSILON
            || viewport_width <= EPSILON
            || viewport_height <= EPSILON
        {
            return default_position;
        }

        let scale = (viewport_width / image_width).max(viewport_height / image_height);
        if !scale.is_finite() || scale <= EPSILON {
            return default_position;
        }

        let visible_width = (viewport_width / scale).min(image_width);
        let visible_height = (viewport_height / scale).min(image_height);
        let horizontal_space = (image_width - visible_width).max(0.0);
        let vertical_space = (image_height - visible_height).max(0.0);

        // With no cover crop, the whole image is already visible and focus
        // would only introduce an unnecessary position change.
        if horizontal_space <= EPSILON && vertical_space <= EPSILON {
            return default_position;
        }

        let rectangle = self.rectangle;
        if rectangle.width <= 0 || rectangle.height <= 0 {
            return default_position;
        }

        let rectangle_x = f64::from(rectangle.x);
        let rectangle_y = f64::from(rectangle.y);
        let rectangle_width = f64::from(rectangle.width);
        let rectangle_height = f64::from(rectangle.height);
        let left = rectangle_x.clamp(0.0, image_width);
        let top = rectangle_y.clamp(0.0, image_height);
        let right = (rectangle_x + rectangle_width).clamp(left, image_width);
        let bottom = (rectangle_y + rectangle_height).clamp(top, image_height);
        if right - left <= EPSILON || bottom - top <= EPSILON {
            return default_position;
        }

        let target_left = match self.overflow {
            BackgroundFocusOverflow::Left => left,
            BackgroundFocusOverflow::Right => right - visible_width,
            BackgroundFocusOverflow::Center | BackgroundFocusOverflow::None => {
                (left + right) / 2.0 - visible_width / 2.0
            }
        };
        let source_left = target_left.clamp(0.0, horizontal_space);
        let source_top = ((top + bottom) / 2.0 - visible_height / 2.0).clamp(0.0, vertical_space);

        BackgroundFocusPosition {
            x_percent: Self::percentage(source_left, horizontal_space),
            y_percent: Self::percentage(source_top, vertical_space),
        }
    }

    fn percentage(value: f64, range: f64) -> f64 {
        if range <= 0.0001 {
            50.0
        } else {
            (value / range * 100.0).clamp(0.0, 100.0)
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AssetsOptions {
    pub directory: String,
    pub articles: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AuthorOptions {
    pub name: String,
    pub email: String,
    pub github: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CopyrightOptions {
    pub holder: String,
    pub notice: String,
    pub license: String,
    pub license_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct SiteLinks {
    #[serde(default = "default_legacy_site_url")]
    pub legacy_site_url: String,
}

fn default_legacy_site_url() -> String {
    "https://iscas-application-guide.pages.dev/".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HomeOptions {
    #[serde(default = "default_home_article_id")]
    pub article_id: String,
    #[serde(default)]
    pub welcome_title: String,
    #[serde(default)]
    pub welcome_text: Vec<String>,
}

fn default_home_article_id() -> String {
    "home".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ArticlesOptions {
    pub maximum_number_per_page: usize,
    pub pagination_size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Site {
    pub name: String,
    pub copyright_year: u16,
    pub assets: AssetsOptions,
    #[serde(default)]
    pub theme: Option<ThemeOptions>,
    pub author: AuthorOptions,
    pub copyright: CopyrightOptions,
    #[serde(default)]
    pub links: SiteLinks,
    pub home: HomeOptions,
    pub articles: ArticlesOptions,
}

impl Site {
    // Fetch site configuration from a JSON file
    pub async fn fetch() -> Result<Self, String> {
        let response = Request::get(&public_url("site.json"))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        let text = response
            .text()
            .await
            .map_err(|e| format!("Failed to get text: {}", e))?;
        let site =
            serde_json_wasm::from_str(&text).map_err(|e| format!("Failed to parse JSON: {}", e))?;
        Ok(site)
    }

    pub fn long(&self) -> String {
        self.name.clone()
    }

    pub fn short(&self) -> String {
        let mut result = String::new();
        for p in self.name.split_whitespace() {
            if let Some(c) = p.chars().next() {
                result.push(c.to_ascii_uppercase())
            }
        }

        result
    }

    pub fn background_url(&self) -> Option<String> {
        self.theme
            .as_ref()
            .map(|theme| public_url(&theme.background))
    }

    pub fn article_asset_url(&self, id: &str, path: &str) -> String {
        self.resolve_article_asset(id, path)
    }

    /// Resolve the `$ASSETS` placeholder used by Markdown and directives.
    /// Keeping this in the typed site contract prevents each renderer from
    /// inventing a different deployment/sub-path URL rule.
    pub fn resolve_article_asset(&self, id: &str, path: &str) -> String {
        let relative_path = path
            .split_once("$ASSETS/")
            .map(|(_, suffix)| suffix)
            .unwrap_or(path)
            .trim_start_matches('/');
        public_url(&format!(
            "{}/{}/{}/{}",
            self.assets.directory, self.assets.articles, id, relative_path
        ))
    }
}

/// Resolve an application asset against Trunk's public URL base.
///
/// A relative URL keeps local Trunk serving and Cloudflare sub-path
/// deployments working without baking a deployment-specific prefix into JSON.
pub fn public_url(path: &str) -> String {
    let path = path.trim_start_matches('/');
    let fallback = format!("/{path}");

    let Some(window) = web_sys::window() else {
        return fallback;
    };
    let Some(document) = window.document() else {
        return fallback;
    };
    let Ok(Some(base)) = document.base_uri() else {
        return fallback;
    };

    web_sys::Url::new_with_base(path, &base)
        .map(|url| url.href())
        .unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::{BackgroundFocusOptions, BackgroundFocusOverflow, BackgroundFocusRectangle, Site};

    fn focus(overflow: BackgroundFocusOverflow) -> BackgroundFocusOptions {
        BackgroundFocusOptions {
            rectangle: BackgroundFocusRectangle {
                x: 1200,
                y: 100,
                width: 400,
                height: 200,
            },
            overflow,
        }
    }

    #[test]
    fn focus_is_disabled_when_the_full_image_is_visible() {
        let position =
            focus(BackgroundFocusOverflow::Right).position(2000.0, 1000.0, 2000.0, 1000.0);
        assert_eq!(position.x_percent, 50.0);
        assert_eq!(position.y_percent, 50.0);
    }

    #[test]
    fn focus_aligns_the_rectangle_on_the_requested_horizontal_edge() {
        let center =
            focus(BackgroundFocusOverflow::Center).position(2000.0, 1000.0, 1000.0, 1000.0);
        let left = focus(BackgroundFocusOverflow::Left).position(2000.0, 1000.0, 1000.0, 1000.0);
        let right = focus(BackgroundFocusOverflow::Right).position(2000.0, 1000.0, 1000.0, 1000.0);

        assert_eq!(center.x_percent, 90.0);
        assert_eq!(left.x_percent, 100.0);
        assert_eq!(right.x_percent, 60.0);
        assert_eq!(center.y_percent, 50.0);
    }

    #[test]
    fn none_and_invalid_rectangles_fall_back_to_center() {
        let disabled =
            focus(BackgroundFocusOverflow::None).position(2000.0, 1000.0, 1000.0, 1000.0);
        assert_eq!(disabled.x_percent, 50.0);

        let invalid = BackgroundFocusOptions {
            rectangle: BackgroundFocusRectangle {
                x: 0,
                y: 0,
                width: -1,
                height: 100,
            },
            overflow: BackgroundFocusOverflow::Center,
        }
        .position(2000.0, 1000.0, 1000.0, 1000.0);
        assert_eq!(invalid.x_percent, 50.0);
        assert_eq!(invalid.y_percent, 50.0);
    }

    #[test]
    fn site_config_parses_background_focus() {
        let site: Site = serde_json_wasm::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/site.json"
        )))
        .expect("site.json should parse with serde-json-wasm");

        let focus = site
            .theme
            .expect("site.json should include theme")
            .background_focus
            .expect("site.json should include background focus");
        assert_eq!(focus.rectangle.x, 460);
        assert_eq!(focus.overflow, BackgroundFocusOverflow::Center);
        assert_eq!(site.copyright.holder, "历届考生及热心网友");
        assert_eq!(site.copyright.license, "CC BY-NC-SA 4.0");
    }
}
