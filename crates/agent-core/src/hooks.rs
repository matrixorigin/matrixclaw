use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HookPoint {
    PreLlmCall,
    PostLlmCall,
    PreToolCall,
    PostToolCall,
    OnSessionStart,
    OnSessionEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookPayload {
    pub hook_point: HookPoint,
    pub session_id: Option<String>,
    pub tool_name: Option<String>,
    pub tool_arguments: Option<serde_json::Value>,
    pub tool_result: Option<String>,
    pub llm_response: Option<String>,
    pub iteration: u32,
}

impl HookPayload {
    pub fn pre_llm_call(session_id: Option<&str>, iteration: u32) -> Self {
        Self {
            hook_point: HookPoint::PreLlmCall,
            session_id: session_id.map(|s| s.to_string()),
            tool_name: None,
            tool_arguments: None,
            tool_result: None,
            llm_response: None,
            iteration,
        }
    }

    pub fn post_llm_call(session_id: Option<&str>, iteration: u32, response: &str) -> Self {
        Self {
            hook_point: HookPoint::PostLlmCall,
            session_id: session_id.map(|s| s.to_string()),
            tool_name: None,
            tool_arguments: None,
            tool_result: None,
            llm_response: Some(response.to_string()),
            iteration,
        }
    }

    pub fn pre_tool_call(
        session_id: Option<&str>,
        iteration: u32,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> Self {
        Self {
            hook_point: HookPoint::PreToolCall,
            session_id: session_id.map(|s| s.to_string()),
            tool_name: Some(tool_name.to_string()),
            tool_arguments: Some(arguments.clone()),
            tool_result: None,
            llm_response: None,
            iteration,
        }
    }

    pub fn post_tool_call(
        session_id: Option<&str>,
        iteration: u32,
        tool_name: &str,
        result: &str,
    ) -> Self {
        Self {
            hook_point: HookPoint::PostToolCall,
            session_id: session_id.map(|s| s.to_string()),
            tool_name: Some(tool_name.to_string()),
            tool_arguments: None,
            tool_result: Some(result.to_string()),
            llm_response: None,
            iteration,
        }
    }

    pub fn on_session_start(session_id: &str) -> Self {
        Self {
            hook_point: HookPoint::OnSessionStart,
            session_id: Some(session_id.to_string()),
            tool_name: None,
            tool_arguments: None,
            tool_result: None,
            llm_response: None,
            iteration: 0,
        }
    }

    pub fn on_session_end(session_id: &str) -> Self {
        Self {
            hook_point: HookPoint::OnSessionEnd,
            session_id: Some(session_id.to_string()),
            tool_name: None,
            tool_arguments: None,
            tool_result: None,
            llm_response: None,
            iteration: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookAction {
    pub block: bool,
    pub reason: Option<String>,
}

impl Default for HookAction {
    fn default() -> Self {
        Self {
            block: false,
            reason: None,
        }
    }
}

impl HookAction {
    pub fn allow() -> Self {
        Self::default()
    }

    pub fn block(reason: impl Into<String>) -> Self {
        Self {
            block: true,
            reason: Some(reason.into()),
        }
    }
}

#[async_trait]
pub trait LifecycleHook: Send + Sync {
    async fn on_event(&self, payload: &HookPayload) -> HookAction;
    fn name(&self) -> &str;
}

pub struct CompositeHook {
    hooks: Vec<Box<dyn LifecycleHook>>,
}

impl CompositeHook {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    pub fn add(&mut self, hook: Box<dyn LifecycleHook>) {
        self.hooks.push(hook);
    }

    pub async fn dispatch(&self, payload: &HookPayload) -> HookAction {
        let mut action = HookAction::allow();
        for hook in &self.hooks {
            let result = hook.on_event(payload).await;
            if result.block {
                action = result;
                break;
            }
        }
        action
    }

    pub fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }
}

impl Default for CompositeHook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AllowHook;
    struct BlockHook;

    #[async_trait]
    impl LifecycleHook for AllowHook {
        async fn on_event(&self, _payload: &HookPayload) -> HookAction {
            HookAction::allow()
        }
        fn name(&self) -> &str {
            "allow"
        }
    }

    #[async_trait]
    impl LifecycleHook for BlockHook {
        async fn on_event(&self, _payload: &HookPayload) -> HookAction {
            HookAction::block("blocked by test hook")
        }
        fn name(&self) -> &str {
            "block"
        }
    }

    #[tokio::test]
    async fn composite_hook_allows_when_no_hooks_block() {
        let mut composite = CompositeHook::new();
        composite.add(Box::new(AllowHook));
        let payload = HookPayload::pre_llm_call(None, 1);
        let action = composite.dispatch(&payload).await;
        assert!(!action.block);
    }

    #[tokio::test]
    async fn composite_hook_blocks_when_any_hook_blocks() {
        let mut composite = CompositeHook::new();
        composite.add(Box::new(AllowHook));
        composite.add(Box::new(BlockHook));
        let payload = HookPayload::pre_llm_call(None, 1);
        let action = composite.dispatch(&payload).await;
        assert!(action.block);
        assert_eq!(action.reason, Some("blocked by test hook".to_string()));
    }

    #[tokio::test]
    async fn composite_hook_stops_at_first_block() {
        let mut composite = CompositeHook::new();
        composite.add(Box::new(BlockHook));
        composite.add(Box::new(AllowHook));
        let payload = HookPayload::pre_tool_call(None, 1, "terminal", &serde_json::json!({}));
        let action = composite.dispatch(&payload).await;
        assert!(action.block);
    }

    #[tokio::test]
    async fn empty_composite_hook_allows() {
        let composite = CompositeHook::new();
        let payload = HookPayload::on_session_start("test-session");
        let action = composite.dispatch(&payload).await;
        assert!(!action.block);
    }

    #[test]
    fn hook_payload_serialization_roundtrip() {
        let payload = HookPayload::pre_tool_call(
            Some("session-1"),
            5,
            "terminal",
            &serde_json::json!({"command": "ls"}),
        );
        let json = serde_json::to_string(&payload).unwrap();
        let deserialized: HookPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.hook_point, HookPoint::PreToolCall);
        assert_eq!(deserialized.tool_name, Some("terminal".to_string()));
        assert_eq!(deserialized.session_id, Some("session-1".to_string()));
        assert_eq!(deserialized.iteration, 5);
    }

    #[test]
    fn hook_action_allow_default() {
        let action = HookAction::allow();
        assert!(!action.block);
        assert!(action.reason.is_none());
    }

    #[test]
    fn hook_action_block_with_reason() {
        let action = HookAction::block("dangerous");
        assert!(action.block);
        assert_eq!(action.reason, Some("dangerous".to_string()));
    }

    #[test]
    fn all_hook_points_serialize_correctly() {
        let points = vec![
            HookPoint::PreLlmCall,
            HookPoint::PostLlmCall,
            HookPoint::PreToolCall,
            HookPoint::PostToolCall,
            HookPoint::OnSessionStart,
            HookPoint::OnSessionEnd,
        ];
        for point in points {
            let json = serde_json::to_string(&point).unwrap();
            let deserialized: HookPoint = serde_json::from_str(&json).unwrap();
            assert_eq!(point, deserialized);
        }
    }
}
