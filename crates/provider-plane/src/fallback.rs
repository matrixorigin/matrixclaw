use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use matrixclaw_agent_core::event::AgentEvent;
use matrixclaw_agent_core::provider::{Provider, ProviderError, ProviderResponse};
use matrixclaw_agent_core::RunRequest;

use crate::health::HealthChecker;
use crate::rate_limit::RateLimiter;
use crate::registry::ProviderRegistry;

pub struct FallbackProvider {
    registry: Arc<ProviderRegistry>,
    chain: Vec<String>,
    health: Arc<HealthChecker>,
    rate_limiters: HashMap<String, RateLimiter>,
}

impl FallbackProvider {
    pub fn new(registry: Arc<ProviderRegistry>, chain: Vec<String>) -> Self {
        let health = Arc::new(HealthChecker::new());
        let rate_limiters = chain
            .iter()
            .map(|name| (name.clone(), RateLimiter::new(60)))
            .collect();
        Self {
            registry,
            chain,
            health,
            rate_limiters,
        }
    }

    pub fn with_rate_limits(mut self, limits: Vec<(String, u32)>) -> Self {
        for (name, rpm) in limits {
            self.rate_limiters.insert(name, RateLimiter::new(rpm));
        }
        self
    }

    pub fn health_checker(&self) -> &Arc<HealthChecker> {
        &self.health
    }

    fn check_rate_limit(&self, name: &str) -> bool {
        self.rate_limiters
            .get(name)
            .map(|limiter| limiter.try_acquire())
            .unwrap_or(true)
    }
}

#[async_trait]
impl Provider for FallbackProvider {
    async fn complete(&mut self, request: &RunRequest) -> Result<ProviderResponse, ProviderError> {
        for name in &self.chain {
            if !self.health.is_healthy(name) {
                continue;
            }
            if !self.check_rate_limit(name) {
                continue;
            }
            let Some(entry) = self.registry.get(name).await else {
                continue;
            };
            let mut provider = entry.provider_mut().await;
            match provider.complete(request).await {
                Ok(response) => return Ok(response),
                Err(_e) => {
                    self.health.mark_unhealthy(name);
                    continue;
                }
            }
        }
        Err(ProviderError(
            "all providers in fallback chain failed".to_string(),
        ))
    }

    async fn stream(
        &mut self,
        request: &RunRequest,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<ProviderResponse, ProviderError> {
        for name in &self.chain {
            if !self.health.is_healthy(name) {
                continue;
            }
            if !self.check_rate_limit(name) {
                continue;
            }
            let Some(entry) = self.registry.get(name).await else {
                continue;
            };
            let mut provider = entry.provider_mut().await;
            match provider.stream(request, on_event).await {
                Ok(response) => return Ok(response),
                Err(_e) => {
                    self.health.mark_unhealthy(name);
                    continue;
                }
            }
        }
        Err(ProviderError(
            "all providers in fallback chain failed".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn empty_chain_returns_error() {
        let registry = ProviderRegistry::new();
        let mut fp = FallbackProvider::new(Arc::new(registry), vec![]);
        let request = RunRequest::new("test");
        let result = fp.stream(&request, &mut |_| {}).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn missing_provider_in_chain_returns_error() {
        let registry = ProviderRegistry::new();
        let mut fp = FallbackProvider::new(Arc::new(registry), vec!["nonexistent".to_string()]);
        let request = RunRequest::new("test");
        let result = fp.stream(&request, &mut |_| {}).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn skips_unhealthy_provider() {
        let registry = ProviderRegistry::new();
        let mut fp = FallbackProvider::new(Arc::new(registry), vec!["bad".to_string()]);
        fp.health.mark_unhealthy("bad");
        let request = RunRequest::new("test");
        let result = fp.stream(&request, &mut |_| {}).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn health_checker_accessible() {
        let registry = ProviderRegistry::new();
        let fp = FallbackProvider::new(Arc::new(registry), vec![]);
        assert!(fp.health_checker().is_healthy("any"));
    }
}
