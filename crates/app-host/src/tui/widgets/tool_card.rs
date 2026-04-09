use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::tui::theme::Theme;

pub struct ToolCard<'a> {
    pub name: String,
    pub args_summary: String,
    pub result: Option<String>,
    pub duration_secs: Option<f64>,
    pub expanded: bool,
    pub is_error: bool,
    theme: &'a Theme,
}

impl<'a> ToolCard<'a> {
    pub fn new(
        name: String,
        args_summary: String,
        result: Option<String>,
        duration_secs: Option<f64>,
        expanded: bool,
        is_error: bool,
        theme: &'a Theme,
    ) -> Self {
        Self {
            name,
            args_summary,
            result,
            duration_secs,
            expanded,
            is_error,
            theme,
        }
    }
}

impl<'a> Widget for ToolCard<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_color = if self.is_error {
            self.theme.tool_error_border
        } else {
            self.theme.tool_border
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(self.theme.tool_bg));

        let duration_str = self
            .duration_secs
            .map(|d| format!(" ({d:.1}s)"))
            .unwrap_or_default();

        let result_summary = self
            .result
            .as_ref()
            .map(|r| {
                let first_line = r.lines().next().unwrap_or("");
                if first_line.len() > 30 {
                    format!(" → {}...", &first_line[..27])
                } else {
                    format!(" → {first_line}")
                }
            })
            .unwrap_or_default();

        let sep = if self.args_summary.is_empty() {
            ""
        } else {
            " "
        };
        let summary = format!(
            "🔧 {}{}{}{}",
            self.name, sep, self.args_summary, result_summary
        );

        if self.expanded {
            let mut lines = vec![Line::from(Span::styled(
                summary,
                Style::default().fg(self.theme.tool_fg),
            ))];
            if !self.args_summary.is_empty() {
                lines.push(Line::from(Span::styled(
                    format!("  Args: {}", self.args_summary),
                    Style::default().fg(self.theme.foreground),
                )));
            }
            if let Some(ref result) = self.result {
                lines.push(Line::from(Span::styled(
                    format!("  Result: {result}"),
                    Style::default().fg(self.theme.foreground),
                )));
            }
            let paragraph = Paragraph::new(lines).block(block);
            paragraph.render(area, buf);
        } else {
            let line = Line::from(Span::styled(
                format!("{summary}{duration_str}"),
                Style::default().fg(self.theme.tool_fg),
            ));
            let paragraph = Paragraph::new(line).block(block);
            paragraph.render(area, buf);
        }
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
        let widget = crate::tui::widgets::tool_card::ToolCard::new(
            "read_file".to_string(),
            "main.rs".to_string(),
            Some("ok".to_string()),
            Some(0.1),
            false,
            false,
            &theme,
        );
        terminal
            .draw(|f| {
                f.render_widget(widget, f.area());
            })
            .unwrap();
    }
}
