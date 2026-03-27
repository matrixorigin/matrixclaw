use crate::message_projection::{
    project_compaction_summary, CompactionSummaryArtifact, SummaryArtifactRole,
};
use crate::RuntimeMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionRecord {
    pub summary: String,
    pub original_messages: Vec<RuntimeMessage>,
}

impl CompactionRecord {
    pub fn new(summary: impl Into<String>, original_messages: Vec<RuntimeMessage>) -> Self {
        Self {
            summary: summary.into(),
            original_messages,
        }
    }

    pub fn summary_message(&self) -> RuntimeMessage {
        RuntimeMessage::RuntimeSummary(self.summary.clone())
    }

    pub fn summary_artifact(&self) -> CompactionSummaryArtifact {
        project_compaction_summary(self.summary.clone())
    }

    pub fn summary_role(&self) -> SummaryArtifactRole {
        self.summary_artifact().role
    }
}
