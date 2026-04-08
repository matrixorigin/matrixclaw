use zstar_app_host::http::queue_api::{
    queue_controls_contract, queue_controls_view, submit_queue_control, QueueControlKind,
    QueueDeliveryTiming, QueueSubmissionRequest,
};
use zstar_session_runtime::queue::SessionQueue;

#[test]
fn queued_controls_ui() {
    let contract = queue_controls_contract();
    assert_ne!(
        contract.steering_submit_route, contract.follow_up_submit_route,
        "steering and follow-up must submit through distinct backend routes"
    );
    assert_eq!(
        contract.state_route, "/api/queue/state",
        "queue state should be exposed through a dedicated route"
    );

    let mut queue = SessionQueue::new();
    queue.push_steering("hold the line");
    queue.push_follow_up("answer in the next run");

    let view = queue_controls_view(&queue);
    assert_eq!(
        view.steering.kind,
        QueueControlKind::Steering,
        "steering control state should be separated from follow-up state"
    );
    assert_eq!(
        view.follow_up.kind,
        QueueControlKind::FollowUp,
        "follow-up control state should be separated from steering state"
    );
    assert_eq!(
        view.steering.delivery_timing,
        QueueDeliveryTiming::NextTurn,
        "steering should be rendered as immediate-next-turn delivery"
    );
    assert_eq!(
        view.follow_up.delivery_timing,
        QueueDeliveryTiming::NextRun,
        "follow-up should be rendered as deferred-until-current-run-completes"
    );
    assert!(
        view.steering.summary.contains("steering"),
        "steering summary should describe steering controls"
    );
    assert!(
        view.follow_up.summary.contains("follow-up"),
        "follow-up summary should describe follow-up controls"
    );

    let steering_submission = submit_queue_control(
        &mut queue,
        QueueSubmissionRequest {
            kind: QueueControlKind::Steering,
            message: "tighten the answer".to_string(),
            session_id: Some("session-test".to_string()),
        },
    );
    assert!(
        steering_submission.accepted,
        "queue submissions should return an accepted result"
    );
    assert_eq!(
        steering_submission.session_id, "session-test",
        "queue submissions should echo the session id they apply to"
    );
    assert_eq!(
        steering_submission.state.delivery_timing,
        QueueDeliveryTiming::NextTurn,
        "steering submissions should preserve next-turn semantics"
    );
}
