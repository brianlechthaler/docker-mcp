use sha2::{Digest, Sha256};

const DEFAULT_MAX_BYTES: usize = 1_048_576;

/// Truncates output to a maximum byte size and redacts common secret patterns.
pub fn sanitize_output(input: &str, max_bytes: usize) -> String {
    let redacted = redact_secrets(input);
    truncate_bytes(&redacted, max_bytes)
}

/// Computes SHA-256 fingerprint of parameters for audit logging.
pub fn params_fingerprint(params: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(params.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

fn redact_secrets(input: &str) -> String {
    let mut out = input.to_string();
    for pattern in [
        "password=",
        "token=",
        "secret=",
        "api_key=",
        "Authorization:",
    ] {
        if let Some(idx) = out.to_lowercase().find(&pattern.to_lowercase()) {
            let line_end = out[idx..].find('\n').map(|i| idx + i).unwrap_or(out.len());
            let prefix = &out[..idx + pattern.len()];
            out = format!("{prefix}[REDACTED]{}", &out[line_end..]);
        }
    }
    out
}

fn truncate_bytes(input: &str, max_bytes: usize) -> String {
    if input.len() <= max_bytes {
        return input.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !input.is_char_boundary(end) {
        end -= 1;
    }
    format!(
        "{}… [truncated, {} bytes total]",
        &input[..end],
        input.len()
    )
}

pub fn default_max_bytes() -> usize {
    DEFAULT_MAX_BYTES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_large_output() {
        let big = "x".repeat(100);
        let out = sanitize_output(&big, 50);
        assert!(out.contains("truncated"));
        assert!(out.len() < 100);
    }

    #[test]
    fn leaves_small_output_unchanged() {
        assert_eq!(sanitize_output("hello", 100), "hello");
    }

    #[test]
    fn redacts_password_patterns() {
        let input = "user=admin\npassword=supersecret\nok=true";
        let out = sanitize_output(input, 1000);
        assert!(out.contains("[REDACTED]"));
        assert!(!out.contains("supersecret"));
    }

    #[test]
    fn fingerprint_is_stable() {
        let a = params_fingerprint(r#"{"id":"abc"}"#);
        let b = params_fingerprint(r#"{"id":"abc"}"#);
        assert_eq!(a, b);
        assert!(a.starts_with("sha256:"));
    }

    #[test]
    fn default_max_bytes_constant() {
        assert_eq!(default_max_bytes(), 1_048_576);
    }

    #[test]
    fn truncate_respects_char_boundaries() {
        let s = "hello 🌍 world";
        let out = truncate_bytes(s, 8);
        assert!(out.contains('…'));
    }
}
