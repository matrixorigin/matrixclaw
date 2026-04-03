use crate::RuntimeMessage;

#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub max_tool_result_chars: usize,
    pub compression_threshold_pct: f64,
    pub protect_last_n: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            max_tool_result_chars: 200,
            compression_threshold_pct: 0.50,
            protect_last_n: 20,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompressedContext {
    pub head: Vec<RuntimeMessage>,
    pub summary: String,
    pub tail: Vec<RuntimeMessage>,
    pub was_compressed: bool,
}

#[derive(Debug, Clone)]
pub struct CompressionResult {
    pub messages: Vec<RuntimeMessage>,
    pub removed_count: usize,
    pub was_compressed: bool,
}

pub fn prune_tool_results(messages: &[RuntimeMessage], max_chars: usize) -> Vec<RuntimeMessage> {
    messages
        .iter()
        .map(|msg| match msg {
            RuntimeMessage::ToolResult(content) if content.len() > max_chars => {
                RuntimeMessage::ToolResult(format!(
                    "[tool result truncated, {} chars]",
                    content.len()
                ))
            }
            other => other.clone(),
        })
        .collect()
}

pub fn calculate_boundaries(total: usize, head_size: usize, tail_size: usize) -> (usize, usize) {
    let tail_size = tail_size.max(head_size);
    let tail_start = total.saturating_sub(tail_size);
    let head_end = head_size.min(tail_start);
    (head_end, tail_start)
}

fn adjust_for_pairs(
    messages: &[RuntimeMessage],
    head_end: usize,
    tail_start: usize,
) -> (usize, usize) {
    let mut head_end = head_end;
    let mut tail_start = tail_start;

    if head_end < messages.len() - 1 {
        if let RuntimeMessage::ToolResult(_) = messages[head_end] {
            head_end += 1;
        }
    }

    if tail_start > 0 && tail_start < messages.len() {
        if let RuntimeMessage::ToolResult(_) = messages[tail_start] {
            tail_start = tail_start.saturating_sub(1);
        }
    }

    (head_end, tail_start)
}

pub fn compress(
    messages: &[RuntimeMessage],
    summary: String,
    config: &CompressionConfig,
) -> CompressionResult {
    let threshold = config.protect_last_n * 2;
    if messages.len() <= threshold {
        return CompressionResult {
            messages: messages.to_vec(),
            removed_count: 0,
            was_compressed: false,
        };
    }

    let head_size =
        (messages.len() as f64 * config.compression_threshold_pct / 2.0).ceil() as usize;
    let (mut head_end, mut tail_start) =
        calculate_boundaries(messages.len(), head_size, config.protect_last_n);

    (head_end, tail_start) = adjust_for_pairs(messages, head_end, tail_start);

    if head_end >= tail_start {
        return CompressionResult {
            messages: messages.to_vec(),
            removed_count: 0,
            was_compressed: false,
        };
    }

    let removed_count = tail_start - head_end;

    let mut result = Vec::with_capacity(head_end + 1 + (messages.len() - tail_start));
    result.extend(messages[..head_end].to_vec());
    result.push(RuntimeMessage::RuntimeSummary(summary));
    result.extend(messages[tail_start..].to_vec());

    CompressionResult {
        messages: result,
        removed_count,
        was_compressed: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_messages(count: usize) -> Vec<RuntimeMessage> {
        (0..count)
            .map(|i| {
                if i % 2 == 0 {
                    RuntimeMessage::User(format!("message {i}"))
                } else {
                    RuntimeMessage::Assistant(format!("reply {i}"))
                }
            })
            .collect()
    }

    #[test]
    fn prune_truncates_long_tool_results() {
        let messages = vec![
            RuntimeMessage::User("do something".into()),
            RuntimeMessage::ToolResult("x".repeat(300)),
        ];
        let pruned = prune_tool_results(&messages, 200);
        assert_eq!(pruned.len(), 2);
        assert!(matches!(&pruned[1], RuntimeMessage::ToolResult(s) if s.contains("300 chars")));
    }

    #[test]
    fn prune_preserves_short_results() {
        let messages = vec![
            RuntimeMessage::User("do something".into()),
            RuntimeMessage::ToolResult("ok".into()),
        ];
        let pruned = prune_tool_results(&messages, 200);
        assert_eq!(pruned, messages);
    }

    #[test]
    fn prune_preserves_non_tool_messages() {
        let messages = vec![
            RuntimeMessage::User("x".repeat(500)),
            RuntimeMessage::Assistant("y".repeat(500)),
        ];
        let pruned = prune_tool_results(&messages, 200);
        assert_eq!(pruned, messages);
    }

    #[test]
    fn compress_splits_head_and_tail() {
        let messages = make_messages(30);
        let config = CompressionConfig {
            protect_last_n: 5,
            ..Default::default()
        };
        let result = compress(&messages, "summary".into(), &config);
        assert!(result.was_compressed);
        assert!(result.messages.len() < messages.len());
        assert!(result.removed_count > 0);

        let has_summary = result
            .messages
            .iter()
            .any(|m| matches!(m, RuntimeMessage::RuntimeSummary(s) if s == "summary"));
        assert!(has_summary);
    }

    #[test]
    fn compress_small_context_unchanged() {
        let messages = make_messages(5);
        let config = CompressionConfig::default();
        let result = compress(&messages, "summary".into(), &config);
        assert!(!result.was_compressed);
        assert_eq!(result.messages, messages);
        assert_eq!(result.removed_count, 0);
    }

    #[test]
    fn calculate_boundaries_clamps_tail() {
        let (head_end, tail_start) = calculate_boundaries(100, 5, 20);
        assert_eq!(head_end, 5);
        assert_eq!(tail_start, 80);
        assert!(head_end <= tail_start);
    }

    #[test]
    fn calculate_boundaries_ensures_tail_minimum() {
        let (head_end, tail_start) = calculate_boundaries(50, 2, 5);
        assert_eq!(head_end, 2);
        assert_eq!(tail_start, 45);
    }

    #[test]
    fn compress_does_not_split_tool_result_pair() {
        let mut messages = make_messages(30);
        messages.insert(10, RuntimeMessage::ToolResult("result data".into()));

        let config = CompressionConfig {
            protect_last_n: 5,
            ..Default::default()
        };
        let result = compress(&messages, "summary".into(), &config);
        assert!(result.was_compressed);

        for (i, msg) in result.messages.iter().enumerate() {
            if matches!(msg, RuntimeMessage::ToolResult(_)) {
                if i > 0 {
                    assert!(
                        !matches!(result.messages[i - 1], RuntimeMessage::RuntimeSummary(_)),
                        "tool result immediately follows summary boundary"
                    );
                }
            }
        }
    }
}
