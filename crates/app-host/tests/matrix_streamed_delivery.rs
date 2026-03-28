use matrixclaw_app_host::gateway::matrix::{
    project_matrix_streamed_delivery, MatrixOutboundRoute, MatrixReplyRoute,
};
use matrixclaw_app_host::gateway::{GatewayThread, OutboundDeliveryKind};
use matrixclaw_app_host::live_runtime::LiveRunEvent;

#[test]
fn matrix_streamed_delivery() {
    let route = MatrixReplyRoute {
        room_id: "!room:example.org".to_string(),
        thread: Some(GatewayThread {
            session_id: "session-7".to_string(),
            thread_id: Some("$thread".to_string()),
        }),
        reply_to: Some("$event".to_string()),
    };

    let events = vec![
        LiveRunEvent {
            sequence: 0,
            kind: "message_delta".to_string(),
            content: Some("first ".to_string()),
        },
        LiveRunEvent {
            sequence: 1,
            kind: "message_delta".to_string(),
            content: Some("second".to_string()),
        },
        LiveRunEvent {
            sequence: 2,
            kind: "run_completed".to_string(),
            content: None,
        },
    ];

    let deliveries = project_matrix_streamed_delivery(
        MatrixOutboundRoute {
            room_id: route.room_id.clone(),
            thread: route.thread.clone(),
            reply_to: route.reply_to.clone(),
        },
        &events,
    );

    assert_eq!(
        deliveries
            .iter()
            .map(|delivery| delivery.kind.clone())
            .collect::<Vec<_>>(),
        vec![
            OutboundDeliveryKind::AssistantChunk,
            OutboundDeliveryKind::AssistantChunk,
            OutboundDeliveryKind::AssistantFinal,
        ]
    );
    assert_eq!(deliveries[0].body, "first ");
    assert_eq!(deliveries[1].body, "second");
    assert_eq!(deliveries[2].body, "first second");
}
