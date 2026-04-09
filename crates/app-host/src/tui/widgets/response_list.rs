use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use crate::tui::theme::Theme;
use crate::tui::widgets::markdown::MarkdownWidget;
use crate::tui::widgets::prompt::PromptBlock;
use crate::tui::widgets::thinking_card::ThinkingCard;
use crate::tui::widgets::tool_card::ToolCard;

pub enum ResponseBlock {
    UserPrompt {
        text: String,
    },
    ThinkingCard {
        duration_secs: Option<f64>,
        content: Option<String>,
        collapsed: bool,
    },
    ToolCard {
        name: String,
        args_summary: String,
        result: Option<String>,
        duration_secs: Option<f64>,
        expanded: bool,
        is_error: bool,
    },
    MarkdownContent {
        parsed: Vec<crate::tui::markdown::StyledLine>,
    },
}

pub struct ResponseList<'a> {
    blocks: &'a [ResponseBlock],
    theme: &'a Theme,
    scroll_offset: u16,
}

impl<'a> ResponseList<'a> {
    pub fn new(blocks: &'a [ResponseBlock], theme: &'a Theme, scroll_offset: u16) -> Self {
        Self {
            blocks,
            theme,
            scroll_offset,
        }
    }
}

fn block_height(block: &ResponseBlock) -> u16 {
    match block {
        ResponseBlock::UserPrompt { .. } => 2,
        ResponseBlock::ThinkingCard { .. } => 3,
        ResponseBlock::ToolCard { expanded, .. } => {
            if *expanded {
                5
            } else {
                3
            }
        }
        ResponseBlock::MarkdownContent { parsed } => parsed.len().clamp(1, 100) as u16,
    }
}

impl<'a> Widget for ResponseList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut y: u16 = 0;
        let mut skip = self.scroll_offset;

        for block in self.blocks {
            let h = block_height(block);
            if skip > 0 {
                skip = skip.saturating_sub(h);
                continue;
            }
            if y >= area.height {
                break;
            }

            let remaining = area.height.saturating_sub(y);
            let block_area = Rect {
                x: area.x,
                y: area.y + y,
                width: area.width,
                height: h.min(remaining),
            };

            match block {
                ResponseBlock::UserPrompt { text } => {
                    PromptBlock::new(text.clone(), self.theme).render(block_area, buf);
                }
                ResponseBlock::ThinkingCard {
                    duration_secs,
                    content,
                    collapsed,
                } => {
                    ThinkingCard::new(
                        duration_secs.is_none() && content.is_none(),
                        *duration_secs,
                        content.clone(),
                        *collapsed,
                        self.theme,
                    )
                    .render(block_area, buf);
                }
                ResponseBlock::ToolCard {
                    name,
                    args_summary,
                    result,
                    duration_secs,
                    expanded,
                    is_error,
                } => {
                    ToolCard::new(
                        name.clone(),
                        args_summary.clone(),
                        result.clone(),
                        *duration_secs,
                        *expanded,
                        *is_error,
                        self.theme,
                    )
                    .render(block_area, buf);
                }
                ResponseBlock::MarkdownContent { parsed } => {
                    MarkdownWidget::new(parsed, self.theme).render(block_area, buf);
                }
            }

            y += h;
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn renders_without_panic() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let backend = TestBackend::new(80, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = crate::tui::theme::Theme::github_dark();

        let blocks = vec![
            super::ResponseBlock::UserPrompt {
                text: "hello".to_string(),
            },
            super::ResponseBlock::ThinkingCard {
                duration_secs: Some(1.2),
                content: None,
                collapsed: true,
            },
            super::ResponseBlock::MarkdownContent {
                parsed: crate::tui::markdown::parse_markdown("hi there", &theme),
            },
            super::ResponseBlock::ToolCard {
                name: "read_file".to_string(),
                args_summary: "main.rs".to_string(),
                result: Some("ok".to_string()),
                duration_secs: Some(0.1),
                expanded: false,
                is_error: false,
            },
        ];

        let widget = super::ResponseList::new(&blocks, &theme, 0);
        terminal
            .draw(|f| {
                f.render_widget(widget, f.area());
            })
            .unwrap();
    }
}
