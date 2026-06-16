use std::path::Path;

use async_trait::async_trait;
use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, LogsOptions, StartContainerOptions,
    StopContainerOptions,
};
use bollard::image::{CreateImageOptions, ListImagesOptions, RemoveImageOptions};
use bollard::models::{ContainerCreateResponse, HostConfig};
use bollard::network::ListNetworksOptions;
use bollard::volume::ListVolumesOptions;
use bollard::Docker;
use futures_util::StreamExt;
use tokio::process::Command;

use super::{
    ContainerSummary, DockerBackend, ImageSummary, NetworkSummary, RunSpec, SystemInfo,
    VolumeSummary,
};
use crate::error::DockerMcpError;
use crate::validate::validate_compose_path;

pub struct BollardBackend {
    docker: Docker,
}

impl BollardBackend {
    pub fn connect(socket: Option<&str>) -> Result<Self, DockerMcpError> {
        let docker = match socket {
            Some(path) => Docker::connect_with_unix(path, 120, bollard::API_DEFAULT_VERSION)
                .map_err(|e| DockerMcpError::docker(e.to_string()))?,
            None => Docker::connect_with_local_defaults()
                .map_err(|e| DockerMcpError::docker(e.to_string()))?,
        };
        Ok(Self { docker })
    }
}

