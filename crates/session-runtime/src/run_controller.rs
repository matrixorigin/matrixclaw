use crate::compaction::{CompactionRequest, Compactor};
use crate::error::RunFailure;
use crate::queue::SessionQueue;
use crate::retry::{RetryDecision, RetryOutcome};
use crate::session::Session;
use crate::RuntimeMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunController {
    session: Session,
}

impl RunController {
    pub fn new(history: Vec<RuntimeMessage>) -> Self {
        Self {
            session: Session::new(history),
        }
    }

    pub fn queue_steering_message(&mut self, message: impl Into<String>) {
        self.session.queue_steering_message(message);
    }

    pub fn queue_follow_up_message(&mut self, message: impl Into<String>) {
        self.session.queue_follow_up_message(message);
    }

    pub fn record_tool_result(&mut self, result: impl Into<String>) {
        self.session
            .history_mut()
            .push(RuntimeMessage::ToolResult(result.into()));
    }

    pub fn project_next_turn(&self) -> Vec<RuntimeMessage> {
        self.session.project_next_turn()
    }

    pub fn project_next_run(&self) -> Vec<RuntimeMessage> {
        self.session.project_next_run()
    }

    pub fn complete_current_run(&mut self) -> Vec<RuntimeMessage> {
        self.session.complete_current_run()
    }

    pub fn queue(&self) -> &SessionQueue {
        self.session.queue()
    }

    pub fn active_context(&self) -> Vec<RuntimeMessage> {
        self.session.history().to_vec()
    }

    pub fn handle_run_failure(
        &mut self,
        failure: &RunFailure,
        compactor: &mut dyn Compactor,
    ) -> Option<RetryOutcome> {
        let decision = RetryDecision::for_failure(failure);
        if !decision.should_retry {
            return None;
        }

        if decision.requires_compaction() {
            let request = self.compaction_request();
            let compacted = compactor.compact(&request);
            let mut compaction_records = self.session.compaction_records().to_vec();
            compaction_records.push(compacted.record.clone());
            self.session = Session::from_parts(
                compacted.compacted_context.clone(),
                SessionQueue::new(),
                compaction_records,
            );
            return Some(RetryOutcome::from_compaction(compacted));
        }

        Some(RetryOutcome {
            retried_context: self.session.history().to_vec(),
            compaction: None,
        })
    }

    pub fn compaction_request(&self) -> CompactionRequest {
        crate::compaction::prepare_compaction_request(&self.active_context())
    }
}
