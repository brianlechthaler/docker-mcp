use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::DockerMcpError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContainerSummary {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImageSummary {
    pub id: String,
    pub tags: Vec<String>,
    pub size: u64,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkSummary {
    pub id: String,
    pub name: String,
    pub driver: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VolumeSummary {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemInfo {
    pub id: String,
    pub containers: i64,
    pub images: i64,
    pub driver: String,
    pub operating_system: String,
    pub architecture: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunSpec {
    pub image: String,
    pub name: Option<String>,
    pub command: Vec<String>,
    pub env: Vec<(String, String)>,
    pub detach: bool,
}

#[async_trait]
pub trait DockerBackend: Send + Sync {
    async fn system_info(&self) -> Result<SystemInfo, DockerMcpError>;
    async fn list_containers(&self, all: bool) -> Result<Vec<ContainerSummary>, DockerMcpError>;
    async fn inspect_container(&self, id: &str) -> Result<String, DockerMcpError>;
    async fn container_logs(&self, id: &str, tail: u64) -> Result<String, DockerMcpError>;
    async fn start_container(&self, id: &str) -> Result<String, DockerMcpError>;
    async fn stop_container(&self, id: &str, timeout_secs: u64) -> Result<String, DockerMcpError>;
    async fn restart_container(
        &self,
        id: &str,
        timeout_secs: u64,
    ) -> Result<String, DockerMcpError>;
    async fn remove_container(&self, id: &str, force: bool) -> Result<String, DockerMcpError>;
    async fn run_container(&self, spec: RunSpec) -> Result<String, DockerMcpError>;
    async fn list_images(&self) -> Result<Vec<ImageSummary>, DockerMcpError>;
    async fn inspect_image(&self, name: &str) -> Result<String, DockerMcpError>;
    async fn pull_image(&self, name: &str) -> Result<String, DockerMcpError>;
    async fn remove_image(&self, name: &str, force: bool) -> Result<String, DockerMcpError>;
    async fn list_networks(&self) -> Result<Vec<NetworkSummary>, DockerMcpError>;
    async fn list_volumes(&self) -> Result<Vec<VolumeSummary>, DockerMcpError>;
    async fn compose_ps(&self, compose_file: &str) -> Result<String, DockerMcpError>;
    async fn compose_config(&self, compose_file: &str) -> Result<String, DockerMcpError>;
}

pub mod bollard_impl;
pub mod mock;

pub use bollard_impl::BollardBackend;
pub use mock::MockBackend;
