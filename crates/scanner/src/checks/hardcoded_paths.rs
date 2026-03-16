use std::path::Path;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::context::{self, FileContext};
use crate::issue::{Issue, Severity};

/// String literals ending in common file extensions.
static PATH_LITERAL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#""([^"]*\.(?:json|toml|yaml|yml|txt|png|svg|wasm|css|html|ico))""#).unwrap()
});

/// Check for hardcoded file path literals outside const/static.
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

        for cap in PATH_LITERAL_RE.captures_iter(line) {
            let val = &cap[1];
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "rust/constants/no-hardcoded-path",
                format!("hardcoded path \"{val}\" — use a named constant"),
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
    fn catches_json_path() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &[r#"let f = "config.json";"#], &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn allows_const_def() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &[r#"const CFG: &str = "config.json";"#], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }
}
