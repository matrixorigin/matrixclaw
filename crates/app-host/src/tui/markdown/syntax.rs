use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    theme_name: String,
}

impl SyntaxHighlighter {
    pub fn new(theme_name: &str) -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            theme_name: theme_name.to_string(),
        }
    }

    pub fn highlight<'a>(
        &self,
        code: &'a str,
        language: Option<&str>,
    ) -> Vec<(syntect::highlighting::Style, &'a str)> {
        let syntax = match language {
            Some(lang) => self.syntax_set.find_syntax_by_token(lang),
            None => None,
        };
        let syntax = syntax
            .or_else(|| self.syntax_set.find_syntax_by_first_line(code))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = match self.theme_set.themes.get(&self.theme_name) {
            Some(t) => t,
            None => self
                .theme_set
                .themes
                .values()
                .next()
                .expect("at least one theme"),
        };
        let mut highlighter = syntect::easy::HighlightLines::new(syntax, theme);

        let mut result = Vec::new();
        for line in syntect::util::LinesWithEndings::from(code) {
            match highlighter.highlight_line(line, &self.syntax_set) {
                Ok(ranges) => result.extend(ranges),
                Err(_) => {
                    result.push((syntect::highlighting::Style::default(), line));
                }
            }
        }
        result
    }

    pub fn syntect_style_to_ratatui(style: syntect::highlighting::Style) -> ratatui::style::Style {
        let mut s = ratatui::style::Style::default().fg(syntect_color_to_ratatui(style.foreground));
        if style
            .font_style
            .contains(syntect::highlighting::FontStyle::BOLD)
        {
            s = s.add_modifier(ratatui::style::Modifier::BOLD);
        }
        if style
            .font_style
            .contains(syntect::highlighting::FontStyle::ITALIC)
        {
            s = s.add_modifier(ratatui::style::Modifier::ITALIC);
        }
        s
    }
}

pub fn syntect_color_to_ratatui(color: syntect::highlighting::Color) -> ratatui::style::Color {
    ratatui::style::Color::Rgb(color.r, color.g, color.b)
}
