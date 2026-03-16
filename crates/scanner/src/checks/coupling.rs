use std::path::Path;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::context::FileContext;
use crate::issue::{Issue, Severity};

static SUPER_SIBLING_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*use\s+super::\w+").unwrap()
});
static PUB_SUPER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bpub\(super\)").unwrap()
});

/// Check for sibling coupling via `use super::` and `pub(super)`.
pub fn check(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    if file_ctx.is_test_file || file_ctx.is_mother_file {
        return;
    }

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }

        if SUPER_SIBLING_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Warning,
                "rust/modules/no-sibling-coupling",
                "use super::sibling — route through parent module or extract to shared/",
            ));
        }

        if PUB_SUPER_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Warning,
                "rust/modules/no-pub-super",
                "pub(super) creates tight coupling — use pub(crate) or restructure",
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Language;

    fn child_ctx() -> FileContext {
        FileContext { language: Language::Rust, is_test_file: false, is_mother_file: false, is_definition_file: false }
    }
    fn mother_ctx() -> FileContext {
        FileContext { language: Language::Rust, is_test_file: false, is_mother_file: true, is_definition_file: false }
    }

    #[test]
    fn catches_super_import() {
        let mut issues = Vec::new();
        check(&child_ctx(), &["use super::sibling;"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn skips_mother_file() {
        let mut issues = Vec::new();
        check(&mother_ctx(), &["use super::sibling;"], &Config::default(), &mut issues, Path::new("mod.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn catches_pub_super() {
        let mut issues = Vec::new();
        check(&child_ctx(), &["pub(super) fn foo() {}"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
    }
}
