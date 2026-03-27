use serde::{Deserialize, Serialize};

use matrixclaw_session_runtime::queue::SessionQueue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QueueControlKind {
    Steering,
    FollowUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QueueDeliveryTiming {
    NextTurn,
    NextRun,
    Queued,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueControlState {
    pub kind: QueueControlKind,
    pub submit_route: String,
    pub delivery_timing: QueueDeliveryTiming,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueControlsView {
    pub steering: QueueControlState,
    pub follow_up: QueueControlState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueControlsContract {
    pub steering_submit_route: String,
    pub follow_up_submit_route: String,
    pub state_route: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueSubmissionRequest {
    pub kind: QueueControlKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueSubmissionResult {
    pub accepted: bool,
    pub state: QueueControlState,
}

pub fn queue_controls_contract() -> QueueControlsContract {
    QueueControlsContract {
        steering_submit_route: "/api/queue/steering".to_string(),
        follow_up_submit_route: "/api/queue/follow-up".to_string(),
        state_route: "/api/queue/state".to_string(),
    }
}

pub fn queue_controls_view(queue: &SessionQueue) -> QueueControlsView {
    let contract = queue_controls_contract();
    let steering_count = queue.steering_items().count();
    let follow_up_count = queue.follow_up_items().count();

    QueueControlsView {
        steering: QueueControlState {
            kind: QueueControlKind::Steering,
            submit_route: contract.steering_submit_route,
            delivery_timing: QueueDeliveryTiming::NextTurn,
            summary: format!(
                "{steering_count} steering item(s) queued for the next assistant turn"
            ),
        },
        follow_up: QueueControlState {
            kind: QueueControlKind::FollowUp,
            submit_route: contract.follow_up_submit_route,
            delivery_timing: QueueDeliveryTiming::NextRun,
            summary: format!(
                "{follow_up_count} follow-up item(s) deferred until the current run completes"
            ),
        },
    }
}

pub fn submit_queue_control(
    queue: &mut SessionQueue,
    request: QueueSubmissionRequest,
) -> QueueSubmissionResult {
    match request.kind {
        QueueControlKind::Steering => queue.push_steering(request.message),
        QueueControlKind::FollowUp => queue.push_follow_up(request.message),
    }

    let view = queue_controls_view(queue);
    let state = match request.kind {
        QueueControlKind::Steering => view.steering,
        QueueControlKind::FollowUp => view.follow_up,
    };

    QueueSubmissionResult {
        accepted: true,
        state,
    }
}
