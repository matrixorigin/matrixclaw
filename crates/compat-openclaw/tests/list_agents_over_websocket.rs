use matrixclaw_compat_openclaw::websocket::{openclaw_agents_list, Frame};

#[test]
fn list_agents_over_websocket() {
    let conversation = openclaw_agents_list(true);

    assert_eq!(conversation.capability.protocol, "openclaw");
    assert_eq!(conversation.capability.version, "0.1");
    assert!(conversation.capability.agent_listing_supported);

    let expected = vec![
        Frame::Challenge {
            token: "challenge-token".to_string(),
        },
        Frame::Authenticated,
        Frame::AgentsList {
            agents: vec![matrixclaw_compat_openclaw::capabilities::AgentDescriptor {
                id: "default".to_string(),
                name: "default".to_string(),
            }],
        },
    ];

    assert_eq!(
        conversation.frames, expected,
        "expected the OpenClaw boundary to authenticate and list agents"
    );
}
