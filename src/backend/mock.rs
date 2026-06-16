use std::sync::Mutex;

use async_trait::async_trait;

use super::{
    ContainerSummary, DockerBackend, ImageSummary, NetworkSummary, RunSpec, SystemInfo,
    VolumeSummary,
};
use crate::error::DockerMcpError;

#[derive(Default)]
pub struct MockBackend {
    pub system_info: Mutex<Option<SystemInfo>>,
    pub containers: Mutex<Vec<ContainerSummary>>,
    pub images: Mutex<Vec<ImageSummary>>,
    pub networks: Mutex<Vec<NetworkSummary>>,
    pub volumes: Mutex<Vec<VolumeSummary>>,
    pub inspect_container: Mutex<Option<String>>,
    pub inspect_image: Mutex<Option<String>>,
    pub logs: Mutex<Option<String>>,
    pub action_result: Mutex<String>,
    pub compose_ps: Mutex<Option<String>>,
    pub compose_config: Mutex<Option<String>>,
    pub fail_with: Mutex<Option<DockerMcpError>>,
}

impl MockBackend {
    pub fn new() -> Self {
        Self {
            action_result: Mutex::new("ok".into()),
            ..Default::default()
        }
    }

    fn check_fail(&self) -> Result<(), DockerMcpError> {
        if let Some(err) = self.fail_with.lock().unwrap().clone() {
            return Err(err);
        }
        Ok(())
    }
}

#[async_trait]
impl DockerBackend for MockBackend {
    async fn system_info(&self) -> Result<SystemInfo, DockerMcpError> {
        self.check_fail()?;
        self.system_info
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| DockerMcpError::docker("no mock system info"))
    }

    async fn list_containers(&self, _all: bool) -> Result<Vec<ContainerSummary>, DockerMcpError> {
        self.check_fail()?;
        Ok(self.containers.lock().unwrap().clone())
    }

    async fn inspect_container(&self, _id: &str) -> Result<String, DockerMcpError> {
        self.check_fail()?;
        self.inspect_container
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| DockerMcpError::docker("no mock inspect"))
    }

    async fn container_logs(&self, _id: &str, _tail: u64) -> Result<String, DockerMcpError> {
        self.check_fail()?;
        self.logs
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| DockerMcpError::docker("no mock logs"))
    }

    async fn start_container(&self, _id: &str) -> Result<String, DockerMcpError> {
        self.check_fail()?;
        Ok(self.action_result.lock().unwrap().clone())
    }

    async fn stop_container(
        &self,
        _id: &str,
        _timeout_secs: u64,
    ) -> Result<String, DockerMcpError> {
        self.check_fail()?;
        Ok(self.action_result.lock().unwrap().clone())
    }

    async fn restart_container(
        &self,
        _id: &str,
        _timeout_secs: u64,
    ) -> Result<String, DockerMcpError> {
        self.check_fail()?;
        Ok(self.action_result.lock().unwrap().clone())
    }

    async fn remove_container(&self, _id: &str, _force: bool) -> Result<String, DockerMcpError> {
        self.check_fail()?;
        Ok(self.action_result.lock().unwrap().clone())
    }

    async fn run_container(&self, _spec: RunSpec) -> Result<String, DockerMcpError> {
        self.check_fail()?;
        Ok(self.action_result.lock().unwrap().clone())
    }

    async fn list_images(&self) -> Result<Vec<ImageSummary>, DockerMcpError> {
        self.check_fail()?;
        Ok(self.images.lock().unwrap().clone())
    }

    async fn inspect_image(&self, _name: &str) -> Result<String, DockerMcpError> {
        self.check_fail()?;
        self.inspect_image
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| DockerMcpError::docker("no mock image inspect"))
    }

    async fn pull_image(&self, _name: &str) -> Result<String, DockerMcpError> {
        self.check_fail()?;
        Ok(self.action_result.lock().unwrap().clone())
    }

    async fn remove_image(&self, _name: &str, _force: bool) -> Result<String, DockerMcpError> {
        self.check_fail()?;
        Ok(self.action_result.lock().unwrap().clone())
    }

    async fn list_networks(&self) -> Result<Vec<NetworkSummary>, DockerMcpError> {
        self.check_fail()?;
        Ok(self.networks.lock().unwrap().clone())
    }

    async fn list_volumes(&self) -> Result<Vec<VolumeSummary>, DockerMcpError> {
        self.check_fail()?;
        Ok(self.volumes.lock().unwrap().clone())
    }

    async fn compose_ps(&self, _compose_file: &str) -> Result<String, DockerMcpError> {
        self.check_fail()?;
        self.compose_ps
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| DockerMcpError::docker("no mock compose ps"))
    }

    async fn compose_config(&self, _compose_file: &str) -> Result<String, DockerMcpError> {
        self.check_fail()?;
        self.compose_config
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| DockerMcpError::docker("no mock compose config"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_returns_configured_data() {
        let mock = MockBackend::new();
        *mock.system_info.lock().unwrap() = Some(SystemInfo {
            id: "abc".into(),
            containers: 1,
            images: 2,
            driver: "overlay2".into(),
            operating_system: "Linux".into(),
            architecture: "amd64".into(),
        });
        let info = mock.system_info().await.unwrap();
        assert_eq!(info.id, "abc");
    }

    #[tokio::test]
    async fn mock_propagates_failures() {
        let mock = MockBackend::new();
        *mock.fail_with.lock().unwrap() = Some(DockerMcpError::docker("fail"));
        assert!(mock.list_images().await.is_err());
    }

    #[tokio::test]
    async fn mock_covers_all_trait_methods() {
        let mock = MockBackend::new();
        *mock.system_info.lock().unwrap() = Some(SystemInfo {
            id: "1".into(),
            containers: 0,
            images: 0,
            driver: "d".into(),
            operating_system: "o".into(),
            architecture: "a".into(),
        });
        *mock.inspect_container.lock().unwrap() = Some("{}".into());
        *mock.inspect_image.lock().unwrap() = Some("{}".into());
        *mock.logs.lock().unwrap() = Some("l".into());
        *mock.compose_ps.lock().unwrap() = Some("[]".into());
        *mock.compose_config.lock().unwrap() = Some("x".into());

        mock.system_info().await.unwrap();
        mock.list_containers(true).await.unwrap();
        mock.inspect_container("c").await.unwrap();
        mock.container_logs("c", 10).await.unwrap();
        mock.start_container("c").await.unwrap();
        mock.stop_container("c", 1).await.unwrap();
        mock.restart_container("c", 1).await.unwrap();
        mock.remove_container("c", false).await.unwrap();
        mock.run_container(RunSpec {
            image: "i".into(),
            name: None,
            command: vec![],
            env: vec![],
            detach: true,
        })
        .await
        .unwrap();
        mock.list_images().await.unwrap();
        mock.inspect_image("i").await.unwrap();
        mock.pull_image("i").await.unwrap();
        mock.remove_image("i", false).await.unwrap();
        mock.list_networks().await.unwrap();
        mock.list_volumes().await.unwrap();
        mock.compose_ps("c.yaml").await.unwrap();
        mock.compose_config("c.yaml").await.unwrap();
    }

    #[tokio::test]
    async fn mock_errors_when_data_missing() {
        let mock = MockBackend::new();
        assert!(mock.system_info().await.is_err());
        assert!(mock.inspect_container("c").await.is_err());
        assert!(mock.container_logs("c", 1).await.is_err());
        assert!(mock.inspect_image("i").await.is_err());
        assert!(mock.compose_ps("c.yaml").await.is_err());
        assert!(mock.compose_config("c.yaml").await.is_err());
    }
}
