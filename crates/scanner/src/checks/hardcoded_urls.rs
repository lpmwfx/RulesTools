use std::path::Path;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::context::{self, FileContext};
use crate::issue::{Issue, Severity};

static URL_LITERAL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#""(https?://[^"]+)""#).unwrap()
});

/// Check for hardcoded URL literals outside const/static.
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

        for cap in URL_LITERAL_RE.captures_iter(line) {
            let url = &cap[1];
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "rust/constants/no-hardcoded-url",
                format!("hardcoded URL \"{url}\" — use a named constant"),
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
    fn catches_url() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &[r#"let u = "https://api.example.com/v1";"#], &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn allows_const_url() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &[r#"const API_URL: &str = "https://api.example.com/v1";"#], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }
}
