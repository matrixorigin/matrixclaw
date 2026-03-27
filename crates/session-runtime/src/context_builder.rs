use crate::message_projection;
use crate::queue::SessionQueue;
use crate::session::Session;
use crate::RuntimeMessage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuationContext {
    pub next_turn: Vec<RuntimeMessage>,
    pub next_run: Vec<RuntimeMessage>,
}

pub struct ContextBuilder;

impl ContextBuilder {
    pub fn build_from_parts(history: &[RuntimeMessage], queue: &SessionQueue) -> ContinuationContext {
        ContinuationContext {
            next_turn: message_projection::project_next_turn(history, queue),
            next_run: message_projection::project_next_run(history, queue),
        }
    }

    pub fn build(session: &Session) -> ContinuationContext {
        Self::build_from_parts(session.history(), session.queue())
    }
}
