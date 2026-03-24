use std::path::Path;

/// Extract the bare binary name from an Exec= value.
///
/// Strips:
/// - "env" command prefix
/// - VAR=value environment assignments
/// - %f/%u/etc field codes (caller is expected to have removed them already, but we're robust)
/// - Surrounding quotes
pub fn parse_exec_binary(exec_raw: &str) -> &str {
    let mut s = exec_raw.trim();

    // Skip "env" command and VAR=value assignments (e.g. "env FOO=bar /usr/bin/app")
    loop {
        if s.starts_with("env ") { s = s[4..].trim(); continue; }
        if s.contains('=') && !s.contains(' ') { return ""; } // lone VAR=val, no binary
        let first = s.split_whitespace().next().unwrap_or("");
        if first.contains('=') { s = s[first.len()..].trim(); continue; }
        break;
    }

    // Take just the first token (the binary name/path)
    let binary = s.split_whitespace().next().unwrap_or("");
    // Strip surrounding quotes
    let binary = binary.trim_matches('"').trim_matches('\'');
    binary
}

/// Returns true if the binary named by `exec` is findable on PATH or exists as absolute path.
pub fn binary_exists(exec: &str) -> bool {
    if exec.is_empty() { return false; }
    // Absolute path
    if exec.starts_with('/') {
        return Path::new(exec).exists();
    }
    // Search PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            if Path::new(&format!("{}/{}", dir, exec)).exists() {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_binary() {
        assert_eq!(parse_exec_binary("firefox %u"), "firefox");
    }

    #[test]
    fn env_prefix_stripped() {
        assert_eq!(parse_exec_binary("env FOO=bar /usr/bin/app --arg"), "/usr/bin/app");
    }

    #[test]
    fn field_codes_handled() {
        // Caller strips field codes, but we should still return the binary cleanly
        assert_eq!(parse_exec_binary("/usr/bin/gimp"), "/usr/bin/gimp");
    }

    #[test]
    fn lone_var_assignment_returns_empty() {
        assert_eq!(parse_exec_binary("FOO=BAR"), "");
    }

    #[test]
    fn quoted_binary_strips_quotes() {
        // Single-token with surrounding quotes (no space inside)
        assert_eq!(parse_exec_binary("\"firefox\" %u"), "firefox");
    }

    #[test]
    fn single_quoted_binary() {
        assert_eq!(parse_exec_binary("'myapp' --flag"), "myapp");
    }

    #[test]
    fn empty_input() {
        assert_eq!(parse_exec_binary(""), "");
    }

    #[test]
    fn binary_exists_empty() {
        assert!(!binary_exists(""));
    }

    #[test]
    fn binary_exists_nonexistent_absolute() {
        assert!(!binary_exists("/nonexistent/path/to/binary"));
    }

    #[test]
    fn binary_exists_on_path() {
        // 'sh' should always be on PATH in a unix-like environment
        assert!(binary_exists("sh"));
    }
}
