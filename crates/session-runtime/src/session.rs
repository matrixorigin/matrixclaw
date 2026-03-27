use crate::compaction_record::CompactionRecord;
use crate::message_projection;
use crate::queue::SessionQueue;
use crate::RuntimeMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    history: Vec<RuntimeMessage>,
    queue: SessionQueue,
    compaction_records: Vec<CompactionRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSnapshot {
    pub history: Vec<RuntimeMessage>,
    pub queue: SessionQueue,
    pub compaction_records: Vec<CompactionRecord>,
}

impl Session {
    pub fn new(history: Vec<RuntimeMessage>) -> Self {
        Self {
            history,
            queue: SessionQueue::new(),
            compaction_records: Vec::new(),
        }
    }

    pub fn from_parts(
        history: Vec<RuntimeMessage>,
        queue: SessionQueue,
        compaction_records: Vec<CompactionRecord>,
    ) -> Self {
        Self {
            history,
            queue,
            compaction_records,
        }
    }

    pub fn snapshot(&self) -> SessionSnapshot {
        SessionSnapshot {
            history: self.history.clone(),
            queue: self.queue.clone(),
            compaction_records: self.compaction_records.clone(),
        }
    }

    pub fn queue_steering_message(&mut self, message: impl Into<String>) {
        self.queue.push_steering(message);
    }

    pub fn queue_follow_up_message(&mut self, message: impl Into<String>) {
        self.queue.push_follow_up(message);
    }

    pub fn history(&self) -> &[RuntimeMessage] {
        &self.history
    }

    pub fn history_mut(&mut self) -> &mut Vec<RuntimeMessage> {
        &mut self.history
    }

    pub fn queue(&self) -> &SessionQueue {
        &self.queue
    }

    pub fn drain_steering_messages(&mut self) -> Vec<String> {
        self.queue.drain_steering_items()
    }

    pub fn drain_follow_up_messages(&mut self) -> Vec<String> {
        self.queue.drain_follow_up_items()
    }

    pub fn compaction_records(&self) -> &[CompactionRecord] {
        &self.compaction_records
    }

    pub fn record_compaction(&mut self, record: CompactionRecord) {
        self.compaction_records.push(record);
    }

    pub fn project_next_turn(&self) -> Vec<RuntimeMessage> {
        message_projection::project_next_turn(&self.history, &self.queue)
    }

    pub fn project_next_run(&self) -> Vec<RuntimeMessage> {
        message_projection::project_next_run(&self.history, &self.queue)
    }

    pub fn complete_current_run(&mut self) -> Vec<RuntimeMessage> {
        let mut projected = self.history.clone();
        let drained_follow_ups = self.queue.drain_follow_up_items();

        projected.extend(drained_follow_ups.into_iter().map(RuntimeMessage::FollowUp));
        projected.push(RuntimeMessage::Assistant(
            "next run assistant turn".to_string(),
        ));
        projected
    }
}
