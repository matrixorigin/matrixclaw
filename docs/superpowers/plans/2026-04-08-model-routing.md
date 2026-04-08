# Model Routing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add config-driven model routing that selects the best provider+model for each task based on prompt characteristics, skill context, and token estimates — without changing the existing `Provider` trait or `FallbackProvider`.

**Architecture:** A `ModelRouter` sits in `provider-plane` as a pure function of `(RunRequest) -> RoutingDecision`. It reads routing rules from a config file. The caller (`chat.rs`) uses the routing decision to select the appropriate provider+model before invoking the agent loop. No hooks needed — routing happens at the call site, not inside the loop.

**Tech Stack:** `serde_json` (config), existing `ProviderPlaneConfig`, `RunRequest` inspection.

---

## Key Design Decisions

### Why not a LifecycleHook?
Hooks can't mutate the `RunRequest` or swap the provider. Routing must happen *before* the provider is called, at the call site in `chat.rs`. This is simpler and more explicit.

### Why not modify FallbackProvider?
`FallbackProvider` already does provider selection within a chain. Model routing is a different concern: *which chain/model to use for this task*. Adding it to FallbackProvider would conflate two responsibilities.

### Where routing lives
`provider-plane/src/router.rs` — a new module. The `ModelRouter` is a pure config reader + matcher. No state, no async, no side effects.

---

## File Structure

| File | Responsibility |
|------|---------------|
| `crates/provider-plane/src/router.rs` | `ModelRouter`, `RoutingRule`, `RoutingDecision`, `RouteMatcher` |
| `crates/provider-plane/src/backend.rs` | Modified: add routing section to `ProviderPlaneConfig` |
| `crates/provider-plane/src/lib.rs` | Modified: export router module |
| `crates/app-host/src/chat.rs` | Modified: use `ModelRouter` before provider call |

---

## Task 1: Routing Types and Config

**Files:**
- Create: `crates/provider-plane/src/router.rs`
- Modify: `crates/provider-plane/src/backend.rs`
- Modify: `crates/provider-plane/src/lib.rs`

- [ ] **Step 1: Write the failing tests and implementation**

