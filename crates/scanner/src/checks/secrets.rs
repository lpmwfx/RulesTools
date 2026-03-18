use std::path::Path;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::context::FileContext;
use crate::issue::{Issue, Severity};

static CREDENTIAL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)\b(password|passwd|pwd|api[-_]?key|apikey|secret[-_]?key|client[-_]?secret|access[-_]?token|auth[-_]?token|bearer[-_]?token|private[-_]?key|signing[-_]?key|database[-_]?url|db[-_]?password|aws[-_]?secret|aws[-_]?access|aws[-_]?key)\s*[=:]\s*["'][^"']{4,}["']"#).unwrap()
});

static PEM_KEY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"-----BEGIN\s+(RSA|EC|OPENSSH|PRIVATE)\s+PRIVATE KEY-----").unwrap()
});

/// Skip these directories entirely.
const SKIP_DIRS: &[&str] = &["test", "tests", "fixtures", "examples", "docs"];

/// Check for hardcoded secrets and credentials.
pub fn check(
    _file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    let path_str = path.to_string_lossy();
    if SKIP_DIRS.iter().any(|d| path_str.contains(d)) {
        return;
    }

    let mut in_test_module = false;
    let mut test_brace_depth: i32 = 0;

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Track #[cfg(test)] mod blocks — skip test data
        if trimmed == "#[cfg(test)]" || trimmed == "#[test]" {
            in_test_module = true;
        }
        if in_test_module {
            test_brace_depth += line.chars().filter(|&c| c == '{').count() as i32;
            test_brace_depth -= line.chars().filter(|&c| c == '}').count() as i32;
            if test_brace_depth > 0 || trimmed.starts_with("#[") {
                continue;
            }
            if test_brace_depth <= 0 {
                in_test_module = false;
                test_brace_depth = 0;
                continue;
            }
        }

        if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with('*') {
            continue;
        }

        if CREDENTIAL_RE.is_match(line) {
            issues.push(Issue::new(
                path,
                line_num + 1,
                1,
                Severity::Error,
                "global/secrets",
                "hardcoded credential — use environment variable or secret manager",
            ));
        }

        if PEM_KEY_RE.is_match(line) {
            issues.push(Issue::new(
                path,
                line_num + 1,
                1,
                Severity::Error,
                "global/secrets",
                "embedded private key — never commit keys to source",
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Language;

    fn make_ctx() -> FileContext {
        FileContext {
            language: Language::Rust,
            is_test_file: false,
            is_mother_file: false,
            is_definition_file: false,
        }
    }

    #[test]
    fn finds_password() {
        let mut issues = Vec::new();
        let lines = vec![r#"let password = "hunter2";"#];
        check(&make_ctx(), &lines, &Config::default(), &mut issues, Path::new("src/main.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("credential"));
    }

    #[test]
    fn finds_pem_key() {
        let mut issues = Vec::new();
        let lines = vec!["-----BEGIN RSA PRIVATE KEY-----"];
        check(&make_ctx(), &lines, &Config::default(), &mut issues, Path::new("src/main.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("private key"));
    }

    #[test]
    fn skips_comment_lines() {
        let mut issues = Vec::new();
        let lines = vec![r#"// password = "test1234""#];
        check(&make_ctx(), &lines, &Config::default(), &mut issues, Path::new("src/main.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_test_dirs() {
        let mut issues = Vec::new();
        let lines = vec![r#"let password = "hunter2";"#];
        check(&make_ctx(), &lines, &Config::default(), &mut issues, Path::new("tests/auth.rs"));
        assert!(issues.is_empty());
    }
}
