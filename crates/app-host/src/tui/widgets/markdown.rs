use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Paragraph, Widget};

use crate::tui::markdown::{styled_lines_to_ratatui, StyledLine};
use crate::tui::theme::Theme;

pub struct MarkdownWidget<'a> {
    lines: &'a [StyledLine],
    theme: &'a Theme,
}

impl<'a> MarkdownWidget<'a> {
    pub fn new(lines: &'a [StyledLine], theme: &'a Theme) -> Self {
        Self { lines, theme }
    }
}

impl<'a> Widget for MarkdownWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rat_lines = styled_lines_to_ratatui(self.lines);
        let paragraph = Paragraph::new(rat_lines).style(Style::default().bg(self.theme.background));
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
        let parsed = crate::tui::markdown::parse_markdown("hello **world**", &theme);
        let widget = crate::tui::widgets::markdown::MarkdownWidget::new(&parsed, &theme);
        terminal
            .draw(|f| {
                f.render_widget(widget, f.area());
            })
            .unwrap();
    }
}
