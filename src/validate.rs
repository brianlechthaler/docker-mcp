use crate::error::DockerMcpError;

const MAX_NAME_LEN: usize = 128;
const MAX_PATH_LEN: usize = 4096;
const MAX_ENV_KEY_LEN: usize = 256;
const MAX_ENV_VALUE_LEN: usize = 4096;
const MAX_CMD_ARGS: usize = 32;
const MAX_ARG_LEN: usize = 512;

/// Validates Docker container or image reference names.
pub fn validate_name(name: &str, field: &str) -> Result<(), DockerMcpError> {
    if name.is_empty() {
        return Err(DockerMcpError::validation(format!(
            "{field} must not be empty"
        )));
    }
    if name.len() > MAX_NAME_LEN {
        return Err(DockerMcpError::validation(format!(
            "{field} exceeds maximum length of {MAX_NAME_LEN}"
        )));
    }
    let valid = name.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/' || c == ':'
    });
    if !valid {
        return Err(DockerMcpError::validation(format!(
            "{field} contains invalid characters"
        )));
    }
    Ok(())
}

/// Validates filesystem paths used for compose files.
pub fn validate_compose_path(path: &str) -> Result<(), DockerMcpError> {
    if path.is_empty() {
        return Err(DockerMcpError::validation("path must not be empty"));
    }
    if path.len() > MAX_PATH_LEN {
        return Err(DockerMcpError::validation("path exceeds maximum length"));
    }
    if path.contains('\0') || path.contains("..") {
        return Err(DockerMcpError::validation(
            "path contains forbidden sequences",
        ));
    }
    Ok(())
}

/// Validates optional command arguments for container run.
pub fn validate_command_args(args: &[String]) -> Result<(), DockerMcpError> {
    if args.len() > MAX_CMD_ARGS {
        return Err(DockerMcpError::validation(format!(
            "command exceeds maximum of {MAX_CMD_ARGS} arguments"
        )));
    }
    for arg in args {
        if arg.len() > MAX_ARG_LEN {
            return Err(DockerMcpError::validation("command argument too long"));
        }
        if arg.contains('\0') {
            return Err(DockerMcpError::validation(
                "command argument contains null byte",
            ));
        }
    }
    Ok(())
}

/// Validates environment variable map for container run.
pub fn validate_env(env: &[(String, String)]) -> Result<(), DockerMcpError> {
    for (key, value) in env {
        if key.is_empty() || key.len() > MAX_ENV_KEY_LEN {
            return Err(DockerMcpError::validation(
                "invalid environment variable key",
            ));
        }
        if value.len() > MAX_ENV_VALUE_LEN {
            return Err(DockerMcpError::validation(
                "environment variable value too long",
            ));
        }
        if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(DockerMcpError::validation(
                "environment key contains invalid characters",
            ));
        }
    }
    Ok(())
}

/// Validates tail line count for log retrieval.
pub fn validate_log_tail(tail: u64) -> Result<(), DockerMcpError> {
    if tail == 0 || tail > 10_000 {
        return Err(DockerMcpError::validation(
            "tail must be between 1 and 10000",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_names() {
        assert!(validate_name("my-container_1", "name").is_ok());
        assert!(validate_name("nginx:latest", "image").is_ok());
        assert!(validate_name("ghcr.io/org/app:v1", "image").is_ok());
    }

    #[test]
    fn rejects_empty_name() {
        assert_eq!(
            validate_name("", "container"),
            Err(DockerMcpError::validation("container must not be empty"))
        );
    }

    #[test]
    fn rejects_long_name() {
        let long = "a".repeat(129);
        assert!(validate_name(&long, "name").is_err());
    }

    #[test]
    fn rejects_invalid_characters() {
        assert!(validate_name("bad name", "name").is_err());
        assert!(validate_name("bad;name", "name").is_err());
    }

    #[test]
    fn compose_path_validation() {
        assert!(validate_compose_path("compose.yaml").is_ok());
        assert!(validate_compose_path("../etc/passwd").is_err());
        assert!(validate_compose_path("").is_err());
    }

    #[test]
    fn command_args_validation() {
        assert!(validate_command_args(&["echo".into(), "hi".into()]).is_ok());
        let many: Vec<String> = (0..33).map(|i| i.to_string()).collect();
        assert!(validate_command_args(&many).is_err());
        assert!(validate_command_args(&["ok\0bad".into()]).is_err());
    }

    #[test]
    fn env_validation() {
        assert!(validate_env(&[("FOO".into(), "bar".into())]).is_ok());
        assert!(validate_env(&[("bad-key".into(), "v".into())]).is_err());
        assert!(validate_env(&[("".into(), "v".into())]).is_err());
    }

    #[test]
    fn log_tail_validation() {
        assert!(validate_log_tail(100).is_ok());
        assert!(validate_log_tail(0).is_err());
        assert!(validate_log_tail(10_001).is_err());
    }

    #[test]
    fn rejects_path_too_long() {
        let long = "a".repeat(4097);
        assert!(validate_compose_path(&long).is_err());
    }

    #[test]
    fn rejects_arg_too_long() {
        let long = "a".repeat(513);
        assert!(validate_command_args(&[long]).is_err());
    }

    #[test]
    fn rejects_env_value_too_long() {
        let long = "a".repeat(4097);
        assert!(validate_env(&[("K".into(), long)]).is_err());
    }

    #[test]
    fn rejects_null_in_path() {
        assert!(validate_compose_path("bad\0path").is_err());
    }
}
