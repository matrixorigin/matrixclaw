use zstar_app_host::http::execution_api::execution_visibility_snapshot;

#[test]
fn execution_backend_visibility() {
    let snapshot = execution_visibility_snapshot();

    assert_eq!(
        snapshot.visible_backends,
        vec![
            "docker".to_string(),
            "e2b".to_string(),
            "daytona".to_string(),
            "local".to_string(),
        ],
        "execution UI should expose the product-facing backend labels"
    );
    assert_eq!(
        snapshot.sandbox_priority,
        vec![
            "docker".to_string(),
            "e2b".to_string(),
            "daytona".to_string(),
            "local".to_string(),
        ],
        "sandbox policy should show docker, e2b, daytona, then local"
    );
    assert_eq!(
        snapshot.sandbox_failure_message, "sandbox required but unavailable",
        "sandbox-required failures should be explicit"
    );
}
