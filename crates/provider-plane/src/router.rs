use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    #[serde(rename = "match")]
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

#[derive(Debug, Clone)]
pub struct ModelRouter {
    rules: Vec<RoutingRule>,
    default_provider: String,
    default_model: Option<String>,
}

impl ModelRouter {
    pub fn new(
        rules: Vec<RoutingRule>,
        default_provider: String,
        default_model: Option<String>,
    ) -> Self {
        Self {
            rules,
            default_provider,
            default_model,
        }
    }

    pub fn empty(default_provider: String, default_model: Option<String>) -> Self {
        Self {
            rules: vec![],
            default_provider,
            default_model,
        }
    }

    pub fn route(
        &self,
        prompt: &str,
        tool_count: usize,
        skill_hints: &[String],
    ) -> RoutingDecision {
        for rule in &self.rules {
            if Self::matches(&rule.match_, prompt, tool_count, skill_hints) {
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

    fn matches(
        matcher: &RouteMatcher,
        prompt: &str,
        tool_count: usize,
        skill_hints: &[String],
    ) -> bool {
        if !matcher.skills.is_empty() {
            let matched = skill_hints.iter().any(|hint| {
                matcher
                    .skills
                    .iter()
                    .any(|s| hint.to_lowercase().contains(&s.to_lowercase()))
            });
            if !matched {
                return false;
            }
        }

        if !matcher.keywords.is_empty() {
            let prompt_lower = prompt.to_lowercase();
            let matched = matcher
                .keywords
                .iter()
                .any(|k| prompt_lower.contains(&k.to_lowercase()));
            if !matched {
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

    #[test]
    fn empty_rules_returns_default() {
        let router = ModelRouter::empty("openrouter".to_string(), Some("kimi".to_string()));
        let decision = router.route("hello", 0, &[]);
        assert_eq!(decision.rule_name, "default");
        assert_eq!(decision.provider, "openrouter");
        assert_eq!(decision.model.as_deref(), Some("kimi"));
    }

    #[test]
    fn matches_by_skill() {
        let router = ModelRouter::new(
            vec![RoutingRule {
                name: "deploy".to_string(),
                match_: RouteMatcher {
                    skills: vec!["deploy".to_string()],
                    ..Default::default()
                },
                provider: "ollama".to_string(),
                model: Some("llama3".to_string()),
            }],
            "openrouter".to_string(),
            None,
        );
        let decision = router.route("do something", 0, &["deploy-skill".to_string()]);
        assert_eq!(decision.rule_name, "deploy");
        assert_eq!(decision.provider, "ollama");
    }

    #[test]
    fn matches_by_keyword() {
        let router = ModelRouter::new(
            vec![RoutingRule {
                name: "quick".to_string(),
                match_: RouteMatcher {
                    keywords: vec!["quick".to_string()],
                    ..Default::default()
                },
                provider: "fast".to_string(),
                model: None,
            }],
            "default".to_string(),
            None,
        );
        let decision = router.route("quick question", 0, &[]);
        assert_eq!(decision.rule_name, "quick");
        assert_eq!(decision.provider, "fast");
    }

    #[test]
    fn no_match_falls_to_default() {
        let router = ModelRouter::new(
            vec![RoutingRule {
                name: "deploy".to_string(),
                match_: RouteMatcher {
                    skills: vec!["deploy".to_string()],
                    ..Default::default()
                },
                provider: "ollama".to_string(),
                model: None,
            }],
            "openrouter".to_string(),
            Some("default-model".to_string()),
        );
        let decision = router.route("hello", 0, &["other-skill".to_string()]);
        assert_eq!(decision.rule_name, "default");
        assert_eq!(decision.provider, "openrouter");
    }

    #[test]
    fn matches_by_tool_count() {
        let router = ModelRouter::new(
            vec![RoutingRule {
                name: "heavy".to_string(),
                match_: RouteMatcher {
                    tool_count_min: Some(5),
                    ..Default::default()
                },
                provider: "big".to_string(),
                model: None,
            }],
            "default".to_string(),
            None,
        );
        let decision = router.route("do stuff", 10, &[]);
        assert_eq!(decision.rule_name, "heavy");
        let decision = router.route("do stuff", 2, &[]);
        assert_eq!(decision.rule_name, "default");
    }

    #[test]
    fn first_matching_rule_wins() {
        let router = ModelRouter::new(
            vec![
                RoutingRule {
                    name: "first".to_string(),
                    match_: RouteMatcher {
                        keywords: vec!["test".to_string()],
                        ..Default::default()
                    },
                    provider: "provider-a".to_string(),
                    model: None,
                },
                RoutingRule {
                    name: "second".to_string(),
                    match_: RouteMatcher {
                        keywords: vec!["test".to_string()],
                        ..Default::default()
                    },
                    provider: "provider-b".to_string(),
                    model: None,
                },
            ],
            "default".to_string(),
            None,
        );
        let decision = router.route("test input", 0, &[]);
        assert_eq!(decision.rule_name, "first");
        assert_eq!(decision.provider, "provider-a");
    }

    #[test]
    fn matches_by_max_prompt_chars() {
        let router = ModelRouter::new(
            vec![RoutingRule {
                name: "short".to_string(),
                match_: RouteMatcher {
                    max_prompt_chars: Some(100),
                    ..Default::default()
                },
                provider: "fast".to_string(),
                model: None,
            }],
            "default".to_string(),
            None,
        );
        let decision = router.route("hi", 0, &[]);
        assert_eq!(decision.rule_name, "short");
        let long_input = "x".repeat(200);
        let decision = router.route(&long_input, 0, &[]);
        assert_eq!(decision.rule_name, "default");
    }

    #[test]
    fn config_deserialization() {
        let json = r#"{
            "name": "code-rule",
            "match": {
                "skills": ["coding"],
                "keywords": ["code", "debug"],
                "max_prompt_chars": 5000,
                "tool_count_min": 3
            },
            "provider": "openrouter",
            "model": "moonshotai/kimi-k2.5"
        }"#;
        let rule: RoutingRule = serde_json::from_str(json).unwrap();
        assert_eq!(rule.name, "code-rule");
        assert_eq!(rule.match_.skills, vec!["coding"]);
        assert_eq!(rule.match_.keywords, vec!["code", "debug"]);
        assert_eq!(rule.match_.max_prompt_chars, Some(5000));
        assert_eq!(rule.match_.tool_count_min, Some(3));
        assert_eq!(rule.provider, "openrouter");
        assert_eq!(rule.model.as_deref(), Some("moonshotai/kimi-k2.5"));

        let serialized = serde_json::to_string(&rule).unwrap();
        let roundtrip: RoutingRule = serde_json::from_str(&serialized).unwrap();
        assert_eq!(roundtrip.name, rule.name);
        assert_eq!(roundtrip.provider, rule.provider);
        assert_eq!(roundtrip.model, rule.model);
    }
}
