use std::path::Path;

use rusqlite::{params, Connection};

use crate::compaction_record::CompactionRecord;
use crate::message_projection::{DurableTranscriptEntry, DurableTranscriptKind};
use crate::queue::QueueItem;
use crate::queue::SessionQueue;
use crate::recovery::{RecoveryError, RecoverySnapshot, SessionRecoveryStore};
use crate::session::Session;
use crate::storage::{project_visible_transcript, StorageError, TranscriptStore};
use crate::RuntimeMessage;

pub struct SqliteStorage {
    conn: Connection,
}

impl SqliteStorage {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS transcript (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                kind TEXT NOT NULL,
                content TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS recovery_message (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                kind TEXT NOT NULL,
                content TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS compaction_record (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                summary TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS compaction_record_message (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                compaction_record_id INTEGER NOT NULL,
                ordinal INTEGER NOT NULL,
                kind TEXT NOT NULL,
                content TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS queue_item (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                kind TEXT NOT NULL,
                content TEXT NOT NULL
            );",
        )?;

        Ok(Self { conn })
    }

    pub fn persist_session(&mut self, session: &Session) -> Result<(), StorageError> {
        let snapshot = session.snapshot();
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM transcript", [])?;
        tx.execute("DELETE FROM recovery_message", [])?;
        tx.execute("DELETE FROM compaction_record_message", [])?;
        tx.execute("DELETE FROM compaction_record", [])?;
        tx.execute("DELETE FROM queue_item", [])?;

        for entry in project_visible_transcript(&snapshot.history) {
            tx.execute(
                "INSERT INTO transcript (kind, content) VALUES (?1, ?2)",
                params![Self::kind_as_str(&entry.kind), entry.content],
            )?;
        }

        for queue_item in snapshot.queue.items() {
            let (kind, content) = Self::queue_item_parts(queue_item);
            tx.execute(
                "INSERT INTO queue_item (kind, content) VALUES (?1, ?2)",
                params![kind, content],
            )?;
        }

        for message in &snapshot.history {
            tx.execute(
                "INSERT INTO recovery_message (kind, content) VALUES (?1, ?2)",
                params![
                    Self::runtime_message_kind_as_str(message),
                    Self::runtime_message_content(message)
                ],
            )?;
        }

        tx.commit()?;
        for record in &snapshot.compaction_records {
            self.persist_compaction_record(record)?;
        }
        Ok(())
    }

    fn kind_as_str(kind: &DurableTranscriptKind) -> &'static str {
        match kind {
            DurableTranscriptKind::Assistant => "assistant",
            DurableTranscriptKind::RuntimeSummary => "runtime_summary",
            DurableTranscriptKind::ToolResult => "tool_result",
            DurableTranscriptKind::Warning => "warning",
            DurableTranscriptKind::RetryMarker => "retry_marker",
        }
    }

    fn kind_from_str(kind: &str) -> DurableTranscriptKind {
        match kind {
            "assistant" => DurableTranscriptKind::Assistant,
            "runtime_summary" => DurableTranscriptKind::RuntimeSummary,
            "tool_result" => DurableTranscriptKind::ToolResult,
            "warning" => DurableTranscriptKind::Warning,
            "retry_marker" => DurableTranscriptKind::RetryMarker,
            _ => DurableTranscriptKind::Warning,
        }
    }

    fn queue_item_parts(item: &QueueItem) -> (&'static str, String) {
        match item {
            QueueItem::Steering(message) => ("steering", message.clone()),
            QueueItem::FollowUp(message) => ("follow_up", message.clone()),
        }
    }

    fn queue_item_from_parts(kind: &str, content: String) -> QueueItem {
        match kind {
            "follow_up" => QueueItem::FollowUp(content),
            _ => QueueItem::Steering(content),
        }
    }

    fn runtime_message_kind_as_str(message: &RuntimeMessage) -> &'static str {
        match message {
            RuntimeMessage::User(_) => "user",
            RuntimeMessage::Assistant(_) => "assistant",
            RuntimeMessage::RuntimeSummary(_) => "runtime_summary",
            RuntimeMessage::ToolResult(_) => "tool_result",
            RuntimeMessage::Steering(_) => "steering",
            RuntimeMessage::FollowUp(_) => "follow_up",
            RuntimeMessage::Warning(_) => "warning",
            RuntimeMessage::RetryMarker(_) => "retry_marker",
        }
    }

    fn runtime_message_content(message: &RuntimeMessage) -> &str {
        match message {
            RuntimeMessage::User(content)
            | RuntimeMessage::Assistant(content)
            | RuntimeMessage::RuntimeSummary(content)
            | RuntimeMessage::ToolResult(content)
            | RuntimeMessage::Steering(content)
            | RuntimeMessage::FollowUp(content)
            | RuntimeMessage::Warning(content)
            | RuntimeMessage::RetryMarker(content) => content,
        }
    }

