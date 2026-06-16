use std::sync::Arc;

use serde::Serialize;

use crate::audit::{AuditLogger, AuditResultStatus};
use crate::auth::{Authorizer, ToolScope};
use crate::backend::{DockerBackend, RunSpec};
use crate::config::Config;
use crate::error::DockerMcpError;
use crate::sanitize::{params_fingerprint, sanitize_output};
use crate::validate::{
    validate_command_args, validate_compose_path, validate_env, validate_log_tail, validate_name,
};

pub struct DockerService {
    pub(crate) backend: Arc<dyn DockerBackend>,
    config: Config,
    audit: AuditLogger,
}

impl DockerService {
    pub fn new(backend: Arc<dyn DockerBackend>, config: Config, audit: AuditLogger) -> Self {
        Self {
            backend,
            config,
            audit,
        }
    }

    pub fn authorizer(&self) -> &Authorizer {
        &self.config.authorizer
    }

    fn audit_result(
        &self,
        tool: &str,
        scope: ToolScope,
        params: &str,
        result: &Result<String, DockerMcpError>,
    ) -> Result<String, DockerMcpError> {
        let (status, bytes, err_msg) = match result {
            Ok(v) => (AuditResultStatus::Success, v.len(), None),
            Err(DockerMcpError::Authorization(_)) => (
                AuditResultStatus::Denied,
                0,
                Some(result.as_ref().err().unwrap().to_string()),
            ),
            Err(e) => (AuditResultStatus::Error, 0, Some(e.to_string())),
        };
        self.audit.log_tool_invoke(
            tool,
            scope.as_str(),
            &params_fingerprint(params),
            status,
            bytes,
            err_msg,
        );
        result.clone()
    }

    fn finish<T: Serialize>(&self, value: &T) -> Result<String, DockerMcpError> {
        let json = serde_json::to_string_pretty(value)
            .map_err(|e| DockerMcpError::internal(e.to_string()))?;
        Ok(sanitize_output(&json, self.config.max_output_bytes))
    }

    fn finish_raw(&self, value: String) -> Result<String, DockerMcpError> {
        Ok(sanitize_output(&value, self.config.max_output_bytes))
    }

    pub async fn system_info(&self) -> Result<String, DockerMcpError> {
        let params = "{}";
        let inner = async {
            self.config.authorizer.authorize(ToolScope::Read)?;
            let info = self.backend.system_info().await?;
            self.finish(&info)
        }
        .await;
        self.audit_result("docker_system_info", ToolScope::Read, params, &inner)
    }

