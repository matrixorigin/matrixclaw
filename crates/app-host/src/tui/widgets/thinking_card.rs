use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::tui::theme::Theme;

pub struct ThinkingCard<'a> {
    pub active: bool,
    pub duration_secs: Option<f64>,
    pub content: Option<String>,
    pub collapsed: bool,
    theme: &'a Theme,
}

impl<'a> ThinkingCard<'a> {
    pub fn new(
        active: bool,
        duration_secs: Option<f64>,
        content: Option<String>,
        collapsed: bool,
        theme: &'a Theme,
    ) -> Self {
        Self {
            active,
            duration_secs,
            content,
            collapsed,
            theme,
        }
    }
}

impl<'a> Widget for ThinkingCard<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = if self.active {
            "💭 Thinking...".to_string()
        } else if self.collapsed {
            match self.duration_secs {
                Some(d) => format!("💭 Thinking ({d:.1}s)"),
                None => "💭 Thinking".to_string(),
            }
        } else {
            "💭 Thinking...".to_string()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.thinking_border))
            .style(
                Style::default()
                    .bg(self.theme.thinking_bg)
                    .fg(self.theme.thinking_fg),
            );

        let paragraph = Paragraph::new(text).block(block);
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
        let widget =
            crate::tui::widgets::thinking_card::ThinkingCard::new(true, None, None, false, &theme);
        terminal
            .draw(|f| {
                f.render_widget(widget, f.area());
            })
            .unwrap();
    }
}
