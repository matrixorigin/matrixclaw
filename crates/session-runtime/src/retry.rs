use crate::compaction::CompactionResult;
use crate::error::{RunFailure, RunFailureKind};
use crate::RuntimeMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryDecision {
    pub should_retry: bool,
    pub compact_before_retry: bool,
}

impl RetryDecision {
    pub fn for_failure(failure: &RunFailure) -> Self {
        match failure.kind {
            RunFailureKind::Overflow => Self {
                should_retry: true,
                compact_before_retry: true,
            },
            RunFailureKind::Transient => Self {
                should_retry: true,
                compact_before_retry: false,
            },
            RunFailureKind::Fatal => Self {
                should_retry: false,
                compact_before_retry: false,
            },
        }
    }

    pub fn requires_compaction(&self) -> bool {
        self.should_retry && self.compact_before_retry
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryOutcome {
    pub retried_context: Vec<RuntimeMessage>,
    pub compaction: Option<CompactionResult>,
}

impl RetryOutcome {
    pub fn from_compaction(result: CompactionResult) -> Self {
        Self {
            retried_context: result.compacted_context.clone(),
            compaction: Some(result),
        }
    }
}
