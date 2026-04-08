use zstar_app_host::gateway::{
    GatewayAdapter, GatewayInboundEvent, GatewayOutboundDelivery, GatewayReplyRoute, GatewaySender,
    GatewayThread, OutboundDeliveryKind,
};
use zstar_app_host::ingress::IngressEnvelope;
use zstar_app_host::live_runtime::LiveRunEvent;

#[test]
fn gateway_adapter_contract() {
    struct FakeGateway;

    impl GatewayAdapter for FakeGateway {
        fn kind(&self) -> &'static str {
            "fake-gateway"
        }

        fn normalize_inbound(
            &self,
            event: &GatewayInboundEvent,
        ) -> Result<IngressEnvelope, String> {
            Ok(event.to_ingress_envelope(self.kind()))
        }

        fn project_runtime_event(
            &self,
            route: &GatewayReplyRoute,
            event: &LiveRunEvent,
        ) -> Vec<GatewayOutboundDelivery> {
            match event.kind.as_str() {
                "message_delta" => vec![GatewayOutboundDelivery {
                    kind: OutboundDeliveryKind::AssistantChunk,
                    channel_id: route.channel_id.clone(),
                    thread: route.thread.clone(),
                    reply_to: route.reply_to.clone(),
                    body: event.content.clone().unwrap_or_default(),
                }],
                _ => Vec::new(),
            }
        }
    }

    let inbound = GatewayInboundEvent {
        sender: GatewaySender {
            id: "user-7".to_string(),
            display_name: Some("Matrix User".to_string()),
        },
        channel_id: "!room:example.org".to_string(),
        thread: Some(GatewayThread {
            session_id: "session-1".to_string(),
            thread_id: Some("$thread".to_string()),
        }),
        target_agent: Some("planner".to_string()),
        prompt: "summarize the project status".to_string(),
        reply_to: Some("$event".to_string()),
    };

    let adapter = FakeGateway;
    let envelope = adapter
        .normalize_inbound(&inbound)
        .expect("normalize gateway inbound event");

    assert_eq!(adapter.kind(), "fake-gateway");
    assert_eq!(envelope.transport.kind, "fake-gateway");
    assert_eq!(envelope.sender.id.as_deref(), Some("user-7"));
    assert_eq!(envelope.sender.display_name.as_deref(), Some("Matrix User"));
    assert_eq!(envelope.conversation.session_id, "session-1");
    assert_eq!(envelope.conversation.thread_id.as_deref(), Some("$thread"));
    assert_eq!(envelope.target_agent.as_deref(), Some("planner"));
    assert_eq!(envelope.payload.prompt, "summarize the project status");
    assert_eq!(
        envelope.reply.channel_id.as_deref(),
        Some("!room:example.org")
    );
    assert_eq!(envelope.reply.thread_id.as_deref(), Some("$thread"));
    assert_eq!(envelope.reply.reply_to.as_deref(), Some("$event"));

    let deliveries = adapter.project_runtime_event(
        &GatewayReplyRoute {
            channel_id: "!room:example.org".to_string(),
            thread: Some(GatewayThread {
                session_id: "session-1".to_string(),
                thread_id: Some("$thread".to_string()),
            }),
            reply_to: Some("$event".to_string()),
        },
        &LiveRunEvent {
            sequence: 1,
            kind: "message_delta".to_string(),
            content: Some("partial answer".to_string()),
        },
    );

    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].kind, OutboundDeliveryKind::AssistantChunk);
    assert_eq!(deliveries[0].channel_id, "!room:example.org");
    assert_eq!(
        deliveries[0]
            .thread
            .as_ref()
            .and_then(|thread| thread.thread_id.as_deref()),
        Some("$thread")
    );
    assert_eq!(deliveries[0].reply_to.as_deref(), Some("$event"));
    assert_eq!(deliveries[0].body, "partial answer");
}
