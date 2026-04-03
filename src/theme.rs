use ratatui::style::Color;
use serde::Deserialize;
use std::collections::BTreeMap;

const THEMES_JSON: &str = include_str!("../data/themes.json");

/// Runtime theme colors used for rendering
#[derive(Clone)]
#[allow(dead_code)]
pub struct Theme {
    pub name: String,
    pub bg: Color,
    pub text: Color,
    pub text_dim: Color,
    pub sub: Color,
    pub main_color: Color,
    pub correct: Color,
    pub incorrect: Color,
    pub extra: Color,
    pub error_bg: Color,
    pub caret: Color,
}

/// Raw theme from JSON
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct RawTheme {
    bg: String,
    main: String,
    caret: String,
    sub: String,
    #[serde(default)]
    sub_alt: String,
    text: String,
    error: String,
    #[serde(default)]
    error_extra: String,
}

fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    // Expand 3-char hex (#abc -> #aabbcc)
    let expanded;
    let hex = if hex.len() == 3 {
        let chars: Vec<char> = hex.chars().collect();
        expanded = format!(
            "{}{}{}{}{}{}",
            chars[0], chars[0], chars[1], chars[1], chars[2], chars[2]
        );
        &expanded
    } else {
        hex
    };
    if hex.len() < 6 {
        return Color::White;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    Color::Rgb(r, g, b)
}

impl Theme {
    fn from_raw(name: &str, raw: &RawTheme) -> Self {
        let bg = hex_to_color(&raw.bg);
        let text = hex_to_color(&raw.text);
        let sub = hex_to_color(&raw.sub);
        let main_color = hex_to_color(&raw.main);
        let error = hex_to_color(&raw.error);
        let error_extra = if raw.error_extra.is_empty() {
            error
        } else {
            hex_to_color(&raw.error_extra)
        };

        Self {
            name: name.to_string(),
            bg,
            text,
            text_dim: sub,
            sub,
            main_color,
            correct: text,
            incorrect: error,
            extra: error_extra,
            error_bg: error_extra,
            caret: hex_to_color(&raw.caret),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        // Serika dark (monkeytype default) - hardcoded fallback
        Self {
            name: "serika_dark".to_string(),
            bg: Color::Rgb(50, 52, 55),
            text: Color::Rgb(210, 208, 200),
            text_dim: Color::Rgb(100, 102, 105),
            sub: Color::Rgb(100, 102, 105),
            main_color: Color::Rgb(226, 183, 20),
            correct: Color::Rgb(210, 208, 200),
            incorrect: Color::Rgb(202, 71, 71),
            extra: Color::Rgb(126, 50, 50),
            error_bg: Color::Rgb(126, 50, 50),
            caret: Color::Rgb(226, 183, 20),
        }
    }
}

/// The global theme catalog
pub struct ThemeCatalog {
    themes: BTreeMap<String, RawTheme>,
    names: Vec<String>,
}

impl ThemeCatalog {
    pub fn load() -> Self {
        let themes: BTreeMap<String, RawTheme> =
            serde_json::from_str(THEMES_JSON).unwrap_or_default();
        let names: Vec<String> = themes.keys().cloned().collect();
        Self { themes, names }
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn get(&self, name: &str) -> Theme {
        match self.themes.get(name) {
            Some(raw) => Theme::from_raw(name, raw),
            None => Theme::default(),
        }
    }

    pub fn count(&self) -> usize {
        self.themes.len()
    }

    /// Search theme names by query (case-insensitive substring match)
    pub fn search(&self, query: &str) -> Vec<&str> {
        let q = query.to_lowercase();
        self.names
            .iter()
            .filter(|n| n.to_lowercase().contains(&q))
            .map(|n| n.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_loads() {
        let catalog = ThemeCatalog::load();
        assert!(catalog.count() > 50, "Expected 50+ themes, got {}", catalog.count());
    }

    #[test]
    fn test_get_known_theme() {
        let catalog = ThemeCatalog::load();
        let theme = catalog.get("dracula");
        // Should not fall back to default
        assert_ne!(theme.bg, Color::Rgb(50, 52, 55), "Should have loaded dracula, not default");
    }

    #[test]
    fn test_search() {
        let catalog = ThemeCatalog::load();
        let results = catalog.search("dark");
        assert!(!results.is_empty());
        for r in &results {
            assert!(r.to_lowercase().contains("dark"));
        }
    }

    #[test]
    fn test_hex_to_color() {
        assert_eq!(hex_to_color("#ff0000"), Color::Rgb(255, 0, 0));
        assert_eq!(hex_to_color("#00ff00"), Color::Rgb(0, 255, 0));
        assert_eq!(hex_to_color("#333a45"), Color::Rgb(51, 58, 69));
        // 3-char hex codes
        assert_eq!(hex_to_color("#111"), Color::Rgb(17, 17, 17));
        assert_eq!(hex_to_color("#eee"), Color::Rgb(238, 238, 238));
        assert_eq!(hex_to_color("#f00"), Color::Rgb(255, 0, 0));
    }

    #[test]
    fn test_all_themes_parse_colors() {
        // Ensure no theme results in all-white due to bad hex parsing
        let catalog = ThemeCatalog::load();
        for name in catalog.names() {
            let theme = catalog.get(name);
            // bg should not be white (Color::White is the error fallback)
            assert_ne!(theme.bg, Color::White, "Theme '{}' has unparseable bg color", name);
        }
    }
}