    pub async fn list_containers(&self, all: bool) -> Result<String, DockerMcpError> {
        let params = format!(r#"{{"all":{all}}}"#);
        let inner = async {
            self.config.authorizer.authorize(ToolScope::Read)?;
            let list = self.backend.list_containers(all).await?;
            self.finish(&list)
        }
        .await;
        self.audit_result("docker_list_containers", ToolScope::Read, &params, &inner)
    }

    pub async fn inspect_container(&self, id: &str) -> Result<String, DockerMcpError> {
        let params = format!(r#"{{"id":"{id}"}}"#);
        let inner = async {
            validate_name(id, "container id")?;
            self.config.authorizer.authorize(ToolScope::Read)?;
            let raw = self.backend.inspect_container(id).await?;
            self.finish_raw(raw)
        }
        .await;
        self.audit_result("docker_inspect_container", ToolScope::Read, &params, &inner)
    }

    pub async fn container_logs(&self, id: &str, tail: u64) -> Result<String, DockerMcpError> {
        let params = format!(r#"{{"id":"{id}","tail":{tail}}}"#);
        let inner = async {
            validate_name(id, "container id")?;
            validate_log_tail(tail)?;
            self.config.authorizer.authorize(ToolScope::Read)?;
            let raw = self.backend.container_logs(id, tail).await?;
            self.finish_raw(raw)
        }
        .await;
        self.audit_result("docker_container_logs", ToolScope::Read, &params, &inner)
    }

    pub async fn start_container(&self, id: &str) -> Result<String, DockerMcpError> {
        let params = format!(r#"{{"id":"{id}"}}"#);
        let inner = async {
            validate_name(id, "container id")?;
            self.config.authorizer.authorize(ToolScope::Write)?;
            self.backend.start_container(id).await
        }
        .await;
        self.audit_result("docker_start_container", ToolScope::Write, &params, &inner)
    }

    pub async fn stop_container(
        &self,
        id: &str,
        timeout_secs: u64,
    ) -> Result<String, DockerMcpError> {
        let params = format!(r#"{{"id":"{id}","timeout_secs":{timeout_secs}}}"#);
        let inner = async {
            validate_name(id, "container id")?;
            self.config.authorizer.authorize(ToolScope::Write)?;
            self.backend.stop_container(id, timeout_secs).await
        }
        .await;
        self.audit_result("docker_stop_container", ToolScope::Write, &params, &inner)
    }

    pub async fn restart_container(
        &self,
        id: &str,
        timeout_secs: u64,
    ) -> Result<String, DockerMcpError> {
        let params = format!(r#"{{"id":"{id}","timeout_secs":{timeout_secs}}}"#);
        let inner = async {
            validate_name(id, "container id")?;
            self.config.authorizer.authorize(ToolScope::Write)?;
            self.backend.restart_container(id, timeout_secs).await
        }
        .await;
        self.audit_result(
            "docker_restart_container",
            ToolScope::Write,
            &params,
            &inner,
        )
    }

    pub async fn remove_container(&self, id: &str, force: bool) -> Result<String, DockerMcpError> {
        let params = format!(r#"{{"id":"{id}","force":{force}}}"#);
        let inner = async {
            validate_name(id, "container id")?;
            self.config.authorizer.authorize(ToolScope::Write)?;
            self.backend.remove_container(id, force).await
        }
        .await;
        self.audit_result("docker_remove_container", ToolScope::Write, &params, &inner)
    }

    pub async fn run_container(
        &self,
        image: &str,
        name: Option<&str>,
        command: Vec<String>,
        env: Vec<(String, String)>,
        detach: bool,
    ) -> Result<String, DockerMcpError> {
        let params = serde_json::json!({
            "image": image,
            "name": name,
            "command": command,
            "env": env,
            "detach": detach,
        })
        .to_string();
        let inner = async {
            validate_name(image, "image")?;
            if let Some(n) = name {
                validate_name(n, "container name")?;
            }
            validate_command_args(&command)?;
            validate_env(&env)?;
            self.config.authorizer.authorize(ToolScope::Write)?;
            let spec = RunSpec {
                image: image.into(),
                name: name.map(str::to_string),
                command,
                env,
                detach,
            };
            self.backend.run_container(spec).await
        }
        .await;
        self.audit_result("docker_run_container", ToolScope::Write, &params, &inner)
    }

    pub async fn list_images(&self) -> Result<String, DockerMcpError> {
        let params = "{}";
        let inner = async {
            self.config.authorizer.authorize(ToolScope::Read)?;
            let list = self.backend.list_images().await?;
            self.finish(&list)
        }
        .await;
        self.audit_result("docker_list_images", ToolScope::Read, params, &inner)
    }

    pub async fn inspect_image(&self, name: &str) -> Result<String, DockerMcpError> {
        let params = format!(r#"{{"name":"{name}"}}"#);
        let inner = async {
            validate_name(name, "image name")?;
            self.config.authorizer.authorize(ToolScope::Read)?;
            let raw = self.backend.inspect_image(name).await?;
            self.finish_raw(raw)
        }
        .await;
        self.audit_result("docker_inspect_image", ToolScope::Read, &params, &inner)
    }

    pub async fn pull_image(&self, name: &str) -> Result<String, DockerMcpError> {
        let params = format!(r#"{{"name":"{name}"}}"#);
        let inner = async {
            validate_name(name, "image name")?;
            self.config.authorizer.authorize(ToolScope::Write)?;
            self.backend.pull_image(name).await
        }
        .await;
        self.audit_result("docker_pull_image", ToolScope::Write, &params, &inner)
    }

    pub async fn remove_image(&self, name: &str, force: bool) -> Result<String, DockerMcpError> {
        let params = format!(r#"{{"name":"{name}","force":{force}}}"#);
        let inner = async {
            validate_name(name, "image name")?;
            self.config.authorizer.authorize(ToolScope::Write)?;
            self.backend.remove_image(name, force).await
        }
        .await;
        self.audit_result("docker_remove_image", ToolScope::Write, &params, &inner)
    }

    pub async fn list_networks(&self) -> Result<String, DockerMcpError> {
        let params = "{}";
        let inner = async {
            self.config.authorizer.authorize(ToolScope::Read)?;
            let list = self.backend.list_networks().await?;
            self.finish(&list)
        }
        .await;
        self.audit_result("docker_list_networks", ToolScope::Read, params, &inner)
    }

    pub async fn list_volumes(&self) -> Result<String, DockerMcpError> {
        let params = "{}";
        let inner = async {
            self.config.authorizer.authorize(ToolScope::Read)?;
            let list = self.backend.list_volumes().await?;
            self.finish(&list)
        }
        .await;
        self.audit_result("docker_list_volumes", ToolScope::Read, params, &inner)
    }

    pub async fn compose_ps(&self, compose_file: &str) -> Result<String, DockerMcpError> {
        let params = format!(r#"{{"compose_file":"{compose_file}"}}"#);
        let inner = async {
            validate_compose_path(compose_file)?;
            self.config.authorizer.authorize(ToolScope::Compose)?;
            let raw = self.backend.compose_ps(compose_file).await?;
            self.finish_raw(raw)
        }
        .await;
        self.audit_result("docker_compose_ps", ToolScope::Compose, &params, &inner)
    }

    pub async fn compose_config(&self, compose_file: &str) -> Result<String, DockerMcpError> {
        let params = format!(r#"{{"compose_file":"{compose_file}"}}"#);
        let inner = async {
            validate_compose_path(compose_file)?;
            self.config.authorizer.authorize(ToolScope::Compose)?;
            let raw = self.backend.compose_config(compose_file).await?;
            self.finish_raw(raw)
        }
        .await;
        self.audit_result("docker_compose_config", ToolScope::Compose, &params, &inner)
    }
}

impl DockerMcpError {
    pub(crate) fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::audit::{AuditLogger, MemoryAuditSink};
    use crate::auth::ToolScope;
    use crate::backend::{ContainerSummary, MockBackend, SystemInfo};
    use crate::config::Config;

    use super::*;

    fn test_service(
        scopes: &[ToolScope],
    ) -> (DockerService, Arc<MockBackend>, Arc<MemoryAuditSink>) {
        let sink = Arc::new(MemoryAuditSink::new());
        let audit = AuditLogger::new(sink.clone(), "test", "sess");
        let backend = Arc::new(MockBackend::new());
        let config = Config::for_test(scopes);
        let svc = DockerService::new(backend.clone(), config, audit);
        (svc, backend, sink)
    }

    #[tokio::test]
    async fn system_info_requires_read_scope() {
        let (svc, _, sink) = test_service(&[]);
        let err = svc.system_info().await.unwrap_err();
        assert!(matches!(err, DockerMcpError::Authorization(_)));
        assert_eq!(sink.events().len(), 1);
    }

    #[tokio::test]
    async fn system_info_returns_data_with_scope() {
        let (svc, mock, _) = test_service(&[ToolScope::Read]);
        *mock.system_info.lock().unwrap() = Some(SystemInfo {
            id: "host".into(),
            containers: 3,
            images: 5,
            driver: "overlay2".into(),
            operating_system: "Linux".into(),
            architecture: "amd64".into(),
        });
        let out = svc.system_info().await.unwrap();
        assert!(out.contains("host"));
    }

    #[tokio::test]
    async fn list_containers_returns_json() {
        let (svc, mock, _) = test_service(&[ToolScope::Read]);
        mock.containers.lock().unwrap().push(ContainerSummary {
            id: "abc".into(),
            name: "web".into(),
            image: "nginx".into(),
            state: "running".into(),
            status: "Up".into(),
        });
        let out = svc.list_containers(true).await.unwrap();
        assert!(out.contains("web"));
    }

    #[tokio::test]
    async fn run_container_validates_image() {
        let (svc, _, _) = test_service(&[ToolScope::Write]);
        assert!(svc
            .run_container("bad image", None, vec![], vec![], true)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn compose_requires_compose_scope() {
        let (svc, _, _) = test_service(&[ToolScope::Read]);
        assert!(svc.compose_ps("compose.yaml").await.is_err());
    }

    #[tokio::test]
    async fn inspect_container_success() {
        let (svc, mock, _) = test_service(&[ToolScope::Read]);
        *mock.inspect_container.lock().unwrap() = Some(r#"{"Id":"abc"}"#.into());
        let out = svc.inspect_container("abc").await.unwrap();
        assert!(out.contains("abc"));
    }

    #[tokio::test]
    async fn container_logs_success() {
        let (svc, mock, _) = test_service(&[ToolScope::Read]);
        *mock.logs.lock().unwrap() = Some("log line".into());
        let out = svc.container_logs("web", 50).await.unwrap();
        assert!(out.contains("log line"));
    }

    #[tokio::test]
    async fn container_logs_invalid_tail() {
        let (svc, _, _) = test_service(&[ToolScope::Read]);
        assert!(svc.container_logs("web", 0).await.is_err());
    }

    #[tokio::test]
    async fn start_stop_restart_remove_containers() {
        let (svc, mock, _) = test_service(&[ToolScope::Write]);
        *mock.action_result.lock().unwrap() = "done".into();
        assert_eq!(svc.start_container("c1").await.unwrap(), "done");
        assert_eq!(svc.stop_container("c1", 5).await.unwrap(), "done");
        assert_eq!(svc.restart_container("c1", 5).await.unwrap(), "done");
        assert_eq!(svc.remove_container("c1", true).await.unwrap(), "done");
    }

    #[tokio::test]
    async fn run_container_success() {
        let (svc, mock, _) = test_service(&[ToolScope::Write]);
        *mock.action_result.lock().unwrap() = "started".into();
        let out = svc
            .run_container(
                "alpine:latest",
                Some("test-c"),
                vec!["echo".into()],
                vec![("FOO".into(), "bar".into())],
                true,
            )
            .await
            .unwrap();
        assert_eq!(out, "started");
    }

    #[tokio::test]
    async fn image_operations() {
        let (svc, mock, _) = test_service(&[ToolScope::Read, ToolScope::Write]);
        use crate::backend::ImageSummary;
        mock.images.lock().unwrap().push(ImageSummary {
            id: "img1".into(),
            tags: vec!["nginx:latest".into()],
            size: 100,
            created: 1,
        });
        *mock.inspect_image.lock().unwrap() = Some(r#"{"Id":"img1"}"#.into());
        *mock.action_result.lock().unwrap() = "pulled".into();

        assert!(svc.list_images().await.unwrap().contains("nginx"));
        assert!(svc
            .inspect_image("nginx:latest")
            .await
            .unwrap()
            .contains("img1"));
        assert_eq!(svc.pull_image("nginx:latest").await.unwrap(), "pulled");
        assert_eq!(
            svc.remove_image("nginx:latest", false).await.unwrap(),
            "pulled"
        );
    }

    #[tokio::test]
    async fn network_and_volume_list() {
        let (svc, mock, _) = test_service(&[ToolScope::Read]);
        use crate::backend::{NetworkSummary, VolumeSummary};
        mock.networks.lock().unwrap().push(NetworkSummary {
            id: "n1".into(),
            name: "bridge".into(),
            driver: "bridge".into(),
        });
        mock.volumes.lock().unwrap().push(VolumeSummary {
            name: "vol1".into(),
            driver: "local".into(),
            mountpoint: "/var/lib/docker/volumes/vol1".into(),
        });
        assert!(svc.list_networks().await.unwrap().contains("bridge"));
        assert!(svc.list_volumes().await.unwrap().contains("vol1"));
    }

    #[tokio::test]
    async fn compose_operations() {
        let (svc, mock, _) = test_service(&[ToolScope::Compose]);
        *mock.compose_ps.lock().unwrap() = Some("[]".into());
        *mock.compose_config.lock().unwrap() = Some("services: {}".into());
        assert_eq!(svc.compose_ps("compose.yaml").await.unwrap(), "[]");
        assert!(svc
            .compose_config("compose.yaml")
            .await
            .unwrap()
            .contains("services"));
    }

    #[tokio::test]
    async fn docker_errors_are_audited() {
        let (svc, mock, sink) = test_service(&[ToolScope::Read]);
        *mock.fail_with.lock().unwrap() = Some(DockerMcpError::docker("boom"));
        assert!(svc.list_images().await.is_err());
        assert_eq!(
            sink.events().last().unwrap().result_status,
            crate::audit::AuditResultStatus::Error
        );
    }

    #[test]
    fn internal_error_constructor() {
        let err = DockerMcpError::internal("serialize failed");
        assert!(matches!(err, DockerMcpError::Internal(_)));
    }

    #[tokio::test]
    async fn run_container_rejects_bad_container_name() {
        let (svc, _, _) = test_service(&[ToolScope::Write]);
        assert!(svc
            .run_container("alpine", Some("bad name"), vec![], vec![], true)
            .await
            .is_err());
    }
}
