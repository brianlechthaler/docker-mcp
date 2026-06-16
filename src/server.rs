use std::sync::Arc;

use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router, ServiceExt};
use serde::Deserialize;

use crate::service::DockerService;

#[derive(Clone)]
pub struct DockerMcpServer {
    pub service: Arc<DockerService>,
}

impl DockerMcpServer {
    pub fn new(service: Arc<DockerService>) -> Self {
        Self { service }
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AllParam {
    #[schemars(description = "Include stopped containers")]
    all: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct IdParam {
    #[schemars(description = "Container ID or name")]
    id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LogsParam {
    id: String,
    #[schemars(description = "Number of log lines (1-10000)")]
    tail: Option<u64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct StopParam {
    id: String,
    #[schemars(description = "Seconds to wait before killing")]
    timeout_secs: Option<u64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ForceParam {
    id: String,
    force: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NameParam {
    #[schemars(description = "Image name or ID")]
    name: String,
    force: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RunParam {
    image: String,
    name: Option<String>,
    command: Option<Vec<String>>,
    env: Option<Vec<(String, String)>>,
    detach: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ComposeParam {
    #[schemars(description = "Path to compose file")]
    compose_file: String,
}

fn tool_err(e: crate::error::DockerMcpError) -> String {
    e.to_string()
}

#[tool_router(server_handler)]
impl DockerMcpServer {
    #[tool(description = "Get Docker daemon system information")]
    async fn docker_system_info(&self) -> Result<String, String> {
        self.service.system_info().await.map_err(tool_err)
    }

    #[tool(description = "List Docker containers")]
    async fn docker_list_containers(
        &self,
        Parameters(AllParam { all }): Parameters<AllParam>,
    ) -> Result<String, String> {
        self.service
            .list_containers(all.unwrap_or(false))
            .await
            .map_err(tool_err)
    }

    #[tool(description = "Inspect a container by ID or name")]
    async fn docker_inspect_container(
        &self,
        Parameters(IdParam { id }): Parameters<IdParam>,
    ) -> Result<String, String> {
        self.service.inspect_container(&id).await.map_err(tool_err)
    }

    #[tool(description = "Fetch container logs")]
    async fn docker_container_logs(
        &self,
        Parameters(LogsParam { id, tail }): Parameters<LogsParam>,
    ) -> Result<String, String> {
        self.service
            .container_logs(&id, tail.unwrap_or(100))
            .await
            .map_err(tool_err)
    }

    #[tool(description = "Start a stopped container")]
    async fn docker_start_container(
        &self,
        Parameters(IdParam { id }): Parameters<IdParam>,
    ) -> Result<String, String> {
        self.service.start_container(&id).await.map_err(tool_err)
    }

    #[tool(description = "Stop a running container")]
    async fn docker_stop_container(
        &self,
        Parameters(StopParam { id, timeout_secs }): Parameters<StopParam>,
    ) -> Result<String, String> {
        self.service
            .stop_container(&id, timeout_secs.unwrap_or(10))
            .await
            .map_err(tool_err)
    }

    #[tool(description = "Restart a container")]
    async fn docker_restart_container(
        &self,
        Parameters(StopParam { id, timeout_secs }): Parameters<StopParam>,
    ) -> Result<String, String> {
        self.service
            .restart_container(&id, timeout_secs.unwrap_or(10))
            .await
            .map_err(tool_err)
    }

    #[tool(description = "Remove a container")]
    async fn docker_remove_container(
        &self,
        Parameters(ForceParam { id, force }): Parameters<ForceParam>,
    ) -> Result<String, String> {
        self.service
            .remove_container(&id, force.unwrap_or(false))
            .await
            .map_err(tool_err)
    }

    #[tool(description = "Run a non-privileged container")]
    async fn docker_run_container(
        &self,
        Parameters(RunParam {
            image,
            name,
            command,
            env,
            detach,
        }): Parameters<RunParam>,
    ) -> Result<String, String> {
        self.service
            .run_container(
                &image,
                name.as_deref(),
                command.unwrap_or_default(),
                env.unwrap_or_default(),
                detach.unwrap_or(true),
            )
            .await
            .map_err(tool_err)
    }

    #[tool(description = "List Docker images")]
    async fn docker_list_images(&self) -> Result<String, String> {
        self.service.list_images().await.map_err(tool_err)
    }

    #[tool(description = "Inspect an image")]
    async fn docker_inspect_image(
        &self,
        Parameters(NameParam { name, force: _ }): Parameters<NameParam>,
    ) -> Result<String, String> {
        self.service.inspect_image(&name).await.map_err(tool_err)
    }

    #[tool(description = "Pull an image from a registry")]
    async fn docker_pull_image(
        &self,
        Parameters(NameParam { name, force: _ }): Parameters<NameParam>,
    ) -> Result<String, String> {
        self.service.pull_image(&name).await.map_err(tool_err)
    }

    #[tool(description = "Remove an image")]
    async fn docker_remove_image(
        &self,
        Parameters(NameParam { name, force }): Parameters<NameParam>,
    ) -> Result<String, String> {
        self.service
            .remove_image(&name, force.unwrap_or(false))
            .await
            .map_err(tool_err)
    }

    #[tool(description = "List Docker networks")]
    async fn docker_list_networks(&self) -> Result<String, String> {
        self.service.list_networks().await.map_err(tool_err)
    }

    #[tool(description = "List Docker volumes")]
    async fn docker_list_volumes(&self) -> Result<String, String> {
        self.service.list_volumes().await.map_err(tool_err)
    }

    #[tool(description = "List compose project services (docker compose ps)")]
    async fn docker_compose_ps(
        &self,
        Parameters(ComposeParam { compose_file }): Parameters<ComposeParam>,
    ) -> Result<String, String> {
        self.service
            .compose_ps(&compose_file)
            .await
            .map_err(tool_err)
    }

    #[tool(description = "Validate and render compose configuration")]
    async fn docker_compose_config(
        &self,
        Parameters(ComposeParam { compose_file }): Parameters<ComposeParam>,
    ) -> Result<String, String> {
        self.service
            .compose_config(&compose_file)
            .await
            .map_err(tool_err)
    }
}

pub async fn run_stdio_server(server: DockerMcpServer) -> anyhow::Result<()> {
    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::audit::{AuditLogger, MemoryAuditSink};
    use crate::auth::ToolScope;
    use crate::backend::{MockBackend, SystemInfo};
    use crate::config::Config;

    use super::*;

    fn make_server(scopes: &[ToolScope]) -> (DockerMcpServer, Arc<MockBackend>) {
        let sink = Arc::new(MemoryAuditSink::new());
        let audit = AuditLogger::new(sink, "test", "sess");
        let backend = Arc::new(MockBackend::new());
        let config = Config::for_test(scopes);
        let service = Arc::new(DockerService::new(backend.clone(), config, audit));
        (DockerMcpServer::new(service), backend)
    }

    #[tokio::test]
    async fn mcp_tool_system_info_integration() {
        let (server, mock) = make_server(&[ToolScope::Read]);
        *mock.system_info.lock().unwrap() = Some(SystemInfo {
            id: "x".into(),
            containers: 0,
            images: 0,
            driver: "overlay2".into(),
            operating_system: "Linux".into(),
            architecture: "amd64".into(),
        });
        let out = server.docker_system_info().await.unwrap();
        assert!(out.contains("x"));
    }

    #[tokio::test]
    async fn all_mcp_tools_callable() {
        use crate::backend::{ContainerSummary, ImageSummary, NetworkSummary, VolumeSummary};

        let (server, mock) = make_server(&[ToolScope::Read, ToolScope::Write, ToolScope::Compose]);
        *mock.system_info.lock().unwrap() = Some(SystemInfo {
            id: "x".into(),
            containers: 0,
            images: 0,
            driver: "overlay2".into(),
            operating_system: "Linux".into(),
            architecture: "amd64".into(),
        });
        mock.containers.lock().unwrap().push(ContainerSummary {
            id: "c1".into(),
            name: "web".into(),
            image: "nginx".into(),
            state: "running".into(),
            status: "Up".into(),
        });
        mock.images.lock().unwrap().push(ImageSummary {
            id: "i1".into(),
            tags: vec!["nginx".into()],
            size: 1,
            created: 1,
        });
        mock.networks.lock().unwrap().push(NetworkSummary {
            id: "n1".into(),
            name: "bridge".into(),
            driver: "bridge".into(),
        });
        mock.volumes.lock().unwrap().push(VolumeSummary {
            name: "v1".into(),
            driver: "local".into(),
            mountpoint: "/m".into(),
        });
        *mock.inspect_container.lock().unwrap() = Some("{}".into());
        *mock.inspect_image.lock().unwrap() = Some("{}".into());
        *mock.logs.lock().unwrap() = Some("log".into());
        *mock.action_result.lock().unwrap() = "ok".into();
        *mock.compose_ps.lock().unwrap() = Some("[]".into());
        *mock.compose_config.lock().unwrap() = Some("x: 1".into());

        assert!(server.docker_system_info().await.is_ok());
        assert!(server
            .docker_list_containers(Parameters(AllParam { all: Some(true) }))
            .await
            .is_ok());
        assert!(server
            .docker_inspect_container(Parameters(IdParam { id: "c1".into() }))
            .await
            .is_ok());
        assert!(server
            .docker_container_logs(Parameters(LogsParam {
                id: "c1".into(),
                tail: Some(10),
            }))
            .await
            .is_ok());
        assert!(server
            .docker_start_container(Parameters(IdParam { id: "c1".into() }))
            .await
            .is_ok());
        assert!(server
            .docker_stop_container(Parameters(StopParam {
                id: "c1".into(),
                timeout_secs: None,
            }))
            .await
            .is_ok());
        assert!(server
            .docker_restart_container(Parameters(StopParam {
                id: "c1".into(),
                timeout_secs: Some(5),
            }))
            .await
            .is_ok());
        assert!(server
            .docker_remove_container(Parameters(ForceParam {
                id: "c1".into(),
                force: Some(false),
            }))
            .await
            .is_ok());
        assert!(server
            .docker_run_container(Parameters(RunParam {
                image: "alpine".into(),
                name: Some("t".into()),
                command: Some(vec!["echo".into()]),
                env: None,
                detach: Some(true),
            }))
            .await
            .is_ok());
        assert!(server.docker_list_images().await.is_ok());
        assert!(server
            .docker_inspect_image(Parameters(NameParam {
                name: "nginx".into(),
                force: None,
            }))
            .await
            .is_ok());
        assert!(server
            .docker_pull_image(Parameters(NameParam {
                name: "nginx".into(),
                force: None,
            }))
            .await
            .is_ok());
        assert!(server
            .docker_remove_image(Parameters(NameParam {
                name: "nginx".into(),
                force: Some(true),
            }))
            .await
            .is_ok());
        assert!(server.docker_list_networks().await.is_ok());
        assert!(server.docker_list_volumes().await.is_ok());
        assert!(server
            .docker_compose_ps(Parameters(ComposeParam {
                compose_file: "compose.yaml".into(),
            }))
            .await
            .is_ok());
        assert!(server
            .docker_compose_config(Parameters(ComposeParam {
                compose_file: "compose.yaml".into(),
            }))
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn tool_err_maps_validation_failure() {
        let (server, _) = make_server(&[ToolScope::Read]);
        let err = server
            .docker_inspect_container(Parameters(IdParam {
                id: "bad id".into(),
            }))
            .await
            .unwrap_err();
        assert!(err.contains("validation"));
    }
}
