use std::path::Path;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::context::{self, FileContext};
use crate::issue::{Issue, Severity};

static MATCH_ARM_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#""([^"]{2,}?)"\s*=>"#).unwrap()
});
static EQUALITY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"[!=]=\s*"([^"]{2,}?)""#).unwrap()
});
/// Strings that look like messages/paths rather than identifiers.
static SKIP_VALUE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:\s|[/\\.]|[A-Z]{2}|[?!,])").unwrap()
});

/// Check for stringly-typed match arms and comparisons.
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

        for cap in MATCH_ARM_RE.captures_iter(line) {
            let val = &cap[1];
            if SKIP_VALUE_RE.is_match(val) {
                continue;
            }
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "rust/types/no-string-match",
                format!("stringly-typed match \"{val}\" — define an enum variant instead"),
            ));
        }

        for cap in EQUALITY_RE.captures_iter(line) {
            let val = &cap[1];
            if SKIP_VALUE_RE.is_match(val) {
                continue;
            }
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "rust/types/no-string-compare",
                format!("stringly-typed comparison \"{val}\" — discriminators must be enums or named consts"),
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
    fn catches_match_arm() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &[r#"    "active" => do_thing(),"#], &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].rule_id.contains("string-match"));
    }

    #[test]
    fn catches_equality() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &[r#"if state == "running" {"#], &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn skips_message_strings() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &[r#"    "file not found" => bail!(),"#], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_path_strings() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &[r#"if ext == "src/main.rs" {"#], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }
}
