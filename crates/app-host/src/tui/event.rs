use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent};
use futures_util::StreamExt;

use crate::live_runtime::LiveRunEvent;

pub enum AppEvent {
    Key(KeyEvent),
    Agent(LiveRunEvent),
    Quit,
}

pub struct EventReader {
    events: EventStream,
}

impl Default for EventReader {
    fn default() -> Self {
        Self::new()
    }
}

impl EventReader {
    pub fn new() -> Self {
        Self {
            events: EventStream::new(),
        }
    }

    pub async fn next(
        &mut self,
        agent_rx: &mut tokio::sync::mpsc::Receiver<LiveRunEvent>,
    ) -> AppEvent {
        tokio::select! {
            maybe = agent_rx.recv() => {
                match maybe {
                    Some(event) => AppEvent::Agent(event),
                    None => AppEvent::Quit,
                }
            }
            maybe = self.events.next() => {
                match maybe {
                    Some(Ok(CrosstermEvent::Key(key))) => AppEvent::Key(key),
                    _ => AppEvent::Quit,
                }
            }
        }
    }
}
