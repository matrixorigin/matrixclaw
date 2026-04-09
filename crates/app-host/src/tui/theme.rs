use ratatui::style::Color;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct ThemeFile {
    color: ThemeColors,
}

#[derive(Debug, Deserialize)]
struct ThemeColors {
    background: String,
    foreground: String,
    border: String,
    accent: String,
    #[serde(default)]
    card: CardColors,
    #[serde(default)]
    syntax: SyntaxColors,
    #[serde(default)]
    markdown: MarkdownColors,
}

#[derive(Debug, Deserialize, Default)]
struct CardColors {
    #[serde(default)]
    thinking_bg: String,
    #[serde(default)]
    thinking_border: String,
    #[serde(default)]
    thinking_fg: String,
    #[serde(default)]
    tool_bg: String,
    #[serde(default)]
    tool_border: String,
    #[serde(default)]
    tool_fg: String,
    #[serde(default)]
    tool_error_border: String,
    #[serde(default)]
    error_fg: String,
    #[serde(default)]
    success_fg: String,
}

#[derive(Debug, Deserialize, Default)]
struct SyntaxColors {
    #[serde(default)]
    highlight_theme: String,
}

#[derive(Debug, Deserialize, Default)]
struct MarkdownColors {
    #[serde(default)]
    heading: String,
    #[serde(default)]
    bold: String,
    #[serde(default)]
    italic: String,
    #[serde(default)]
    code_bg: String,
    #[serde(default)]
    code_fg: String,
    #[serde(default)]
    link: String,
}

#[derive(Debug, Deserialize)]
pub struct ThemeConfig {
    color: ThemeColors,
}

#[derive(Debug, Deserialize)]
pub struct Keybindings {
    #[serde(default = "default_expand_detail")]
    pub expand_detail: String,
}

fn default_expand_detail() -> String {
    "d".to_string()
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            expand_detail: default_expand_detail(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TuiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub keybindings: Keybindings,
}

fn default_theme() -> String {
    "github-dark".to_string()
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            keybindings: Keybindings::default(),
        }
    }
}

pub fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 0x10).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 0x10).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 0x10).ok()?;
    Some(Color::Rgb(r, g, b))
}

#[derive(Debug)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub border: Color,
    pub accent: Color,
    pub thinking_bg: Color,
    pub thinking_border: Color,
    pub thinking_fg: Color,
    pub tool_bg: Color,
    pub tool_border: Color,
    pub tool_fg: Color,
    pub tool_error_border: Color,
    pub error_fg: Color,
    pub success_fg: Color,
    pub syntax_highlight_theme: String,
    pub heading: Color,
    pub bold: Color,
    pub italic: Color,
    pub code_bg: Color,
    pub code_fg: Color,
    pub link: Color,
}

impl Theme {
    pub fn from_config(config: &ThemeConfig) -> Self {
        let c = &config.color;
        Self {
            background: parse_hex_color(&c.background).unwrap_or(Color::Reset),
            foreground: parse_hex_color(&c.foreground).unwrap_or(Color::Reset),
            border: parse_hex_color(&c.border).unwrap_or(Color::Reset),
            accent: parse_hex_color(&c.accent).unwrap_or(Color::Reset),
            thinking_bg: parse_hex_color(&c.card.thinking_bg).unwrap_or(Color::Reset),
            thinking_border: parse_hex_color(&c.card.thinking_border).unwrap_or(Color::Reset),
            thinking_fg: parse_hex_color(&c.card.thinking_fg).unwrap_or(Color::Reset),
            tool_bg: parse_hex_color(&c.card.tool_bg).unwrap_or(Color::Reset),
            tool_border: parse_hex_color(&c.card.tool_border).unwrap_or(Color::Reset),
            tool_fg: parse_hex_color(&c.card.tool_fg).unwrap_or(Color::Reset),
            tool_error_border: parse_hex_color(&c.card.tool_error_border).unwrap_or(Color::Reset),
            error_fg: parse_hex_color(&c.card.error_fg).unwrap_or(Color::Reset),
            success_fg: parse_hex_color(&c.card.success_fg).unwrap_or(Color::Reset),
            syntax_highlight_theme: c.syntax.highlight_theme.clone(),
            heading: parse_hex_color(&c.markdown.heading).unwrap_or(Color::Reset),
            bold: parse_hex_color(&c.markdown.bold).unwrap_or(Color::Reset),
            italic: parse_hex_color(&c.markdown.italic).unwrap_or(Color::Reset),
            code_bg: parse_hex_color(&c.markdown.code_bg).unwrap_or(Color::Reset),
            code_fg: parse_hex_color(&c.markdown.code_fg).unwrap_or(Color::Reset),
            link: parse_hex_color(&c.markdown.link).unwrap_or(Color::Reset),
        }
    }

