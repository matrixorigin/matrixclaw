use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::tui::theme::Theme;

pub struct PromptBlock<'a> {
    text: String,
    theme: &'a Theme,
}

impl<'a> PromptBlock<'a> {
    pub fn new(text: String, theme: &'a Theme) -> Self {
        Self { text, theme }
    }
}

impl<'a> Widget for PromptBlock<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let prompt_line = Line::from(vec![
            Span::styled(
                "❯ ",
                Style::default()
                    .fg(self.theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                self.text,
                Style::default()
                    .fg(self.theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let separator = Line::from(Span::styled(
            "─".repeat(area.width as usize),
            Style::default().fg(self.theme.border),
        ));

        let paragraph = Paragraph::new(vec![prompt_line, separator])
            .style(Style::default().bg(self.theme.background));
        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn renders_without_panic() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = crate::tui::theme::Theme::github_dark();
        let widget = crate::tui::widgets::prompt::PromptBlock::new("hello".to_string(), &theme);
        terminal
            .draw(|f| {
                f.render_widget(widget, f.area());
            })
            .unwrap();
    }
}
