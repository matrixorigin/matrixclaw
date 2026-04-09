use std::time::Instant;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Clear};
use ratatui::Frame;

use crate::tui::theme::Theme;
use crate::tui::widgets::input_bar::InputBar;
use crate::tui::widgets::response_list::ResponseBlock;
use crate::tui::widgets::response_list::ResponseList;

pub struct ChatApp {
    pub responses: Vec<ResponseBlock>,
    pub input: String,
    pub input_cursor: usize,
    pub input_history: Vec<String>,
    pub history_index: usize,
    pub scroll_offset: u16,
    pub auto_scroll: bool,
    pub theme: Theme,
    pub thinking_start: Option<Instant>,
    pub should_quit: bool,
    pub pending_markdown: String,
    pub active_tool_name: Option<String>,
    pub active_tool_start: Option<Instant>,
}

impl ChatApp {
    pub fn new(theme: Theme) -> Self {
        Self {
            responses: Vec::new(),
            input: String::new(),
            input_cursor: 0,
            input_history: Vec::new(),
            history_index: 0,
            scroll_offset: 0,
            auto_scroll: true,
            theme,
            thinking_start: None,
            should_quit: false,
            pending_markdown: String::new(),
            active_tool_name: None,
            active_tool_start: None,
        }
    }

    pub fn handle_event(&mut self, event: crate::tui::event::AppEvent) -> Option<String> {
        match event {
            crate::tui::event::AppEvent::Key(key) => {
                use crossterm::event::{KeyCode, KeyModifiers};
                match (key.modifiers, key.code) {
                    (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                        self.should_quit = true;
                        None
                    }
                    (_, KeyCode::Enter) => {
                        if self.input.is_empty() {
                            return None;
                        }
                        let text = self.input.clone();
                        self.responses
                            .push(ResponseBlock::UserPrompt { text: text.clone() });
                        self.input_history.push(text.clone());
                        self.history_index = self.input_history.len();
                        self.input.clear();
                        self.input_cursor = 0;
                        self.thinking_start = Some(Instant::now());
                        self.auto_scroll = true;
                        Some(text)
                    }
                    (_, KeyCode::Char('d')) => {
                        self.toggle_last_expand();
                        None
                    }
                    (_, KeyCode::Char(c)) => {
                        self.input.insert(self.input_cursor, c);
                        self.input_cursor += 1;
                        None
                    }
                    (_, KeyCode::Backspace) => {
                        if self.input_cursor > 0 {
                            self.input_cursor -= 1;
                            self.input.remove(self.input_cursor);
                        }
                        None
                    }
                    (_, KeyCode::Left) => {
                        if self.input_cursor > 0 {
                            self.input_cursor -= 1;
                        }
                        None
                    }
                    (_, KeyCode::Right) => {
                        if self.input_cursor < self.input.len() {
                            self.input_cursor += 1;
                        }
                        None
                    }
                    (_, KeyCode::Up) => {
                        if self.history_index > 0 {
                            self.history_index -= 1;
                            self.input = self.input_history[self.history_index].clone();
                            self.input_cursor = self.input.len();
                        }
                        None
                    }
                    (_, KeyCode::Down) => {
                        if self.history_index < self.input_history.len() - 1 {
                            self.history_index += 1;
                            self.input = self.input_history[self.history_index].clone();
                        } else {
                            self.history_index = self.input_history.len();
                            self.input.clear();
                        }
                        self.input_cursor = self.input.len();
                        None
                    }
                    (_, KeyCode::PageUp) => {
                        self.auto_scroll = false;
                        self.scroll_offset = self.scroll_offset.saturating_add(10);
                        None
                    }
                    (_, KeyCode::PageDown) => {
                        self.scroll_offset = self.scroll_offset.saturating_sub(10);
                        if self.scroll_offset == 0 {
                            self.auto_scroll = true;
                        }
                        None
                    }
                    _ => None,
                }
            }
            crate::tui::event::AppEvent::Agent(live_event) => {
                self.handle_agent_event(live_event);
                None
            }
            crate::tui::event::AppEvent::Quit => {
                self.should_quit = true;
                None
            }
        }
    }

