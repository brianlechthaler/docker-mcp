use crate::auth::{parse_scopes, Authorizer, ToolScope};
use crate::error::DockerMcpError;
use crate::sanitize::default_max_bytes;

#[derive(Debug, Clone)]
pub struct Config {
    pub caller_identity: String,
    pub session_id: String,
    pub max_output_bytes: usize,
    pub authorizer: Authorizer,
    pub docker_socket: Option<String>,
    pub audit_json_logs: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, DockerMcpError> {
        let scopes_raw = std::env::var("DOCKER_MCP_SCOPES")
            .unwrap_or_else(|_| "docker:read,docker:write,docker:compose".to_string());
        let scopes = parse_scopes(&scopes_raw);
        if scopes.is_empty() {
            return Err(DockerMcpError::validation(
                "DOCKER_MCP_SCOPES must include at least one valid scope",
            ));
        }
        let authorizer = Authorizer::from_scopes(&scopes);
        let max_output_bytes = std::env::var("DOCKER_MCP_MAX_OUTPUT_BYTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(default_max_bytes);
        Ok(Self {
            caller_identity: std::env::var("DOCKER_MCP_CALLER_IDENTITY")
                .unwrap_or_else(|_| "unknown-agent".into()),
            session_id: std::env::var("DOCKER_MCP_SESSION_ID")
                .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string()),
            max_output_bytes,
            authorizer,
            docker_socket: std::env::var("DOCKER_HOST").ok(),
            audit_json_logs: std::env::var("DOCKER_MCP_AUDIT_JSON")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(true),
        })
    }

    pub fn for_test(scopes: &[ToolScope]) -> Self {
        Self {
            caller_identity: "test-agent".into(),
            session_id: "test-session".into(),
            max_output_bytes: 4096,
            authorizer: Authorizer::from_scopes(scopes),
            docker_socket: None,
            audit_json_logs: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    #[test]
    fn for_test_builds_config() {
        let cfg = Config::for_test(&[ToolScope::Read]);
        assert_eq!(cfg.caller_identity, "test-agent");
        assert!(cfg.authorizer.authorize(ToolScope::Read).is_ok());
    }

    #[test]
    fn from_env_uses_defaults() {
        let _guard = env_lock();
        std::env::remove_var("DOCKER_MCP_SCOPES");
        std::env::remove_var("DOCKER_MCP_CALLER_IDENTITY");
        let cfg = Config::from_env().unwrap();
        assert_eq!(cfg.caller_identity, "unknown-agent");
        assert!(cfg.authorizer.authorize(ToolScope::Read).is_ok());
    }

    #[test]
    fn from_env_rejects_empty_scopes() {
        let _guard = env_lock();
        std::env::set_var("DOCKER_MCP_SCOPES", "invalid,also-invalid");
        let err = Config::from_env().unwrap_err();
        assert!(matches!(err, DockerMcpError::Validation(_)));
        std::env::remove_var("DOCKER_MCP_SCOPES");
    }

    #[test]
    fn from_env_reads_docker_host() {
        let _guard = env_lock();
        std::env::set_var("DOCKER_HOST", "unix:///var/run/docker.sock");
        std::env::set_var("DOCKER_MCP_SCOPES", "docker:read");
        let cfg = Config::from_env().unwrap();
        assert_eq!(
            cfg.docker_socket,
            Some("unix:///var/run/docker.sock".into())
        );
        std::env::remove_var("DOCKER_HOST");
        std::env::remove_var("DOCKER_MCP_SCOPES");
    }
}
