use std::path::Path;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::context::{self, FileContext};
use crate::issue::{Issue, Severity};

static FIRE_FORGET_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:tokio::spawn|thread::spawn)\s*\(").unwrap()
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

/// Check for fire-and-forget spawns — `tokio::spawn(...)` or `thread::spawn(...)`
/// without a `let` binding on the same line.
pub fn check_fire_and_forget(
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

        if FIRE_FORGET_RE.is_match(trimmed) {
            // Allow if line has `let` binding — handle is captured
            if trimmed.starts_with("let ") {
                continue;
            }
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "rust/threading/no-fire-and-forget",
                "fire-and-forget spawn — capture the JoinHandle with `let handle = ...`",
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

    #[test]
    fn catches_fire_and_forget_tokio() {
        let mut issues = Vec::new();
        check_fire_and_forget(&prod_ctx(), &["    tokio::spawn(async move { work() });"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].rule_id.contains("fire-and-forget"));
    }

    #[test]
    fn catches_fire_and_forget_thread() {
        let mut issues = Vec::new();
        check_fire_and_forget(&prod_ctx(), &["    thread::spawn(|| heavy_work());"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn allows_captured_spawn() {
        let mut issues = Vec::new();
        check_fire_and_forget(&prod_ctx(), &["    let handle = tokio::spawn(async move { work() });"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn fire_forget_skips_test() {
        let test_ctx = FileContext { language: Language::Rust, is_test_file: true, is_mother_file: false, is_definition_file: false };
        let mut issues = Vec::new();
        check_fire_and_forget(&test_ctx, &["    tokio::spawn(async {});"], &Config::default(), &mut issues, Path::new("tests/a.rs"));
        assert!(issues.is_empty());
    }
}