    fn handle_agent_event(&mut self, event: crate::live_runtime::LiveRunEvent) {
        match event.kind.as_str() {
            "message_started" => {
                self.thinking_start = Some(Instant::now());
                self.flush_markdown();
                self.responses.push(ResponseBlock::ThinkingCard {
                    duration_secs: None,
                    content: None,
                    collapsed: false,
                });
            }
            "message_delta" => {
                if let Some(content) = &event.content {
                    self.pending_markdown.push_str(content);
                }
            }
            "tool_call_started" => {
                self.flush_markdown();
                self.finalize_thinking();
                let name = event.content.clone().unwrap_or_default();
                self.active_tool_name = Some(name.clone());
                self.active_tool_start = Some(Instant::now());
                self.responses.push(ResponseBlock::ToolCard {
                    name,
                    args_summary: String::new(),
                    result: None,
                    duration_secs: None,
                    expanded: false,
                    is_error: false,
                });
            }
            "tool_execution_completed" => {
                if let Some(start) = self.active_tool_start.take() {
                    let duration = start.elapsed().as_secs_f64();
                    if let Some(ResponseBlock::ToolCard {
                        duration_secs,
                        result,
                        is_error,
                        ..
                    }) = self.responses.last_mut()
                    {
                        *duration_secs = Some(duration);
                        *result = Some(event.content.clone().unwrap_or_default());
                        *is_error = event
                            .content
                            .as_deref()
                            .is_some_and(|c| c.contains("error") || c.contains("Error"));
                    }
                }
                self.active_tool_name = None;
            }
            "message_completed" => {
                self.finalize_thinking();
                if let Some(content) = event.content {
                    self.pending_markdown.push_str(&content);
                }
                self.flush_markdown();
                self.thinking_start = None;
            }
            "run_started" | "run_completed" => {}
            _ => {}
        }
    }

    fn flush_markdown(&mut self) {
        if self.pending_markdown.is_empty() {
            return;
        }
        let parsed = crate::tui::markdown::parse_markdown(&self.pending_markdown, &self.theme);
        self.responses
            .push(ResponseBlock::MarkdownContent { parsed });
        self.pending_markdown.clear();
    }

    fn finalize_thinking(&mut self) {
        if let Some(start) = self.thinking_start.take() {
            let duration = start.elapsed().as_secs_f64();
            if let Some(ResponseBlock::ThinkingCard {
                duration_secs,
                collapsed,
                ..
            }) = self.responses.last_mut()
            {
                *duration_secs = Some(duration);
                *collapsed = true;
            }
        }
    }

    fn toggle_last_expand(&mut self) {
        if let Some(block) = self.responses.last_mut() {
            match block {
                ResponseBlock::ToolCard { expanded, .. } => *expanded = !*expanded,
                ResponseBlock::ThinkingCard {
                    collapsed, content, ..
                } => {
                    if content.is_some() {
                        *collapsed = !*collapsed;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn draw(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(f.area());

        f.render_widget(Clear, f.area());
        let bg = Block::default().style(ratatui::style::Style::default().bg(self.theme.background));
        f.render_widget(bg, f.area());

        let response_list = ResponseList::new(&self.responses, &self.theme, self.scroll_offset);
        f.render_widget(response_list, chunks[0]);

        let input_bar = InputBar::new(&self.input, self.input_cursor, &self.theme);
        f.render_widget(input_bar, chunks[1]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn app_handles_key_input() {
        let theme = crate::tui::theme::Theme::github_dark();
        let mut app = ChatApp::new(theme);

        app.handle_event(crate::tui::event::AppEvent::Key(KeyEvent::new(
            KeyCode::Char('h'),
            KeyModifiers::NONE,
        )));
        app.handle_event(crate::tui::event::AppEvent::Key(KeyEvent::new(
            KeyCode::Char('i'),
            KeyModifiers::NONE,
        )));

        assert_eq!(app.input, "hi");
        assert_eq!(app.input_cursor, 2);
    }

    #[test]
    fn app_enters_text_and_submits() {
        let theme = crate::tui::theme::Theme::github_dark();
        let mut app = ChatApp::new(theme);

        app.handle_event(crate::tui::event::AppEvent::Key(KeyEvent::new(
            KeyCode::Char('h'),
            KeyModifiers::NONE,
        )));
        let result = app.handle_event(crate::tui::event::AppEvent::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));

        assert_eq!(result, Some("h".to_string()));
        assert!(app.input.is_empty());
        assert_eq!(app.responses.len(), 1);
    }

    #[test]
    fn app_quit_on_ctrl_c() {
        let theme = crate::tui::theme::Theme::github_dark();
        let mut app = ChatApp::new(theme);

        app.handle_event(crate::tui::event::AppEvent::Key(KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
        )));

        assert!(app.should_quit);
    }
}