In `crates/provider-plane/src/router.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteMatcher {
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub max_prompt_chars: Option<usize>,
    #[serde(default)]
    pub tool_count_min: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub name: String,
    #[serde(default)]
    pub match_: RouteMatcher,
    pub provider: String,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub rule_name: String,
    pub provider: String,
    pub model: Option<String>,
}

pub struct ModelRouter {
    rules: Vec<RoutingRule>,
    default_provider: String,
    default_model: Option<String>,
}

impl ModelRouter {
    pub fn new(rules: Vec<RoutingRule>, default_provider: String, default_model: Option<String>) -> Self {
        Self { rules, default_provider, default_model }
    }

    pub fn empty(default_provider: String, default_model: Option<String>) -> Self {
        Self::new(vec![], default_provider, default_model)
    }

    pub fn route(&self, prompt: &str, tool_count: usize, skill_hints: &[String]) -> RoutingDecision {
        for rule in &self.rules {
            if self.matches(&rule.match_, prompt, tool_count, skill_hints) {
                return RoutingDecision {
                    rule_name: rule.name.clone(),
                    provider: rule.provider.clone(),
                    model: rule.model.clone(),
                };
            }
        }
        RoutingDecision {
            rule_name: "default".to_string(),
            provider: self.default_provider.clone(),
            model: self.default_model.clone(),
        }
    }

    fn matches(&self, matcher: &RouteMatcher, prompt: &str, tool_count: usize, skill_hints: &[String]) -> bool {
        if !matcher.skills.is_empty() {
            let has_skill = skill_hints.iter().any(|s| {
                matcher.skills.iter().any(|ms| s.to_lowercase().contains(&ms.to_lowercase()))
            });
            if !has_skill {
                return false;
            }
        }

        if !matcher.keywords.is_empty() {
            let prompt_lower = prompt.to_lowercase();
            let has_keyword = matcher.keywords.iter().any(|kw| prompt_lower.contains(&kw.to_lowercase()));
            if !has_keyword {
                return false;
            }
        }

        if let Some(max_chars) = matcher.max_prompt_chars {
            if prompt.len() > max_chars {
                return false;
            }
        }

        if let Some(min_tools) = matcher.tool_count_min {
            if tool_count < min_tools {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_router(rules: Vec<RoutingRule>) -> ModelRouter {
        ModelRouter::new(rules, "default".to_string(), Some("default-model".to_string()))
    }

    #[test]
    fn empty_rules_returns_default() {
        let router = make_router(vec![]);
        let decision = router.route("hello", 0, &[]);
        assert_eq!(decision.rule_name, "default");
        assert_eq!(decision.provider, "default");
    }

    #[test]
    fn matches_by_skill() {
        let rules = vec![RoutingRule {
            name: "code".to_string(),
            match_: RouteMatcher {
                skills: vec!["deploy".to_string()],
                keywords: vec![],
                max_prompt_chars: None,
                tool_count_min: None,
            },
            provider: "openrouter".to_string(),
            model: Some("claude-sonnet-4".to_string()),
        }];
        let router = make_router(rules);
        let decision = router.route("do something", 0, &["deploy-skill".to_string()]);
        assert_eq!(decision.rule_name, "code");
        assert_eq!(decision.provider, "openrouter");
        assert_eq!(decision.model.as_deref(), Some("claude-sonnet-4"));
    }

    #[test]
    fn matches_by_keyword() {
        let rules = vec![RoutingRule {
            name: "fast".to_string(),
            match_: RouteMatcher {
                skills: vec![],
                keywords: vec!["quick".to_string(), "simple".to_string()],
                max_prompt_chars: Some(500),
                tool_count_min: None,
            },
            provider: "local".to_string(),
            model: Some("llama3".to_string()),
        }];
        let router = make_router(rules);
        let decision = router.route("quick question", 0, &[]);
        assert_eq!(decision.provider, "local");
    }

    #[test]
    fn no_match_falls_to_default() {
        let rules = vec![RoutingRule {
            name: "code".to_string(),
            match_: RouteMatcher {
                skills: vec!["deploy".to_string()],
                keywords: vec![],
                max_prompt_chars: None,
                tool_count_min: None,
            },
            provider: "openrouter".to_string(),
            model: None,
        }];
        let router = make_router(rules);
        let decision = router.route("hello", 0, &[]);
        assert_eq!(decision.rule_name, "default");
    }

    #[test]
    fn matches_by_tool_count() {
        let rules = vec![RoutingRule {
            name: "heavy".to_string(),
            match_: RouteMatcher {
                skills: vec![],
                keywords: vec![],
                max_prompt_chars: None,
                tool_count_min: Some(5),
            },
            provider: "big-model".to_string(),
            model: None,
        }];
        let router = make_router(rules);
        let decision = router.route("complex task", 10, &[]);
        assert_eq!(decision.rule_name, "heavy");
        let no_match = router.route("simple", 2, &[]);
        assert_eq!(no_match.rule_name, "default");
    }

    #[test]
    fn first_matching_rule_wins() {
        let rules = vec![
            RoutingRule {
                name: "first".to_string(),
                match_: RouteMatcher {
                    skills: vec![],
                    keywords: vec!["test".to_string()],
                    max_prompt_chars: None,
                    tool_count_min: None,
                },
                provider: "a".to_string(),
                model: None,
            },
            RoutingRule {
                name: "second".to_string(),
                match_: RouteMatcher {
                    skills: vec![],
                    keywords: vec!["test".to_string()],
                    max_prompt_chars: None,
                    tool_count_min: None,
                },
                provider: "b".to_string(),
                model: None,
            },
        ];
        let router = make_router(rules);
        let decision = router.route("test something", 0, &[]);
        assert_eq!(decision.provider, "a");
    }

    #[test]
    fn matches_by_max_prompt_chars() {
        let rules = vec![RoutingRule {
            name: "short".to_string(),
            match_: RouteMatcher {
                skills: vec![],
                keywords: vec![],
                max_prompt_chars: Some(100),
                tool_count_min: None,
            },
            provider: "fast".to_string(),
            model: None,
        }];
        let router = make_router(rules);
        let short = router.route("hi", 0, &[]);
        assert_eq!(short.provider, "fast");
        let long = router.route(&"x".repeat(200), 0, &[]);
        assert_eq!(long.provider, "default");
    }

    #[test]
    fn config_deserialization() {
        let json = r#"{
            "name": "code",
            "match": {
                "skills": ["deploy", "debug"],
                "keywords": ["code", "implement"],
                "max_prompt_chars": 10000
            },
            "provider": "openrouter",
            "model": "anthropic/claude-sonnet-4"
        }"#;
        let rule: RoutingRule = serde_json::from_str(json).unwrap();
        assert_eq!(rule.name, "code");
        assert_eq!(rule.match_.skills.len(), 2);
        assert_eq!(rule.match_.keywords.len(), 2);
        assert_eq!(rule.match_.max_prompt_chars, Some(10000));
        assert_eq!(rule.provider, "openrouter");
        assert_eq!(rule.model.as_deref(), Some("anthropic/claude-sonnet-4"));
    }
}
```

In `crates/provider-plane/src/lib.rs`, add:
```rust
pub mod router;
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test -p matrixclaw-provider router`
Expected: 8 tests PASS

- [ ] **Step 3: Commit**

```bash
git add crates/provider-plane/src/router.rs crates/provider-plane/src/lib.rs
git commit -m "feat(provider): add ModelRouter with config-driven routing rules"
```

---

## Task 2: Add Routing to ProviderPlaneConfig

**Files:**
- Modify: `crates/provider-plane/src/backend.rs`
- Modify: `crates/provider-plane/src/config.rs`

- [ ] **Step 1: Add routing fields to ProviderPlaneConfig**

