use std::path::Path;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::context::{self, FileContext};
use crate::issue::{Issue, Severity};

/// Variable names too generic to convey meaning.
const BANNED_NAMES: &[&str] = &[
    "data", "info", "tmp", "temp", "val", "value", "item", "obj", "thing",
    "result", "ret", "res", "ctx", "mgr", "manager", "handler", "processor",
    "helper", "util", "misc",
];

static BOOL_DECL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\blet\s+(?:mut\s+)?(\w+)\s*(?::\s*bool)?.*=\s*(?:true|false)\b").unwrap()
});
static FN_BOOL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bfn\s+(\w+)\s*\(.*\)\s*->\s*bool\b").unwrap()
});
static UNSAFE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bunsafe\s*(\{|fn\b)").unwrap()
});
static SAFETY_COMMENT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"//\s*SAFETY\s*:").unwrap()
});
static LET_BINDING_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\blet\s+(?:mut\s+)?(\w+)\b").unwrap()
});

/// Check naming conventions in Rust code.
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

        check_banned_names(line, line_num, issues, path);
        check_bool_prefix(line, line_num, issues, path);
        check_unsafe_comment(lines, line_num, issues, path);
    }
}

fn check_banned_names(line: &str, line_num: usize, issues: &mut Vec<Issue>, path: &Path) {
    if let Some(caps) = LET_BINDING_RE.captures(line) {
        let name = &caps[1];
        if name.starts_with('_') {
            return;
        }
        if BANNED_NAMES.contains(&name) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "rust/naming/no-noise-names",
                format!("'{name}' is a banned bare name — add a domain suffix"),
            ));
        }
    }
}

fn check_bool_prefix(line: &str, line_num: usize, issues: &mut Vec<Issue>, path: &Path) {
    let valid_prefixes = ["is_", "has_", "can_", "should_", "will_", "did_", "was_"];

    if let Some(caps) = BOOL_DECL_RE.captures(line) {
        let name = &caps[1];
        if name.starts_with('_') || valid_prefixes.iter().any(|p| name.starts_with(p)) {
            return;
        }
        issues.push(Issue::new(
            path, line_num + 1, 1, Severity::Warning,
            "rust/naming/bool-prefix",
            format!("bool `{name}` should have is_/has_/can_/should_ prefix"),
        ));
    }

    if let Some(caps) = FN_BOOL_RE.captures(line) {
        let name = &caps[1];
        if name.starts_with('_') || valid_prefixes.iter().any(|p| name.starts_with(p)) {
            return;
        }
        issues.push(Issue::new(
            path, line_num + 1, 1, Severity::Warning,
            "rust/naming/bool-prefix",
            format!("fn `{name}` -> bool should have is_/has_/can_/should_ prefix"),
        ));
    }
}

fn check_unsafe_comment(lines: &[&str], line_num: usize, issues: &mut Vec<Issue>, path: &Path) {
    let line = lines[line_num];
    if !UNSAFE_RE.is_match(line) {
        return;
    }

    // Check line before, same line, line after for // SAFETY:
    let has_safety = SAFETY_COMMENT_RE.is_match(line)
        || (line_num > 0 && SAFETY_COMMENT_RE.is_match(lines[line_num - 1]))
        || (line_num + 1 < lines.len() && SAFETY_COMMENT_RE.is_match(lines[line_num + 1]));

    if !has_safety {
        issues.push(Issue::new(
            path, line_num + 1, 1, Severity::Error,
            "rust/naming/unsafe-comment",
            "unsafe block/fn requires a `// SAFETY:` comment",
        ));
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
    fn catches_banned_name() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["let data = vec![];"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("data"));
    }

    #[test]
    fn allows_prefixed_name() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["let scan_data = vec![];"], &Config::default(), &mut issues, Path::new("a.rs"));
        // "scan_data" is not in BANNED_NAMES
        assert!(issues.is_empty());
    }

    #[test]
    fn bool_without_prefix() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["let active = true;"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.iter().any(|i| i.rule_id.contains("bool-prefix")));
    }

    #[test]
    fn bool_with_prefix_ok() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["let is_active = true;"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.iter().all(|i| !i.rule_id.contains("bool-prefix")));
    }

    #[test]
    fn unsafe_without_safety_comment() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["", "unsafe {"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.iter().any(|i| i.rule_id.contains("unsafe")));
    }

    #[test]
    fn unsafe_with_safety_comment() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["// SAFETY: pointer is valid", "unsafe {"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.iter().all(|i| !i.rule_id.contains("unsafe")));
    }
}
