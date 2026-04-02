use std::fmt;

use crate::event_sink::TranscriptEventSink;
use crate::message_projection::DurableTranscriptEntry;
use crate::RuntimeMessage;

#[derive(Debug)]
pub enum StorageError {
    Sqlite(rusqlite::Error),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Sqlite(error) => write!(f, "sqlite error: {error}"),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<rusqlite::Error> for StorageError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

pub trait TranscriptStore {
    fn persist_runtime_messages(&mut self, history: &[RuntimeMessage]) -> Result<(), StorageError>;
    fn load_transcript(&self) -> Result<Vec<DurableTranscriptEntry>, StorageError>;
}

pub fn project_visible_transcript(history: &[RuntimeMessage]) -> Vec<DurableTranscriptEntry> {
    TranscriptEventSink::from_history(history).into_entries()
}
