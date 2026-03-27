use matrixclaw_agent_core::message::ToolResultMessage;
use matrixclaw_session_runtime::queue::QueueItem;
use matrixclaw_session_runtime::run_controller::RunController;
use matrixclaw_session_runtime::RuntimeMessage;

#[test]
fn follow_up_queue_delivery() {
    let mut controller = RunController::new(vec![
        RuntimeMessage::ToolResult(
            ToolResultMessage::new("search", "alpha").as_assistant_fragment(),
        ),
        RuntimeMessage::ToolResult(
            ToolResultMessage::new("search", "beta").as_assistant_fragment(),
        ),
    ]);

    controller.queue_follow_up_message("what changed?");

    assert_eq!(
        controller.queue().items(),
        &[QueueItem::FollowUp("what changed?".to_string())],
        "the follow-up message should be stored separately from steering messages"
    );

    assert_eq!(
        controller.project_next_turn(),
        vec![
            RuntimeMessage::ToolResult("result:alpha".to_string()),
            RuntimeMessage::ToolResult("result:beta".to_string()),
            RuntimeMessage::Assistant("next assistant turn".to_string()),
        ],
        "follow-up messages must not splice into the active turn"
    );

    let next_run = controller.complete_current_run();
    assert_eq!(
        next_run,
        vec![
            RuntimeMessage::ToolResult("result:alpha".to_string()),
            RuntimeMessage::ToolResult("result:beta".to_string()),
            RuntimeMessage::FollowUp("what changed?".to_string()),
            RuntimeMessage::Assistant("next run assistant turn".to_string()),
        ],
        "follow-up messages should be delivered only after the current run completes"
    );

    assert!(
        controller.queue().items().is_empty(),
        "a delivered follow-up should no longer remain pending in the queue"
    );
}
