#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetupServerContract {
    pub shell_route: &'static str,
    pub submit_route: &'static str,
    pub health_route: &'static str,
    pub browser_first: bool,
}

pub const LOOPBACK_BASE_URL: &str = "http://127.0.0.1:38495";
pub const SETUP_SHELL_ROUTE: &str = "/setup";
pub const SETUP_SUBMIT_ROUTE: &str = "/api/setup/config";
pub const SETUP_HEALTH_ROUTE: &str = "/healthz";

pub fn setup_server_contract() -> SetupServerContract {
    SetupServerContract {
        shell_route: SETUP_SHELL_ROUTE,
        submit_route: SETUP_SUBMIT_ROUTE,
        health_route: SETUP_HEALTH_ROUTE,
        browser_first: true,
    }
}

pub fn is_health_route(path: &str) -> bool {
    normalize_path(path) == SETUP_HEALTH_ROUTE
}

pub fn is_submit_route(path: &str) -> bool {
    normalize_path(path) == SETUP_SUBMIT_ROUTE
}

pub fn is_shell_route(path: &str) -> bool {
    let normalized = normalize_path(path);
    normalized == "/"
        || normalized == SETUP_SHELL_ROUTE
        || normalized.starts_with("/setup/")
        || (!normalized.starts_with("/api/") && !normalized.starts_with("/healthz"))
}

fn normalize_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "/" {
        return "/".to_string();
    }

    let without_query = trimmed
        .split_once(['?', '#'])
        .map(|(prefix, _)| prefix)
        .unwrap_or(trimmed);

    if without_query.starts_with('/') {
        without_query.to_string()
    } else {
        format!("/{without_query}")
    }
}
