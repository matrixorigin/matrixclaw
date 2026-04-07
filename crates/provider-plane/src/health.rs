use std::collections::HashMap;
use std::sync::Mutex;

pub struct HealthChecker {
    states: Mutex<HashMap<String, bool>>,
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
        }
    }

    pub fn is_healthy(&self, provider_name: &str) -> bool {
        let states = self.states.lock().unwrap();
        states.get(provider_name).copied().unwrap_or(true)
    }

    pub fn mark_unhealthy(&self, provider_name: &str) {
        let mut states = self.states.lock().unwrap();
        states.insert(provider_name.to_string(), false);
    }

    pub fn mark_healthy(&self, provider_name: &str) {
        let mut states = self.states.lock().unwrap();
        states.insert(provider_name.to_string(), true);
    }

    pub async fn probe_endpoint(&self, provider_name: &str, base_url: &str) -> bool {
        let client = reqwest::Client::new();
        let url = format!("{base_url}/models");
        let result = client
            .head(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;
        let healthy = result.is_ok();
        if healthy {
            self.mark_healthy(provider_name);
        } else {
            self.mark_unhealthy(provider_name);
        }
        healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_endpoint_is_healthy_by_default() {
        let checker = HealthChecker::new();
        assert!(checker.is_healthy("test-provider"));
    }

    #[test]
    fn mark_unhealthy_then_recover() {
        let checker = HealthChecker::new();
        checker.mark_unhealthy("test-provider");
        assert!(!checker.is_healthy("test-provider"));
        checker.mark_healthy("test-provider");
        assert!(checker.is_healthy("test-provider"));
    }

    #[test]
    fn unknown_provider_is_healthy() {
        let checker = HealthChecker::new();
        assert!(checker.is_healthy("anything"));
    }
}
