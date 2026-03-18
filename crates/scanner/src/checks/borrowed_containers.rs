use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::config::Config;
use crate::context::{self, FileContext};
use crate::issue::{Issue, Severity};

static BORROWED_VEC_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"&Vec<").unwrap());
static BORROWED_STRING_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"&String\b").unwrap());
static BORROWED_HASHMAP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"&HashMap<").unwrap());
static BORROWED_HASHSET_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"&HashSet<").unwrap());

/// Check for borrowed container types in function parameters.
/// `&Vec<T>` should be `&[T]`, `&String` should be `&str`.
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

        // Only check function signatures
        if !trimmed.contains("fn ") {
            continue;
        }

        if BORROWED_VEC_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "rust/types/no-borrowed-container",
                "&Vec<T> in parameter — use &[T] instead",
            ));
        }

        if BORROWED_STRING_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "rust/types/no-borrowed-container",
                "&String in parameter — use &str instead",
            ));
        }

        if BORROWED_HASHMAP_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "rust/types/no-borrowed-container",
                "&HashMap<K,V> in parameter — use &HashMap or accept a trait bound",
            ));
        }

        if BORROWED_HASHSET_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "rust/types/no-borrowed-container",
                "&HashSet<T> in parameter — use &HashSet or accept a trait bound",
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

    fn test_ctx() -> FileContext {
        FileContext { language: Language::Rust, is_test_file: true, is_mother_file: false, is_definition_file: false }
    }

    #[test]
    fn catches_borrowed_vec() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["pub fn foo(items: &Vec<u32>) {}"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("&Vec<T>"));
    }

    #[test]
    fn catches_borrowed_string() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["fn bar(name: &String) {}"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("&String"));
    }

    #[test]
    fn allows_slice_and_str() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["pub fn foo(items: &[u32], name: &str) {}"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_test_files() {
        let mut issues = Vec::new();
        check(&test_ctx(), &["pub fn foo(items: &Vec<u32>) {}"], &Config::default(), &mut issues, Path::new("tests/test_a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_non_fn_lines() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["let x: &Vec<u32> = &v;"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }
}
