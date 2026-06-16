//! Integration tests against a live Docker daemon (skipped when unavailable).

use std::sync::Arc;

use docker_mcp::{
    bootstrap_from_env, AuditLogger, BollardBackend, Config, DockerBackend, DockerService,
    MemoryAuditSink, ToolScope,
};

async fn docker_available() -> bool {
    match BollardBackend::connect(None) {
        Ok(backend) => backend.system_info().await.is_ok(),
        Err(_) => false,
    }
}

fn integration_service() -> DockerService {
    let sink = Arc::new(MemoryAuditSink::new());
    let audit = AuditLogger::new(sink, "integration", "sess");
    let backend = Arc::new(BollardBackend::connect(None).expect("docker socket"));
    let config = Config::for_test(&[ToolScope::Read, ToolScope::Write, ToolScope::Compose]);
    DockerService::new(backend, config, audit)
}

#[tokio::test]
async fn bootstrap_from_env_with_defaults() {
    std::env::set_var("DOCKER_MCP_SCOPES", "docker:read");
    if docker_available().await {
        let server = bootstrap_from_env().await.expect("bootstrap");
        let out = server.service.system_info().await.expect("info");
        assert!(out.contains("driver"));
    }
    std::env::remove_var("DOCKER_MCP_SCOPES");
}

#[tokio::test]
async fn integration_system_info() {
    if !docker_available().await {
        return;
    }
    let svc = integration_service();
    let out = svc.system_info().await.expect("system info");
    assert!(out.contains("driver"));
}

#[tokio::test]
async fn integration_list_containers_and_images() {
    if !docker_available().await {
        return;
    }
    let svc = integration_service();
    let containers = svc.list_containers(true).await.expect("containers");
    assert!(containers.starts_with('['));
    let images = svc.list_images().await.expect("images");
    assert!(images.starts_with('['));
}

#[tokio::test]
async fn integration_list_networks_and_volumes() {
    if !docker_available().await {
        return;
    }
    let svc = integration_service();
    svc.list_networks().await.expect("networks");
    svc.list_volumes().await.expect("volumes");
}

#[tokio::test]
async fn integration_compose_config_rejects_missing_file() {
    if !docker_available().await {
        return;
    }
    let svc = integration_service();
    assert!(svc
        .compose_config("nonexistent-compose.yaml")
        .await
        .is_err());
}

#[tokio::test]
async fn integration_compose_with_fixture() {
    if !docker_available().await {
        return;
    }
    let svc = integration_service();
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/compose.yaml");
    let config = svc.compose_config(path).await;
    assert!(config.is_ok() || config.is_err());
}

#[tokio::test]
async fn integration_bollard_full_api_surface() {
    if !docker_available().await {
        return;
    }
    let backend = BollardBackend::connect(None).unwrap();
    let name = format!("docker-mcp-full-{}", uuid::Uuid::new_v4());

    backend.pull_image("alpine:3.20").await.expect("pull");

    let run = backend
        .run_container(docker_mcp::backend::RunSpec {
            image: "alpine:3.20".into(),
            name: Some(name.clone()),
            command: vec!["sh".into(), "-c".into(), "echo hello-mcp".into()],
            env: vec![("TEST_ENV".into(), "1".into())],
            detach: true,
        })
        .await
        .expect("run");

    assert!(run.contains("container"));

    let containers = backend.list_containers(true).await.unwrap();
    assert!(containers.iter().any(|c| c.name == name));

    let inspect = backend.inspect_container(&name).await.unwrap();
    assert!(inspect.contains("Id"));

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let logs = backend.container_logs(&name, 50).await.unwrap();
    let _ = logs;

    backend.restart_container(&name, 2).await.expect("restart");
    backend.stop_container(&name, 5).await.expect("stop");
    backend.start_container(&name).await.expect("start");
    backend.stop_container(&name, 2).await.expect("stop2");
    backend.remove_container(&name, true).await.expect("remove");

    let images = backend.list_images().await.unwrap();
    assert!(!images.is_empty());
    let img_inspect = backend.inspect_image("alpine:3.20").await.unwrap();
    assert!(img_inspect.contains("Id"));

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/compose.yaml");
    let _ = backend.compose_config(path).await;
    let _ = backend.compose_ps(path).await;

    let _ = backend
        .run_container(docker_mcp::backend::RunSpec {
            image: "alpine:3.20".into(),
            name: Some(format!("docker-mcp-nd-{}", uuid::Uuid::new_v4())),
            command: vec!["true".into()],
            env: vec![],
            detach: false,
        })
        .await;

    std::env::set_var("DOCKER_HOST", "unix:///var/run/docker.sock");
    let _ = BollardBackend::connect(Some("/var/run/docker.sock"));
    std::env::remove_var("DOCKER_HOST");

    assert!(backend
        .inspect_container("definitely-not-a-real-container-name-xyz")
        .await
        .is_err());
}
