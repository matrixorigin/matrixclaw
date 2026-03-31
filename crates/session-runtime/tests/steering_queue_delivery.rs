use matrixclaw_session_runtime::queue::QueueItem;
use matrixclaw_session_runtime::run_controller::RunController;
use matrixclaw_session_runtime::RuntimeMessage;

#[test]
fn steering_queue_delivery() {
    let mut controller = RunController::new(vec![
        RuntimeMessage::ToolResult("result:alpha".to_string()),
        RuntimeMessage::ToolResult("result:beta".to_string()),
    ]);

    controller.queue_steering_message("hold the line");

    assert_eq!(
        controller.queue().items(),
        &[QueueItem::Steering("hold the line".to_string())],
        "the steering message should be stored in the session runtime queue"
    );

    let projected_turn = controller.project_next_turn();
    assert_eq!(
        projected_turn,
        vec![
            RuntimeMessage::ToolResult("result:alpha".to_string()),
            RuntimeMessage::ToolResult("result:beta".to_string()),
            RuntimeMessage::Steering("hold the line".to_string()),
            RuntimeMessage::Assistant("next assistant turn".to_string()),
        ],
        "steering must be delivered before the next assistant turn without disturbing tool-result ordering"
    );
}
