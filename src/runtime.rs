use std::io;
use std::sync::Arc;

use crate::audit::{AuditLogger, JsonLineAuditSink};
use crate::config::Config;
use tracing_subscriber::EnvFilter;

pub fn init_tracing(config: &Config) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    if config.audit_json_logs {
        let _ = tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .with_writer(io::stderr)
            .try_init();
    } else {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(io::stderr)
            .try_init();
    }
}

pub fn build_audit_logger(config: &Config) -> AuditLogger {
    let sink: Arc<dyn crate::audit::AuditSink> =
        Arc::new(JsonLineAuditSink::new(Box::new(io::stderr())));
    AuditLogger::new(
        sink,
        config.caller_identity.clone(),
        config.session_id.clone(),
    )
}
