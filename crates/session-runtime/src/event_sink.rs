use crate::message_projection::{DurableTranscriptEntry, DurableTranscriptKind};
use crate::storage::{StorageError, TranscriptStore};
use crate::RuntimeMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisibleTranscriptEvent {
    Assistant(String),
    RuntimeSummary(String),
    ToolResult(String),
    Warning(String),
    RetryMarker(String),
}

impl VisibleTranscriptEvent {
    fn into_entry(self) -> DurableTranscriptEntry {
        match self {
            VisibleTranscriptEvent::RuntimeSummary(content) => DurableTranscriptEntry {
                kind: DurableTranscriptKind::RuntimeSummary,
                content,
            },
            VisibleTranscriptEvent::Assistant(content) => DurableTranscriptEntry {
                kind: DurableTranscriptKind::Assistant,
                content,
            },
            VisibleTranscriptEvent::ToolResult(content) => DurableTranscriptEntry {
                kind: DurableTranscriptKind::ToolResult,
                content,
            },
            VisibleTranscriptEvent::Warning(content) => DurableTranscriptEntry {
                kind: DurableTranscriptKind::Warning,
                content,
            },
            VisibleTranscriptEvent::RetryMarker(content) => DurableTranscriptEntry {
                kind: DurableTranscriptKind::RetryMarker,
                content,
            },
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TranscriptEventSink {
    events: Vec<VisibleTranscriptEvent>,
}

impl TranscriptEventSink {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn from_history(history: &[RuntimeMessage]) -> Self {
        let mut sink = Self::new();
        sink.extend(history);
        sink
    }

    pub fn record(&mut self, message: &RuntimeMessage) {
        if let Some(event) = project_visible_transcript_event(message) {
            self.events.push(event);
        }
    }

    pub fn extend(&mut self, history: &[RuntimeMessage]) {
        for message in history {
            self.record(message);
        }
    }

    pub fn into_entries(self) -> Vec<DurableTranscriptEntry> {
        self.events
            .into_iter()
            .map(VisibleTranscriptEvent::into_entry)
            .collect()
    }
}

pub fn project_visible_transcript_event(
    message: &RuntimeMessage,
) -> Option<VisibleTranscriptEvent> {
    match message {
        RuntimeMessage::User(_) => None,
        RuntimeMessage::Assistant(content) => {
            Some(VisibleTranscriptEvent::Assistant(content.clone()))
        }
        RuntimeMessage::RuntimeSummary(content) => {
            Some(VisibleTranscriptEvent::RuntimeSummary(content.clone()))
        }
        RuntimeMessage::ToolResult(content) => {
            Some(VisibleTranscriptEvent::ToolResult(content.clone()))
        }
        RuntimeMessage::Warning(content) => Some(VisibleTranscriptEvent::Warning(content.clone())),
        RuntimeMessage::RetryMarker(content) => {
            Some(VisibleTranscriptEvent::RetryMarker(content.clone()))
        }
        RuntimeMessage::Steering(_) | RuntimeMessage::FollowUp(_) => None,
    }
}

pub fn persist_visible_transcript(
    store: &mut dyn TranscriptStore,
    history: &[RuntimeMessage],
) -> Result<(), StorageError> {
    store.persist_runtime_messages(history)
}
