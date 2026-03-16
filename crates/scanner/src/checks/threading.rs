use std::path::Path;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::context::{self, FileContext};
use crate::issue::{Issue, Severity};

// Reserved for fire-and-forget spawn detection (phase 2)
#[allow(dead_code)]
static FIRE_FORGET_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\btokio::spawn\s*\(|\.spawn\s*\(").unwrap()
});
static ARC_RC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:Arc|Rc)::(?:new|clone)\b").unwrap()
});
static ARC_RC_COMMENT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"//.*(?:Arc|Rc|shared|ownership)").unwrap()
});
static STATIC_MUT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bstatic\s+mut\b").unwrap()
});

/// Check threading patterns in Rust code.
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
        if context::is_test_context(lines, line_num) {
            continue;
        }

        if STATIC_MUT_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "rust/threading/no-static-mut",
                "static mut is unsafe and racy — use AtomicX, Mutex, or OnceLock",
            ));
        }

        if ARC_RC_RE.is_match(line) {
            let has_comment = ARC_RC_COMMENT_RE.is_match(line)
                || (line_num > 0 && ARC_RC_COMMENT_RE.is_match(lines[line_num - 1]));
            if !has_comment {
                issues.push(Issue::new(
                    path, line_num + 1, 1, Severity::Warning,
                    "rust/threading/arc-rc-comment",
                    "Arc/Rc usage without ownership comment — add // why shared?",
                ));
            }
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
    fn catches_static_mut() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["static mut COUNTER: u32 = 0;"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].rule_id.contains("static-mut"));
    }

    #[test]
    fn arc_without_comment() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["let shared = Arc::new(data);"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.iter().any(|i| i.rule_id.contains("arc-rc")));
    }

    #[test]
    fn arc_with_comment() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["// Arc: shared across threads for state", "let shared = Arc::new(data);"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.iter().all(|i| !i.rule_id.contains("arc-rc")));
    }
}