    pub fn github_dark() -> Self {
        let file: ThemeFile = toml::from_str(include_str!("themes/github_dark.toml"))
            .expect("github_dark.toml is valid");
        Self::from_config(&ThemeConfig { color: file.color })
    }

    pub fn tokyo_night() -> Self {
        let file: ThemeFile = toml::from_str(include_str!("themes/tokyo_night.toml"))
            .expect("tokyo_night.toml is valid");
        Self::from_config(&ThemeConfig { color: file.color })
    }

    pub fn light() -> Self {
        let file: ThemeFile =
            toml::from_str(include_str!("themes/light.toml")).expect("light.toml is valid");
        Self::from_config(&ThemeConfig { color: file.color })
    }
}

pub fn load_tui_config(home: &Path) -> (TuiConfig, Theme) {
    let config_path = home.join(".zstar").join("config").join("tui.toml");
    let config: TuiConfig = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    } else {
        TuiConfig::default()
    };
    let theme = load_theme(home, &config.theme);
    (config, theme)
}

pub fn load_theme(home: &Path, name: &str) -> Theme {
    match name {
        "github-dark" => Theme::github_dark(),
        "tokyo-night" => Theme::tokyo_night(),
        "light" => Theme::light(),
        _ => {
            let theme_path = home
                .join(".zstar")
                .join("themes")
                .join(format!("{name}.toml"));
            if theme_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&theme_path) {
                    if let Ok(file) = toml::from_str::<ThemeFile>(&content) {
                        return Theme::from_config(&ThemeConfig { color: file.color });
                    }
                }
            }
            Theme::github_dark()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn parse_hex_color_valid() {
        assert_eq!(parse_hex_color("#58a6ff"), Some(Color::Rgb(88, 166, 255)));
    }

    #[test]
    fn parse_hex_color_black() {
        assert_eq!(parse_hex_color("#000000"), Some(Color::Rgb(0, 0, 0)));
    }

    #[test]
    fn parse_hex_color_invalid() {
        assert_eq!(parse_hex_color("not-a-color"), None);
    }

    #[test]
    fn load_github_dark_theme() {
        let theme = Theme::github_dark();
        assert_eq!(theme.background, Color::Rgb(13, 17, 23));
        assert_eq!(theme.tool_border, Color::Rgb(35, 134, 54));
    }

    #[test]
    fn load_tokyo_night_theme() {
        let theme = Theme::tokyo_night();
        assert_eq!(theme.background, Color::Rgb(26, 27, 38));
    }

    #[test]
    fn load_light_theme() {
        let theme = Theme::light();
        assert_eq!(theme.background, Color::Rgb(250, 250, 250));
    }

    #[test]
    fn default_tui_config() {
        let config = TuiConfig::default();
        assert_eq!(config.theme, "github-dark");
        assert_eq!(config.keybindings.expand_detail, "d");
    }

    #[test]
    fn fallback_to_github_dark_for_unknown() {
        let home = tempfile::tempdir().expect("tempdir");
        let theme = load_theme(home.path(), "nonexistent-theme");
        assert_eq!(theme.background, Color::Rgb(13, 17, 23));
    }
}
