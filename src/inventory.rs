use std::collections::HashMap;

use sha2::{Digest, Sha256};

/// Tool definition metadata for inventory pinning (NSA MCP requirement #5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolDefinition {
    pub name: String,
    pub version: String,
    pub description: String,
    pub scope: String,
}

pub const SERVER_VERSION: &str = "1.0.0";

pub fn registered_tools() -> Vec<ToolDefinition> {
    vec![
        tool(
            "docker_system_info@1",
            "docker:read",
            "Docker daemon system information",
        ),
        tool(
            "docker_list_containers@1",
            "docker:read",
            "List Docker containers",
        ),
        tool(
            "docker_inspect_container@1",
            "docker:read",
            "Inspect a container by ID or name",
        ),
        tool(
            "docker_container_logs@1",
            "docker:read",
            "Fetch container logs with tail limit",
        ),
        tool("docker_list_images@1", "docker:read", "List Docker images"),
        tool(
            "docker_inspect_image@1",
            "docker:read",
            "Inspect an image by name or ID",
        ),
        tool(
            "docker_list_networks@1",
            "docker:read",
            "List Docker networks",
        ),
        tool(
            "docker_list_volumes@1",
            "docker:read",
            "List Docker volumes",
        ),
        tool(
            "docker_start_container@1",
            "docker:write",
            "Start a stopped container",
        ),
        tool(
            "docker_stop_container@1",
            "docker:write",
            "Stop a running container",
        ),
        tool(
            "docker_restart_container@1",
            "docker:write",
            "Restart a container",
        ),
        tool(
            "docker_remove_container@1",
            "docker:write",
            "Remove a container",
        ),
        tool(
            "docker_run_container@1",
            "docker:write",
            "Run a container (non-privileged)",
        ),
        tool(
            "docker_pull_image@1",
            "docker:write",
            "Pull an image from a registry",
        ),
        tool("docker_remove_image@1", "docker:write", "Remove an image"),
        tool(
            "docker_compose_ps@1",
            "docker:compose",
            "List compose project services",
        ),
        tool(
            "docker_compose_config@1",
            "docker:compose",
            "Validate and render compose config",
        ),
    ]
}

fn tool(name: &str, scope: &str, description: &str) -> ToolDefinition {
    ToolDefinition {
        name: name.into(),
        version: "1".into(),
        description: description.into(),
        scope: scope.into(),
    }
}

pub fn inventory_fingerprint() -> String {
    let tools = registered_tools();
    let mut hasher = Sha256::new();
    for t in tools {
        hasher.update(t.name.as_bytes());
        hasher.update(t.version.as_bytes());
        hasher.update(t.description.as_bytes());
        hasher.update(t.scope.as_bytes());
    }
    format!("sha256:{:x}", hasher.finalize())
}

pub fn inventory_map() -> HashMap<String, ToolDefinition> {
    registered_tools()
        .into_iter()
        .map(|t| (t.name.clone(), t))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_seventeen_tools() {
        assert_eq!(registered_tools().len(), 17);
    }

    #[test]
    fn fingerprint_is_stable() {
        assert_eq!(inventory_fingerprint(), inventory_fingerprint());
    }

    #[test]
    fn inventory_map_has_all_tools() {
        let map = inventory_map();
        assert!(map.contains_key("docker_system_info@1"));
        assert_eq!(map["docker_run_container@1"].scope, "docker:write");
    }
}
