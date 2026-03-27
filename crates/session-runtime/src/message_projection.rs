use crate::event_sink::TranscriptEventSink;
use crate::queue::SessionQueue;
use crate::RuntimeMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DurableTranscriptKind {
    Assistant,
    RuntimeSummary,
    ToolResult,
    Warning,
    RetryMarker,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurableTranscriptEntry {
    pub kind: DurableTranscriptKind,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SummaryArtifactRole {
    RuntimeSystem,
    UserAuthored,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionSummaryArtifact {
    pub role: SummaryArtifactRole,
    pub content: String,
}

pub fn project_compaction_summary(content: impl Into<String>) -> CompactionSummaryArtifact {
    CompactionSummaryArtifact {
        role: SummaryArtifactRole::RuntimeSystem,
        content: content.into(),
    }
}

pub fn project_next_turn(history: &[RuntimeMessage], queue: &SessionQueue) -> Vec<RuntimeMessage> {
    let mut projected = history.to_vec();

    projected.extend(
        queue
            .steering_items()
            .map(|message| RuntimeMessage::Steering(message.to_string())),
    );

    projected.push(RuntimeMessage::Assistant("next assistant turn".to_string()));
    projected
}

pub fn project_next_run(history: &[RuntimeMessage], queue: &SessionQueue) -> Vec<RuntimeMessage> {
    let mut projected = history.to_vec();

    projected.extend(
        queue
            .follow_up_items()
            .map(|message| RuntimeMessage::FollowUp(message.to_string())),
    );

    projected.push(RuntimeMessage::Assistant(
        "next run assistant turn".to_string(),
    ));
    projected
}

pub fn project_durable_transcript(history: &[RuntimeMessage]) -> Vec<DurableTranscriptEntry> {
    TranscriptEventSink::from_history(history).into_entries()
}