    fn runtime_message_from_parts(kind: &str, content: String) -> RuntimeMessage {
        match kind {
            "user" => RuntimeMessage::User(content),
            "assistant" => RuntimeMessage::Assistant(content),
            "runtime_summary" => RuntimeMessage::RuntimeSummary(content),
            "tool_result" => RuntimeMessage::ToolResult(content),
            "steering" => RuntimeMessage::Steering(content),
            "follow_up" => RuntimeMessage::FollowUp(content),
            "warning" => RuntimeMessage::Warning(content),
            "retry_marker" => RuntimeMessage::RetryMarker(content),
            _ => RuntimeMessage::Warning(content),
        }
    }

    pub fn persist_compaction_record(
        &mut self,
        record: &CompactionRecord,
    ) -> Result<(), StorageError> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO compaction_record (summary) VALUES (?1)",
            params![record.summary],
        )?;
        let compaction_record_id = tx.last_insert_rowid();

        for (ordinal, message) in record.original_messages.iter().enumerate() {
            tx.execute(
                "INSERT INTO compaction_record_message (compaction_record_id, ordinal, kind, content) VALUES (?1, ?2, ?3, ?4)",
                params![
                    compaction_record_id,
                    ordinal as i64,
                    Self::runtime_message_kind_as_str(message),
                    Self::runtime_message_content(message),
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn load_compaction_records(&self) -> Result<Vec<CompactionRecord>, StorageError> {
        let mut record_stmt = self
            .conn
            .prepare("SELECT id, summary FROM compaction_record ORDER BY id ASC")?;
        let rows = record_stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let summary: String = row.get(1)?;
            Ok((id, summary))
        })?;

        let mut records = Vec::new();
        for row in rows {
            let (record_id, summary) = row?;
            let mut message_stmt = self.conn.prepare(
                "SELECT kind, content FROM compaction_record_message WHERE compaction_record_id = ?1 ORDER BY ordinal ASC",
            )?;
            let messages = message_stmt.query_map(params![record_id], |message_row| {
                let kind: String = message_row.get(0)?;
                let content: String = message_row.get(1)?;
                Ok(Self::runtime_message_from_parts(&kind, content))
            })?;

            let mut original_messages = Vec::new();
            for message in messages {
                original_messages.push(message?);
            }

            records.push(CompactionRecord::new(summary, original_messages));
        }

        Ok(records)
    }
}

impl TranscriptStore for SqliteStorage {
    fn persist_runtime_messages(&mut self, history: &[RuntimeMessage]) -> Result<(), StorageError> {
        let transcript = project_visible_transcript(history);
        let tx = self.conn.transaction()?;

        for entry in transcript {
            tx.execute(
                "INSERT INTO transcript (kind, content) VALUES (?1, ?2)",
                params![Self::kind_as_str(&entry.kind), entry.content],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    fn load_transcript(&self) -> Result<Vec<DurableTranscriptEntry>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT kind, content FROM transcript ORDER BY id ASC")?;
        let rows = stmt.query_map([], |row| {
            let kind: String = row.get(0)?;
            let content: String = row.get(1)?;
            Ok(DurableTranscriptEntry {
                kind: Self::kind_from_str(&kind),
                content,
            })
        })?;

        let mut transcript = Vec::new();
        for row in rows {
            transcript.push(row?);
        }

        Ok(transcript)
    }
}

impl SessionRecoveryStore for SqliteStorage {
    fn load_recovery_snapshot(&self) -> Result<RecoverySnapshot, RecoveryError> {
        let history = self
            .load_recovery_history()
            .unwrap_or_else(|_| {
                self.load_transcript()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|entry| match entry.kind {
                        DurableTranscriptKind::Assistant => RuntimeMessage::Assistant(entry.content),
                        DurableTranscriptKind::RuntimeSummary => {
                            RuntimeMessage::RuntimeSummary(entry.content)
                        }
                        DurableTranscriptKind::ToolResult => RuntimeMessage::ToolResult(entry.content),
                        DurableTranscriptKind::Warning => RuntimeMessage::Warning(entry.content),
                        DurableTranscriptKind::RetryMarker => RuntimeMessage::RetryMarker(entry.content),
                    })
                    .collect()
            });

        let mut stmt = self
            .conn
            .prepare("SELECT kind, content FROM queue_item ORDER BY id ASC")
            .map_err(StorageError::from)?;
        let rows = stmt
            .query_map([], |row| {
                let kind: String = row.get(0)?;
                let content: String = row.get(1)?;
                Ok(Self::queue_item_from_parts(&kind, content))
            })
            .map_err(StorageError::from)?;

        let mut queue_items = Vec::new();
        for row in rows {
            queue_items.push(row.map_err(StorageError::from)?);
        }

        Ok(RecoverySnapshot {
            history,
            queue: SessionQueue::from_items(queue_items),
            compaction_records: self.load_compaction_records()?,
        })
    }
}

impl SqliteStorage {
    fn load_recovery_history(&self) -> Result<Vec<RuntimeMessage>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT kind, content FROM recovery_message ORDER BY id ASC")?;
        let rows = stmt.query_map([], |row| {
            let kind: String = row.get(0)?;
            let content: String = row.get(1)?;
            Ok(Self::runtime_message_from_parts(&kind, content))
        })?;

        let mut history = Vec::new();
        for row in rows {
            history.push(row?);
        }

        if history.is_empty() {
            return Err(StorageError::Sqlite(rusqlite::Error::QueryReturnedNoRows));
        }

        Ok(history)
    }
}
