use std::io::Write;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditResultStatus {
    Success,
    Denied,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditEvent {
    pub timestamp: String,
    pub event_type: String,
    pub tool_name: String,
    pub caller_identity: String,
    pub authorization_scope: String,
    pub params_fingerprint: String,
    pub result_status: AuditResultStatus,
    pub result_bytes: usize,
    pub correlation_id: String,
    pub session_id: String,
    pub error_message: Option<String>,
}

pub trait AuditSink: Send + Sync {
    fn write_event(&self, event: &AuditEvent);
}

#[derive(Default)]
pub struct MemoryAuditSink {
    events: Mutex<Vec<AuditEvent>>,
}

impl MemoryAuditSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> Vec<AuditEvent> {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}

impl AuditSink for MemoryAuditSink {
    fn write_event(&self, event: &AuditEvent) {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(event.clone());
    }
}

pub struct AuditLogger {
    sink: Arc<dyn AuditSink>,
    caller_identity: String,
    session_id: String,
}

impl AuditLogger {
    pub fn new(
        sink: Arc<dyn AuditSink>,
        caller_identity: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            sink,
            caller_identity: caller_identity.into(),
            session_id: session_id.into(),
        }
    }

    pub fn log_tool_invoke(
        &self,
        tool_name: &str,
        scope: &str,
        params_fingerprint: &str,
        result_status: AuditResultStatus,
        result_bytes: usize,
        error_message: Option<String>,
    ) {
        let event = AuditEvent {
            timestamp: chrono_now(),
            event_type: "mcp.tool.result".into(),
            tool_name: tool_name.into(),
            caller_identity: self.caller_identity.clone(),
            authorization_scope: scope.into(),
            params_fingerprint: params_fingerprint.into(),
            result_status,
            result_bytes,
            correlation_id: Uuid::new_v4().to_string(),
            session_id: self.session_id.clone(),
            error_message,
        };
        self.sink.write_event(&event);
    }
}

pub struct JsonLineAuditSink {
    writer: Mutex<Box<dyn Write + Send>>,
}

impl JsonLineAuditSink {
    pub fn new(writer: Box<dyn Write + Send>) -> Self {
        Self {
            writer: Mutex::new(writer),
        }
    }
}

impl AuditSink for JsonLineAuditSink {
    fn write_event(&self, event: &AuditEvent) {
        if let Ok(line) = serde_json::to_string(event) {
            let mut guard = self.writer.lock().unwrap_or_else(|e| e.into_inner());
            let _ = writeln!(guard, "{line}");
        }
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}Z", dur.as_secs(), dur.subsec_millis())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_sink_records_events() {
        let sink = Arc::new(MemoryAuditSink::new());
        let logger = AuditLogger::new(sink.clone(), "agent-1", "sess-1");
        logger.log_tool_invoke(
            "docker_list_containers",
            "docker:read",
            "sha256:abc",
            AuditResultStatus::Success,
            42,
            None,
        );
        let events = sink.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].tool_name, "docker_list_containers");
        assert_eq!(events[0].result_status, AuditResultStatus::Success);
        assert_eq!(events[0].caller_identity, "agent-1");
    }

    #[test]
    fn json_line_sink_writes_json() {
        let shared: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
        let shared_clone = shared.clone();
        struct SharedWriter(Arc<Mutex<Vec<u8>>>);
        impl Write for SharedWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let sink = JsonLineAuditSink::new(Box::new(SharedWriter(shared_clone)));
        let event = AuditEvent {
            timestamp: "t".into(),
            event_type: "mcp.tool.result".into(),
            tool_name: "docker_list_containers".into(),
            caller_identity: "c".into(),
            authorization_scope: "s".into(),
            params_fingerprint: "p".into(),
            result_status: AuditResultStatus::Denied,
            result_bytes: 0,
            correlation_id: "cid".into(),
            session_id: "sid".into(),
            error_message: Some("denied".into()),
        };
        sink.write_event(&event);
        let data = String::from_utf8(shared.lock().unwrap().clone()).unwrap();
        assert!(data.contains("docker_list_containers"));
        assert!(data.contains("denied"));
        let mut writer = SharedWriter(shared.clone());
        writer.flush().unwrap();
    }

    #[test]
    fn json_line_sink_direct_write_event() {
        let sink = JsonLineAuditSink::new(Box::new(std::io::sink()));
        let event = AuditEvent {
            timestamp: "t".into(),
            event_type: "mcp.tool.result".into(),
            tool_name: "t".into(),
            caller_identity: "c".into(),
            authorization_scope: "s".into(),
            params_fingerprint: "p".into(),
            result_status: AuditResultStatus::Success,
            result_bytes: 0,
            correlation_id: "c".into(),
            session_id: "s".into(),
            error_message: None,
        };
        sink.write_event(&event);
    }

    #[test]
    fn chrono_now_format() {
        let ts = chrono_now();
        assert!(ts.ends_with('Z'));
    }
}
