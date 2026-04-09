use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::tui::theme::Theme;

pub struct InputBar<'a> {
    text: &'a str,
    cursor_pos: usize,
    theme: &'a Theme,
}

impl<'a> InputBar<'a> {
    pub fn new(text: &'a str, cursor_pos: usize, theme: &'a Theme) -> Self {
        Self {
            text,
            cursor_pos,
            theme,
        }
    }
}

impl<'a> Widget for InputBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let pos = self.cursor_pos.min(self.text.len());
        let before = &self.text[..pos];
        let after = &self.text[pos..];

        let spans = vec![
            Span::styled("❯ ", Style::default().fg(self.theme.accent)),
            Span::styled(before, Style::default().fg(self.theme.foreground)),
            Span::styled("█", Style::default().fg(self.theme.foreground)),
            Span::styled(after, Style::default().fg(self.theme.foreground)),
        ];

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line).style(Style::default().bg(self.theme.background));
        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn renders_without_panic() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let backend = TestBackend::new(80, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = crate::tui::theme::Theme::github_dark();
        let text = "hello".to_string();
        let widget = crate::tui::widgets::input_bar::InputBar::new(&text, 2, &theme);
        terminal
            .draw(|f| {
                f.render_widget(widget, f.area());
            })
            .unwrap();
    }
}
