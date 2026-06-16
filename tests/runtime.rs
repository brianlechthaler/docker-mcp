use std::sync::Arc;

use docker_mcp::runtime::{build_audit_logger, init_tracing};
use docker_mcp::{AuditLogger, Config, JsonLineAuditSink};

#[test]
fn init_tracing_plain_text() {
    let mut config = Config::for_test(&[]);
    config.audit_json_logs = false;
    let _ = std::panic::catch_unwind(|| init_tracing(&config));
}

#[test]
fn build_audit_logger_returns_logger() {
    let config = Config::for_test(&[]);
    let logger = build_audit_logger(&config);
    logger.log_tool_invoke(
        "t",
        "docker:read",
        "fp",
        docker_mcp::AuditResultStatus::Success,
        1,
        None,
    );
}

#[test]
fn json_audit_sink_stderr_constructible() {
    let sink = JsonLineAuditSink::new(Box::new(std::io::sink()));
    let _logger = AuditLogger::new(Arc::new(sink), "a", "s");
}
