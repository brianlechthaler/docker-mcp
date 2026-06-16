use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum DockerMcpError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("authorization denied: {0}")]
    Authorization(String),
    #[error("docker error: {0}")]
    Docker(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl DockerMcpError {
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn authorization(msg: impl Into<String>) -> Self {
        Self::Authorization(msg.into())
    }

    pub fn docker(msg: impl Into<String>) -> Self {
        Self::Docker(msg.into())
    }
}

impl From<DockerMcpError> for String {
    fn from(value: DockerMcpError) -> Self {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_and_constructors() {
        let v = DockerMcpError::validation("bad input");
        assert_eq!(v.to_string(), "validation error: bad input");

        let a = DockerMcpError::authorization("no scope");
        assert_eq!(a.to_string(), "authorization denied: no scope");

        let d = DockerMcpError::docker("daemon down");
        assert_eq!(d.to_string(), "docker error: daemon down");

        let i = DockerMcpError::Internal("oops".into());
        assert_eq!(i.to_string(), "internal error: oops");
    }

    #[test]
    fn converts_to_string() {
        let err = DockerMcpError::validation("x");
        let s: String = err.into();
        assert_eq!(s, "validation error: x");
    }
}
