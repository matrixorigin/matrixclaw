use crate::compaction_record::CompactionRecord;
use crate::message_projection::CompactionSummaryArtifact;
use crate::RuntimeMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionRequest {
    pub active_context: Vec<RuntimeMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionResult {
    pub compacted_context: Vec<RuntimeMessage>,
    pub summary: String,
    pub record: CompactionRecord,
}

impl CompactionResult {
    pub fn summary_artifact(&self) -> CompactionSummaryArtifact {
        self.record.summary_artifact()
    }
}

pub trait Compactor {
    fn compact(&mut self, request: &CompactionRequest) -> CompactionResult;
}

pub fn remove_failure_only_context(history: &[RuntimeMessage]) -> Vec<RuntimeMessage> {
    history
        .iter()
        .filter_map(|message| match message {
            RuntimeMessage::Warning(_) | RuntimeMessage::RetryMarker(_) => None,
            other => Some(other.clone()),
        })
        .collect()
}

pub fn prepare_compaction_request(history: &[RuntimeMessage]) -> CompactionRequest {
    CompactionRequest {
        active_context: remove_failure_only_context(history),
    }
}

pub fn compact_with_summary(
    history: &[RuntimeMessage],
    summary: impl Into<String>,
) -> CompactionResult {
    let summary = summary.into();
    let record = CompactionRecord::new(summary.clone(), history.to_vec());

    CompactionResult {
        compacted_context: vec![record.summary_message()],
        summary,
        record,
    }
}
