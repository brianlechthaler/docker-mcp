use std::collections::HashSet;

use crate::error::DockerMcpError;

/// Scope required to invoke a Docker MCP tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolScope {
    Read,
    Write,
    Compose,
}

impl ToolScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "docker:read",
            Self::Write => "docker:write",
            Self::Compose => "docker:compose",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Authorizer {
    allowed: HashSet<ToolScope>,
    deny_all: bool,
}

impl Authorizer {
    pub fn from_scopes(scopes: &[ToolScope]) -> Self {
        Self {
            allowed: scopes.iter().copied().collect(),
            deny_all: false,
        }
    }

    pub fn deny_all() -> Self {
        Self {
            allowed: HashSet::new(),
            deny_all: true,
        }
    }

    pub fn all() -> Self {
        Self::from_scopes(&[ToolScope::Read, ToolScope::Write, ToolScope::Compose])
    }

    pub fn authorize(&self, scope: ToolScope) -> Result<(), DockerMcpError> {
        if self.deny_all || !self.allowed.contains(&scope) {
            return Err(DockerMcpError::authorization(format!(
                "missing required scope {}",
                scope.as_str()
            )));
        }
        Ok(())
    }
}

pub fn parse_scopes(raw: &str) -> Vec<ToolScope> {
    raw.split(',')
        .map(str::trim)
        .filter_map(|s| match s {
            "docker:read" => Some(ToolScope::Read),
            "docker:write" => Some(ToolScope::Write),
            "docker:compose" => Some(ToolScope::Compose),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_configured_scope() {
        let auth = Authorizer::from_scopes(&[ToolScope::Read]);
        assert!(auth.authorize(ToolScope::Read).is_ok());
        assert!(auth.authorize(ToolScope::Write).is_err());
    }

    #[test]
    fn deny_all_blocks_everything() {
        let auth = Authorizer::deny_all();
        assert!(auth.authorize(ToolScope::Read).is_err());
    }

    #[test]
    fn all_grants_every_scope() {
        let auth = Authorizer::all();
        assert!(auth.authorize(ToolScope::Compose).is_ok());
    }

    #[test]
    fn parse_scopes_from_env_string() {
        let scopes = parse_scopes("docker:read, docker:write");
        assert_eq!(scopes, vec![ToolScope::Read, ToolScope::Write]);
    }

    #[test]
    fn scope_as_str() {
        assert_eq!(ToolScope::Read.as_str(), "docker:read");
        assert_eq!(ToolScope::Write.as_str(), "docker:write");
        assert_eq!(ToolScope::Compose.as_str(), "docker:compose");
    }
}
