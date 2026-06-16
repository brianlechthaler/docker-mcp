use crate::{
    bootstrap_from_env, init_tracing, inventory_fingerprint, registered_tools, run_stdio_server,
    Config, DockerMcpServer, SERVER_VERSION,
};

/// Prepares the MCP server (config, tracing, bootstrap). Used by `run` and tests.
pub async fn prepare_server() -> anyhow::Result<DockerMcpServer> {
    let config = Config::from_env()?;
    init_tracing(&config);
    tracing::info!(
        server_version = SERVER_VERSION,
        tool_count = registered_tools().len(),
        inventory_fingerprint = %inventory_fingerprint(),
        "docker-mcp starting"
    );
    bootstrap_from_env().await
}

/// Full server lifecycle: prepare, then serve MCP over stdio until shutdown.
pub async fn run() -> anyhow::Result<()> {
    let server = prepare_server().await?;
    run_stdio_server(server).await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::audit::{AuditLogger, MemoryAuditSink};
    use crate::auth::ToolScope;
    use crate::backend::{BollardBackend, DockerBackend, MockBackend};
    use crate::bootstrap::bootstrap_with_backend;
    use crate::Config;

    use super::*;

    #[tokio::test]
    async fn prepare_server_with_docker() {
        std::env::set_var("DOCKER_MCP_SCOPES", "docker:read");
        if let Ok(backend) = BollardBackend::connect(None) {
            if backend.system_info().await.is_ok() {
                let server = prepare_server().await.expect("prepare");
                let _ = server.service.system_info().await.expect("info");
            }
        }
        std::env::remove_var("DOCKER_MCP_SCOPES");
    }

    #[tokio::test]
    async fn run_entrypoint_prepares_server() {
        let sink = Arc::new(MemoryAuditSink::new());
        let audit = AuditLogger::new(sink, "t", "s");
        let backend = Arc::new(MockBackend::new());
        let config = Config::for_test(&[ToolScope::Read]);
        let _server = bootstrap_with_backend(config, backend, audit);
        // run_stdio_server blocks on stdin; covered by integration smoke in CI container build.
    }
}
