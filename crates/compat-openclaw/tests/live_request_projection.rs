use matrixclaw_compat_openclaw::translation::{
    split_openclaw_request, OpenClawChatMessage, OpenClawChatRequest, OpenClawChatRole,
};
use matrixclaw_session_runtime::RuntimeMessage;

#[test]
fn split_openclaw_request_projects_history_for_live_runtime() {
    let request = OpenClawChatRequest::new(
        "conversation-1",
        vec![
            OpenClawChatMessage::system("stay protocol-shaped"),
            OpenClawChatMessage {
                role: OpenClawChatRole::Assistant,
                content: "prior answer".to_string(),
            },
            OpenClawChatMessage {
                role: OpenClawChatRole::Tool,
                content: "result:42".to_string(),
            },
            OpenClawChatMessage::user("resume the run"),
        ],
    );

    let (history, prompt) = split_openclaw_request(&request).expect("project live request");

    assert_eq!(
        history,
        vec![
            RuntimeMessage::RuntimeSummary("stay protocol-shaped".to_string()),
            RuntimeMessage::Assistant("prior answer".to_string()),
            RuntimeMessage::ToolResult("result:42".to_string()),
        ]
    );
    assert_eq!(prompt, "resume the run");
}

#[test]
fn split_openclaw_request_requires_final_user_message() {
    let request = OpenClawChatRequest::new(
        "conversation-2",
        vec![OpenClawChatMessage::system("missing user turn")],
    );

    let error = split_openclaw_request(&request).expect_err("reject non-user tail");
    assert!(
        error.contains("end with a user message"),
        "unexpected projection error: {error}"
    );
}
