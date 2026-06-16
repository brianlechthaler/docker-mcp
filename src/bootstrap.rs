use std::sync::Arc;

use crate::audit::AuditLogger;
use crate::backend::DockerBackend;
use crate::config::Config;
use crate::server::DockerMcpServer;
use crate::service::DockerService;

pub fn bootstrap_with_backend(
    config: Config,
    backend: Arc<dyn DockerBackend>,
    audit: AuditLogger,
) -> DockerMcpServer {
    let service = Arc::new(DockerService::new(backend, config, audit));
    DockerMcpServer::new(service)
}

pub async fn bootstrap_from_env() -> anyhow::Result<DockerMcpServer> {
    let config = Config::from_env()?;
    let backend = Arc::new(crate::backend::BollardBackend::connect(
        config.docker_socket.as_deref(),
    )?);
    let audit = crate::runtime::build_audit_logger(&config);
    Ok(bootstrap_with_backend(config, backend, audit))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::audit::{AuditLogger, MemoryAuditSink};
    use crate::auth::ToolScope;
    use crate::backend::MockBackend;
    use crate::config::Config;

    use super::*;

    #[test]
    fn bootstrap_with_mock_backend() {
        let sink = Arc::new(MemoryAuditSink::new());
        let audit = AuditLogger::new(sink, "test", "sess");
        let backend = Arc::new(MockBackend::new());
        let config = Config::for_test(&[ToolScope::Read]);
        let server = bootstrap_with_backend(config, backend, audit);
        assert!(server
            .service
            .authorizer()
            .authorize(ToolScope::Read)
            .is_ok());
    }
}
