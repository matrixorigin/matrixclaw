use std::fmt;

use crate::compaction_record::CompactionRecord;
use crate::context_builder::{ContinuationContext, ContextBuilder};
use crate::queue::SessionQueue;
use crate::session::Session;
use crate::storage::{StorageError, TranscriptStore};
use crate::RuntimeMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoverySnapshot {
    pub history: Vec<RuntimeMessage>,
    pub queue: SessionQueue,
    pub compaction_records: Vec<CompactionRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveredSession {
    pub session: Session,
    pub context: ContinuationContext,
}

#[derive(Debug)]
pub enum RecoveryError {
    Storage(StorageError),
}

impl fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecoveryError::Storage(error) => write!(f, "storage recovery error: {}", error),
        }
    }
}

impl std::error::Error for RecoveryError {}

impl From<StorageError> for RecoveryError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

pub trait SessionRecoveryStore: TranscriptStore {
    fn load_recovery_snapshot(&self) -> Result<RecoverySnapshot, RecoveryError>;
}

pub fn restore_session(snapshot: RecoverySnapshot) -> RecoveredSession {
    let session = Session::from_parts(snapshot.history, snapshot.queue, snapshot.compaction_records);
    let context = ContextBuilder::build(&session);
    RecoveredSession { session, context }
}
