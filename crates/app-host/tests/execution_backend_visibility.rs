use matrixclaw_app_host::http::execution_api::execution_visibility_snapshot;

#[test]
fn execution_backend_visibility() {
    let snapshot = execution_visibility_snapshot();

    assert_eq!(
        snapshot.visible_backends,
        vec![
            "local".to_string(),
            "docker".to_string(),
            "boxlite".to_string(),
        ],
        "execution UI should expose the product-facing backend labels"
    );
    assert_eq!(
        snapshot.sandbox_priority,
        vec!["docker".to_string(), "boxlite".to_string()],
        "sandbox policy should show docker first and boxlite second"
    );
    assert_eq!(
        snapshot.sandbox_failure_message, "sandbox required but unavailable",
        "sandbox-required failures should be explicit"
    );
}
