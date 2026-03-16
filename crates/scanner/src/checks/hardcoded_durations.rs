use std::path::Path;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::context::{self, FileContext};
use crate::issue::{Issue, Severity};

static DURATION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Duration::(?:from_secs|from_millis|from_nanos|from_micros|new)\s*\(\s*(\d+)").unwrap()
});

/// Check for hardcoded Duration literals outside const/static.
pub fn check(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    if file_ctx.is_test_file {
        return;
    }

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        if context::is_const_def(line) {
            continue;
        }
        if context::is_test_context(lines, line_num) {
            continue;
        }

        for cap in DURATION_RE.captures_iter(line) {
            let val = &cap[1];
            if val == "0" {
                continue;
            }
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "rust/constants/no-hardcoded-duration",
                format!("hardcoded Duration({val}) — use a named constant from state/"),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Language;

    fn prod_ctx() -> FileContext {
        FileContext { language: Language::Rust, is_test_file: false, is_mother_file: false, is_definition_file: false }
    }

    #[test]
    fn catches_duration_literal() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["let d = Duration::from_secs(30);"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn allows_zero_duration() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["let d = Duration::from_secs(0);"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn allows_const_duration() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["const TIMEOUT: Duration = Duration::from_secs(30);"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }
}