async fn run_compose(compose_file: &str, args: &[&str]) -> Result<String, DockerMcpError> {
    validate_compose_path(compose_file)?;
    let mut cmd = Command::new("docker");
    cmd.arg("compose").arg("-f").arg(compose_file);
    for arg in args {
        cmd.arg(arg);
    }
    let output = cmd
        .output()
        .await
        .map_err(|e| DockerMcpError::docker(format!("compose command failed: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DockerMcpError::docker(format!("compose failed: {stderr}")));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[async_trait]
impl DockerBackend for BollardBackend {
    async fn system_info(&self) -> Result<SystemInfo, DockerMcpError> {
        let info = self
            .docker
            .info()
            .await
            .map_err(|e| DockerMcpError::docker(e.to_string()))?;
        Ok(SystemInfo {
            id: info.id.unwrap_or_default(),
            containers: info.containers.unwrap_or(0),
            images: info.images.unwrap_or(0),
            driver: info.driver.unwrap_or_default(),
            operating_system: info.operating_system.unwrap_or_default(),
            architecture: info.architecture.unwrap_or_default(),
        })
    }

    async fn list_containers(&self, all: bool) -> Result<Vec<ContainerSummary>, DockerMcpError> {
        let options = Some(ListContainersOptions::<String> {
            all,
            ..Default::default()
        });
        let list = self
            .docker
            .list_containers(options)
            .await
            .map_err(|e| DockerMcpError::docker(e.to_string()))?;
        Ok(list
            .into_iter()
            .map(|c| ContainerSummary {
                id: c.id.unwrap_or_default(),
                name: c
                    .names
                    .unwrap_or_default()
                    .first()
                    .cloned()
                    .unwrap_or_default()
                    .trim_start_matches('/')
                    .to_string(),
                image: c.image.unwrap_or_default(),
                state: c.state.unwrap_or_default(),
                status: c.status.unwrap_or_default(),
            })
            .collect())
    }

    async fn inspect_container(&self, id: &str) -> Result<String, DockerMcpError> {
        let inspect = self
            .docker
            .inspect_container(id, None)
            .await
            .map_err(|e| DockerMcpError::docker(e.to_string()))?;
        serde_json::to_string_pretty(&inspect).map_err(|e| DockerMcpError::Internal(e.to_string()))
    }

    async fn container_logs(&self, id: &str, tail: u64) -> Result<String, DockerMcpError> {
        let options = Some(LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: tail.to_string(),
            ..Default::default()
        });
        let mut stream = self.docker.logs(id, options);
        let mut output = String::new();
        while let Some(chunk) = stream.next().await {
            let log = chunk.map_err(|e| DockerMcpError::docker(e.to_string()))?;
            use bollard::container::LogOutput;
            match log {
                LogOutput::StdOut { message } | LogOutput::StdErr { message } => {
                    output.push_str(&String::from_utf8_lossy(&message));
                }
                LogOutput::StdIn { message } => {
                    output.push_str(&String::from_utf8_lossy(&message));
                }
                LogOutput::Console { message } => {
                    output.push_str(&String::from_utf8_lossy(&message));
                }
            }
        }
        Ok(output)
    }

    async fn start_container(&self, id: &str) -> Result<String, DockerMcpError> {
        self.docker
            .start_container(id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| DockerMcpError::docker(e.to_string()))?;
        Ok(format!("started container {id}"))
    }

    async fn stop_container(&self, id: &str, timeout_secs: u64) -> Result<String, DockerMcpError> {
        let options = Some(StopContainerOptions {
            t: timeout_secs as i64,
        });
        self.docker
            .stop_container(id, options)
            .await
            .map_err(|e| DockerMcpError::docker(e.to_string()))?;
        Ok(format!("stopped container {id}"))
    }

    async fn restart_container(
        &self,
        id: &str,
        timeout_secs: u64,
    ) -> Result<String, DockerMcpError> {
        use bollard::container::RestartContainerOptions;
        let options = Some(RestartContainerOptions {
            t: timeout_secs as isize,
        });
        self.docker
            .restart_container(id, options)
            .await
            .map_err(|e| DockerMcpError::docker(e.to_string()))?;
        Ok(format!("restarted container {id}"))
    }

    async fn remove_container(&self, id: &str, force: bool) -> Result<String, DockerMcpError> {
        use bollard::container::RemoveContainerOptions;
        let options = Some(RemoveContainerOptions {
            force,
            ..Default::default()
        });
        self.docker
            .remove_container(id, options)
            .await
            .map_err(|e| DockerMcpError::docker(e.to_string()))?;
        Ok(format!("removed container {id}"))
    }

    async fn run_container(&self, spec: RunSpec) -> Result<String, DockerMcpError> {
        let env: Vec<String> = spec.env.iter().map(|(k, v)| format!("{k}={v}")).collect();
        let host_config = HostConfig {
            auto_remove: Some(false),
            privileged: Some(false),
            network_mode: Some("bridge".into()),
            ..Default::default()
        };
        let config = Config {
            image: Some(spec.image.clone()),
            cmd: if spec.command.is_empty() {
                None
            } else {
                Some(spec.command)
            },
            env: if env.is_empty() { None } else { Some(env) },
            host_config: Some(host_config),
            ..Default::default()
        };
        let create_name = spec.name.clone();
        let options = create_name.map(|name| CreateContainerOptions {
            name,
            platform: None,
        });
        let response: ContainerCreateResponse = self
            .docker
            .create_container(options, config)
            .await
            .map_err(|e| DockerMcpError::docker(e.to_string()))?;
        let id = response.id;
        self.docker
            .start_container(&id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| DockerMcpError::docker(e.to_string()))?;
        if spec.detach {
            Ok(format!("started container {id}"))
        } else {
            Ok(format!("created and started container {id}"))
        }
    }

    async fn list_images(&self) -> Result<Vec<ImageSummary>, DockerMcpError> {
        let list = self
            .docker
            .list_images(None::<ListImagesOptions<String>>)
            .await
            .map_err(|e| DockerMcpError::docker(e.to_string()))?;
        Ok(list
            .into_iter()
            .map(|i| ImageSummary {
                id: i.id,
                tags: i.repo_tags,
                size: i.size as u64,
                created: i.created,
            })
            .collect())
    }

    async fn inspect_image(&self, name: &str) -> Result<String, DockerMcpError> {
        let inspect = self
            .docker
            .inspect_image(name)
            .await
            .map_err(|e| DockerMcpError::docker(e.to_string()))?;
        serde_json::to_string_pretty(&inspect).map_err(|e| DockerMcpError::Internal(e.to_string()))
    }

    async fn pull_image(&self, name: &str) -> Result<String, DockerMcpError> {
        let options = Some(CreateImageOptions {
            from_image: name,
            ..Default::default()
        });
        let mut stream = self.docker.create_image(options, None, None);
        while let Some(item) = stream.next().await {
            item.map_err(|e| DockerMcpError::docker(e.to_string()))?;
        }
        Ok(format!("pulled image {name}"))
    }

    async fn remove_image(&self, name: &str, force: bool) -> Result<String, DockerMcpError> {
        let options = Some(RemoveImageOptions {
            force,
            ..Default::default()
        });
        self.docker
            .remove_image(name, options, None)
            .await
            .map_err(|e| DockerMcpError::docker(e.to_string()))?;
        Ok(format!("removed image {name}"))
    }

    async fn list_networks(&self) -> Result<Vec<NetworkSummary>, DockerMcpError> {
        let list = self
            .docker
            .list_networks(None::<ListNetworksOptions<String>>)
            .await
            .map_err(|e| DockerMcpError::docker(e.to_string()))?;
        Ok(list
            .into_iter()
            .map(|n| NetworkSummary {
                id: n.id.unwrap_or_default(),
                name: n.name.unwrap_or_default(),
                driver: n.driver.unwrap_or_default(),
            })
            .collect())
    }

    async fn list_volumes(&self) -> Result<Vec<VolumeSummary>, DockerMcpError> {
        let list = self
            .docker
            .list_volumes(None::<ListVolumesOptions<String>>)
            .await
            .map_err(|e| DockerMcpError::docker(e.to_string()))?;
        Ok(list
            .volumes
            .unwrap_or_default()
            .into_iter()
            .map(|v| VolumeSummary {
                name: v.name,
                driver: v.driver,
                mountpoint: v.mountpoint,
            })
            .collect())
    }

    async fn compose_ps(&self, compose_file: &str) -> Result<String, DockerMcpError> {
        if !Path::new(compose_file).exists() {
            return Err(DockerMcpError::validation("compose file does not exist"));
        }
        run_compose(compose_file, &["ps", "--format", "json"]).await
    }

    async fn compose_config(&self, compose_file: &str) -> Result<String, DockerMcpError> {
        if !Path::new(compose_file).exists() {
            return Err(DockerMcpError::validation("compose file does not exist"));
        }
        run_compose(compose_file, &["config"]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_without_socket_uses_defaults() {
        // May fail if docker not available; that's ok for unit test isolation
        let _ = BollardBackend::connect(None);
    }

    #[test]
    fn connect_invalid_socket() {
        assert!(BollardBackend::connect(Some("/nonexistent/docker.sock")).is_err());
    }

    #[tokio::test]
    async fn run_compose_rejects_bad_path() {
        let err = run_compose("../bad", &["ps"]).await.unwrap_err();
        assert!(matches!(err, DockerMcpError::Validation(_)));
    }

    #[tokio::test]
    async fn run_compose_reports_failure() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("broken.yaml");
        std::fs::write(&path, "not: valid: compose").unwrap();
        let path_str = path.to_str().unwrap();
        let err = run_compose(path_str, &["config"]).await.unwrap_err();
        assert!(matches!(err, DockerMcpError::Docker(_)));
    }
}
