mod syntax;

use comrak::nodes::NodeValue;
use ratatui::style::{Modifier, Style};

use crate::tui::theme::Theme;

pub struct StyledLine(pub Vec<(Style, String)>);

pub fn parse_markdown(input: &str, theme: &Theme) -> Vec<StyledLine> {
    let arena = comrak::Arena::new();
    let root = comrak::parse_document(&arena, input, &comrak::Options::default());

    let mut lines: Vec<StyledLine> = Vec::new();

    for node in root.children() {
        match &node.data.borrow().value {
            NodeValue::Heading(n) => {
                let prefix = "#".repeat(n.level as usize) + " ";
                let text = collect_text(node);
                lines.push(StyledLine(vec![(
                    Style::default()
                        .fg(theme.heading)
                        .add_modifier(Modifier::BOLD),
                    prefix + &text,
                )]));
                lines.push(StyledLine(vec![]));
            }
            NodeValue::Paragraph => {
                let spans = collect_inline_spans(node, theme);
                lines.push(StyledLine(spans));
                lines.push(StyledLine(vec![]));
            }
            NodeValue::CodeBlock(code) => {
                let highlighter = syntax::SyntaxHighlighter::new(&theme.syntax_highlight_theme);
                let lang = if code.info.is_empty() {
                    None
                } else {
                    Some(code.info.as_str())
                };
                let highlighted = highlighter.highlight(&code.literal, lang);

                let mut current_line_spans: Vec<(Style, String)> = Vec::new();
                for (sstyle, text) in highlighted {
                    let rat_style = syntax::SyntaxHighlighter::syntect_style_to_ratatui(sstyle)
                        .bg(theme.code_bg);
                    for (i, part) in text.split('\n').enumerate() {
                        if i > 0 && !current_line_spans.is_empty() {
                            lines.push(StyledLine(std::mem::take(&mut current_line_spans)));
                        }
                        if !part.is_empty() {
                            current_line_spans.push((rat_style, part.to_string()));
                        }
                    }
                }
                if !current_line_spans.is_empty() {
                    lines.push(StyledLine(current_line_spans));
                }
                lines.push(StyledLine(vec![]));
            }
            NodeValue::List(_) => {
                let mut i = 1;
                for item in node.children() {
                    let prefix = format!("  {i}. ");
                    let mut spans = vec![(Style::default().fg(theme.foreground), prefix)];
                    spans.extend(collect_inline_spans(item, theme));
                    lines.push(StyledLine(spans));
                    i += 1;
                }
                lines.push(StyledLine(vec![]));
            }
            NodeValue::BlockQuote => {
                for child in node.children() {
                    let mut quoted = vec![(Style::default().fg(theme.border), "│ ".to_string())];
                    quoted.extend(collect_inline_spans(child, theme));
                    lines.push(StyledLine(quoted));
                }
            }
            NodeValue::ThematicBreak => {
                lines.push(StyledLine(vec![(
                    Style::default().fg(theme.border),
                    "─".repeat(40),
                )]));
                lines.push(StyledLine(vec![]));
            }
            _ => {
                let spans = collect_inline_spans(node, theme);
                if !spans.is_empty() {
                    lines.push(StyledLine(spans));
                }
            }
        }
    }

    lines
}

fn collect_inline_spans<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    theme: &Theme,
) -> Vec<(Style, String)> {
    let mut spans = Vec::new();

    for child in node.children() {
        match &child.data.borrow().value {
            NodeValue::Text(text) => {
                spans.push((Style::default().fg(theme.foreground), text.clone()));
            }
            NodeValue::Code(code) => {
                spans.push((
                    Style::default().fg(theme.code_fg).bg(theme.code_bg),
                    code.literal.clone(),
                ));
            }
            NodeValue::Strong => {
                let inner = collect_text(child);
                spans.push((
                    Style::default().fg(theme.bold).add_modifier(Modifier::BOLD),
                    inner,
                ));
            }
            NodeValue::Emph => {
                let inner = collect_text(child);
                spans.push((
                    Style::default()
                        .fg(theme.italic)
                        .add_modifier(Modifier::ITALIC),
                    inner,
                ));
            }
            NodeValue::Link(_) => {
                let inner = collect_text(child);
                spans.push((
                    Style::default()
                        .fg(theme.link)
                        .add_modifier(Modifier::UNDERLINED),
                    inner,
                ));
            }
            NodeValue::SoftBreak | NodeValue::LineBreak => {
                spans.push((Style::default(), " ".to_string()));
            }
            _ => {
                spans.extend(collect_inline_spans(child, theme));
            }
        }
    }

    spans
}

fn collect_text<'a>(node: &'a comrak::nodes::AstNode<'a>) -> String {
    let mut result = String::new();
    collect_text_recursive(node, &mut result);
    result
}

fn collect_text_recursive<'a>(node: &'a comrak::nodes::AstNode<'a>, out: &mut String) {
    match &node.data.borrow().value {
        NodeValue::Text(text) => {
            out.push_str(text);
        }
        NodeValue::Code(code) => {
            out.push_str(&code.literal);
        }
        NodeValue::SoftBreak | NodeValue::LineBreak => {
            out.push(' ');
        }
        _ => {
            for child in node.children() {
                collect_text_recursive(child, out);
            }
        }
    }
}

pub fn styled_lines_to_ratatui(styled: &[StyledLine]) -> Vec<ratatui::text::Line<'_>> {
    styled
        .iter()
        .map(|StyledLine(spans)| {
            ratatui::text::Line::from(
                spans
                    .iter()
                    .map(|(style, text)| ratatui::text::Span::styled(text.clone(), *style))
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_theme() -> crate::tui::theme::Theme {
        crate::tui::theme::Theme::github_dark()
    }

    #[test]
    fn renders_heading() {
        let lines = parse_markdown("# Hello", &test_theme());
        let text: String = lines
            .iter()
            .flat_map(|l| l.0.iter().map(|(_, t)| t.as_str()))
            .collect();
        assert!(text.contains("Hello"));
    }

    #[test]
    fn renders_bold() {
        let lines = parse_markdown("**bold**", &test_theme());
        assert!(!lines.is_empty());
        let has_bold = lines.iter().any(|l| {
            l.0.iter()
                .any(|(s, _)| s.add_modifier.contains(Modifier::BOLD))
        });
        assert!(has_bold);
    }

    #[test]
    fn renders_code_inline() {
        let lines = parse_markdown("`cargo build`", &test_theme());
        let text: String = lines
            .iter()
            .flat_map(|l| l.0.iter().map(|(_, t)| t.as_str()))
            .collect();
        assert!(text.contains("cargo build"));
    }

    #[test]
    fn renders_code_block() {
        let lines = parse_markdown("```rust\nfn main() {}\n```", &test_theme());
        assert!(lines.len() >= 2);
    }

    #[test]
    fn renders_link() {
        let lines = parse_markdown("[click here](https://example.com)", &test_theme());
        let text: String = lines
            .iter()
            .flat_map(|l| l.0.iter().map(|(_, t)| t.as_str()))
            .collect();
        assert!(text.contains("click here"));
    }

    #[test]
    fn styled_lines_to_ratatui_conversion() {
        let styled = vec![StyledLine(vec![(Style::default(), "hello".to_string())])];
        let rat_lines = styled_lines_to_ratatui(&styled);
        assert_eq!(rat_lines.len(), 1);
    }
}