In `crates/provider-plane/src/backend.rs`, add to `ProviderPlaneConfig`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPlaneConfig {
    pub providers: Vec<ProviderConfig>,
    #[serde(default)]
    pub fallback_chain: Vec<String>,
    #[serde(default)]
    pub routes: Vec<crate::router::RoutingRule>,
    #[serde(default)]
    pub default_provider: Option<String>,
}
```

Add a test that parses a full config with routes:

```rust
#[test]
fn parses_config_with_routes() {
    let plane: ProviderPlaneConfig = serde_json::from_str(
        r#"{
            "providers": [
                {"name": "openrouter", "type": "open_ai", "api_key": "sk-1"},
                {"name": "local", "type": "ollama"}
            ],
            "fallback_chain": ["openrouter", "local"],
            "default_provider": "openrouter",
            "routes": [
                {
                    "name": "code",
                    "match": {"skills": ["deploy"], "keywords": []},
                    "provider": "openrouter",
                    "model": "anthropic/claude-sonnet-4"
                },
                {
                    "name": "fast",
                    "match": {"max_prompt_chars": 500},
                    "provider": "local"
                }
            ]
        }"#,
    )
    .unwrap();
    assert_eq!(plane.routes.len(), 2);
    assert_eq!(plane.routes[0].name, "code");
    assert_eq!(plane.routes[1].name, "fast");
    assert_eq!(plane.default_provider.as_deref(), Some("openrouter"));
}
```

- [ ] **Step 2: Add helper to build ModelRouter from config**

In `crates/provider-plane/src/config.rs`, add:

```rust
use crate::router::ModelRouter;

impl ProviderPlaneConfig {
    pub fn build_router(&self, fallback_model: Option<String>) -> ModelRouter {
        let default_provider = self.default_provider.clone()
            .or_else(|| self.fallback_chain.first().cloned())
            .unwrap_or_default();
        ModelRouter::new(
            self.routes.clone(),
            default_provider,
            fallback_model,
        )
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p matrixclaw-provider`
Expected: All existing + new tests PASS

- [ ] **Step 4: Commit**

```bash
git add crates/provider-plane/src/backend.rs crates/provider-plane/src/config.rs
git commit -m "feat(provider): add routing rules to ProviderPlaneConfig"
```

---

## Task 3: Wire ModelRouter into Chat

**Files:**
- Modify: `crates/app-host/src/chat.rs`

- [ ] **Step 1: Use ModelRouter in the chat loop**

In `chat.rs`, after building the provider parts and before the chat loop:

1. Import `ModelRouter`:
```rust
use matrixclaw_provider::router::ModelRouter;
```

2. Build the router from the plane config:
```rust
let router = plane_config.build_router(Some(model.clone()));
```

3. In the chat loop, before calling `service.run_with_provider_and_queue_stream`, apply routing:
```rust
let skill_hints: Vec<String> = Vec::new();
let tool_count_for_routing = service.tool_count().await;
let decision = router.route(input, tool_count_for_routing, &skill_hints);
let effective_model = decision.model.as_deref().unwrap_or(&model);
```

Use `effective_model` in the `run_with_provider_and_queue_stream` call instead of `&model`.

Print the routing decision when it's not default:
```rust
if decision.rule_name != "default" {
    println!("  [route: {} -> {}]", decision.rule_name, decision.provider);
}
```

- [ ] **Step 2: Run full validation**

Run: `cargo check --workspace && cargo test --workspace && cargo clippy --workspace --all-targets && cargo fmt --all -- --check`
Expected: ALL CLEAN

- [ ] **Step 3: Commit**

```bash
git add crates/app-host/src/chat.rs
git commit -m "feat(chat): wire ModelRouter for automatic task-to-model routing"
```

---

## Task 4: Update Documentation

**Files:**
- Modify: `docs/plans/runtime-rethink.md`
- Modify: `DESIGN.md`

- [ ] **Step 1: Update roadmap**

Change:
```
- [ ] Model routing: automatic task-to-model assignment
```
to:
```
- [x] Model routing: config-driven ModelRouter with skill/keyword/token matching
```

- [ ] **Step 2: Add routing section to DESIGN.md**

```markdown
### Model Routing

Config-driven routing selects the best provider+model for each task:

1. **RoutingRule** — defines a named route with match criteria (skills, keywords, max_prompt_chars, tool_count_min) and target provider+model
2. **ModelRouter** — evaluates rules in order, first match wins, falls back to default chain
3. **Config** — routes live in `providers.json` alongside provider definitions

**Example routing** — short prompts go to fast local model, skill-heavy prompts go to capable cloud model.
```

- [ ] **Step 3: Commit**

```bash
git add docs/plans/runtime-rethink.md DESIGN.md
git commit -m "docs: add model routing to roadmap and design docs"
```

---

## Summary

| Task | Component | LOC Est. | Tests |
|------|-----------|----------|-------|
| 1 | ModelRouter + RoutingRule + RouteMatcher | ~180 | 8 |
| 2 | ProviderPlaneConfig integration | ~40 | 1 |
| 3 | Wire into chat.rs | ~30 | 0 |
| 4 | Documentation | ~20 | 0 |
| **Total** | | **~270** | **9** |
