use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::config::Config;
use crate::context::{is_comment, FileContext};
use crate::issue::{Issue, Severity};

static CLASS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(?:class|struct)\s+(\w+)").unwrap());
static PUB_DECL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(?:class|struct|void|int|bool|auto|static|virtual|template)\s").unwrap());
static RAW_MEMORY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:malloc|free|new\s+\w|delete\s)").unwrap());

/// Check C++ naming conventions: PascalCase classes, snake_case/camelCase functions.
pub fn check_naming(
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
        if is_comment(line, file_ctx.language) {
            continue;
        }

        // Class/struct names should be PascalCase
        if let Some(caps) = CLASS_RE.captures(line) {
            let name = &caps[1];
            // Skip if all lowercase with underscores (C-style struct)
            if name.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) {
                issues.push(Issue::new(
                    path, line_num + 1, 1, Severity::Warning,
                    "cpp/naming/conventions",
                    format!("class/struct `{name}` should be PascalCase"),
                ));
            }
        }
    }
}

/// Check C++ public items have doc comments.
pub fn check_doc_required(
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
        if is_comment(line, file_ctx.language) {
            continue;
        }

        // Check class/struct and public function declarations
        let is_decl = CLASS_RE.is_match(line)
            || (PUB_DECL_RE.is_match(line) && line.contains('('));

        if is_decl && !has_doc_above(lines, line_num) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "cpp/docs/doc-required",
                "public declaration missing doc comment (/// or /** */)",
            ));
        }
    }
}

/// Check for raw memory management — prefer smart pointers.
pub fn check_safety(
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
        if is_comment(line, file_ctx.language) {
            continue;
        }
        if RAW_MEMORY_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Warning,
                "cpp/safety/no-raw-memory",
                "raw memory management — use std::unique_ptr/std::shared_ptr/RAII",
            ));
        }
    }
}

fn has_doc_above(lines: &[&str], line_idx: usize) -> bool {
    if line_idx == 0 {
        return false;
    }
    let trimmed = lines[line_idx - 1].trim();
    trimmed.starts_with("///") || trimmed.ends_with("*/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Language;

    fn cpp_ctx() -> FileContext {
        FileContext { language: Language::Cpp, is_test_file: false, is_mother_file: false, is_definition_file: false }
    }

    fn test_ctx() -> FileContext {
        FileContext { language: Language::Cpp, is_test_file: true, is_mother_file: false, is_definition_file: false }
    }

    // --- naming ---
    #[test]
    fn catches_lowercase_class() {
        let mut issues = Vec::new();
        check_naming(&cpp_ctx(), &["class my_parser {"], &Config::default(), &mut issues, Path::new("a.cpp"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("PascalCase"));
    }

    #[test]
    fn allows_pascal_class() {
        let mut issues = Vec::new();
        check_naming(&cpp_ctx(), &["class MyParser {"], &Config::default(), &mut issues, Path::new("a.cpp"));
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_test_naming() {
        let mut issues = Vec::new();
        check_naming(&test_ctx(), &["class my_test {"], &Config::default(), &mut issues, Path::new("test_a.cpp"));
        assert!(issues.is_empty());
    }

    // --- doc_required ---
    #[test]
    fn catches_undocumented_class() {
        let mut issues = Vec::new();
        check_doc_required(&cpp_ctx(), &["", "class Parser {"], &Config::default(), &mut issues, Path::new("a.hpp"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn allows_documented_class() {
        let mut issues = Vec::new();
        check_doc_required(&cpp_ctx(), &["/// A parser.", "class Parser {"], &Config::default(), &mut issues, Path::new("a.hpp"));
        assert!(issues.is_empty());
    }

    #[test]
    fn catches_undocumented_function() {
        let mut issues = Vec::new();
        check_doc_required(&cpp_ctx(), &["", "void process(int x) {"], &Config::default(), &mut issues, Path::new("a.cpp"));
        assert_eq!(issues.len(), 1);
    }

    // --- safety ---
    #[test]
    fn catches_malloc() {
        let mut issues = Vec::new();
        check_safety(&cpp_ctx(), &["    void* p = malloc(100);"], &Config::default(), &mut issues, Path::new("a.cpp"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("raw memory"));
    }

    #[test]
    fn catches_raw_new() {
        let mut issues = Vec::new();
        check_safety(&cpp_ctx(), &["    auto* p = new Widget();"], &Config::default(), &mut issues, Path::new("a.cpp"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn allows_smart_pointer() {
        let mut issues = Vec::new();
        check_safety(&cpp_ctx(), &["    auto p = std::make_unique<Widget>();"], &Config::default(), &mut issues, Path::new("a.cpp"));
        assert!(issues.is_empty());
    }
}
