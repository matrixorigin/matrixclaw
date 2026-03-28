use matrixclaw_app_host::ingress::{normalize_openclaw_request, OpenClawIngressMetadata};
use matrixclaw_compat_openclaw::translation::{OpenClawChatMessage, OpenClawChatRequest};
use matrixclaw_session_runtime::RuntimeMessage;

#[test]
fn normalized_ingress_envelope() {
    let request = OpenClawChatRequest::new(
        "shared-session".to_string(),
        vec![
            OpenClawChatMessage::system("keep transport metadata out of the runtime"),
            OpenClawChatMessage::user("summarize the status"),
        ],
    );
    let metadata = OpenClawIngressMetadata {
        sender_id: Some("matrix-user-7".to_string()),
        sender_display_name: Some("Matrix User".to_string()),
        channel_id: Some("!room:example.org".to_string()),
        thread_id: Some("$thread".to_string()),
        reply_to: Some("$event".to_string()),
        target_agent: Some("planner".to_string()),
    };

    let envelope =
        normalize_openclaw_request(&request, &metadata).expect("normalize OpenClaw request");

    assert_eq!(envelope.transport.kind, "openclaw");
    assert_eq!(envelope.transport.route, "/api/openclaw/chat");
    assert_eq!(envelope.sender.id.as_deref(), Some("matrix-user-7"));
    assert_eq!(envelope.sender.display_name.as_deref(), Some("Matrix User"));
    assert_eq!(envelope.conversation.session_id, "shared-session");
    assert_eq!(envelope.conversation.thread_id.as_deref(), Some("$thread"));
    assert_eq!(envelope.target_agent.as_deref(), Some("planner"));
    assert_eq!(envelope.payload.prompt, "summarize the status");
    assert_eq!(
        envelope.payload.seed_history,
        vec![RuntimeMessage::RuntimeSummary(
            "keep transport metadata out of the runtime".to_string()
        )]
    );
    assert_eq!(envelope.reply.conversation_id, "shared-session");
    assert_eq!(
        envelope.reply.channel_id.as_deref(),
        Some("!room:example.org")
    );
    assert_eq!(envelope.reply.thread_id.as_deref(), Some("$thread"));
    assert_eq!(envelope.reply.reply_to.as_deref(), Some("$event"));

    let live_request = envelope.to_live_run_request();
    assert_eq!(live_request.session_id.as_deref(), Some("shared-session"));
    assert_eq!(live_request.prompt, "summarize the status");
}
