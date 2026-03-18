use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::config::Config;
use crate::context::{is_comment, FileContext};
use crate::issue::{Issue, Severity};

static CLASS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(?:data\s+)?class\s+(\w+)").unwrap());
static FUN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(?:(?:private|internal|public|override|suspend)\s+)*fun\s+(\w+)").unwrap());
static CONST_NAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bconst\s+val\s+(\w+)").unwrap());
static KDOC_FUN_CLASS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(?:(?:data|sealed|abstract|open|private|internal|public|override|suspend)\s+)*(?:fun|class)\s+\w+").unwrap());

/// Check Kotlin naming conventions.
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

        // Classes should be PascalCase
        if let Some(caps) = CLASS_RE.captures(line) {
            let name = &caps[1];
            if name.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) {
                issues.push(Issue::new(
                    path, line_num + 1, 1, Severity::Warning,
                    "kotlin/naming/conventions",
                    format!("class `{name}` should be PascalCase"),
                ));
            }
        }

        // Functions should be camelCase
        if let Some(caps) = FUN_RE.captures(line) {
            let name = &caps[1];
            if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                issues.push(Issue::new(
                    path, line_num + 1, 1, Severity::Warning,
                    "kotlin/naming/conventions",
                    format!("function `{name}` should be camelCase"),
                ));
            }
        }

        // Constants should be UPPER_SNAKE_CASE
        if let Some(caps) = CONST_NAME_RE.captures(line) {
            let name = &caps[1];
            if name.chars().any(|c| c.is_lowercase()) {
                issues.push(Issue::new(
                    path, line_num + 1, 1, Severity::Warning,
                    "kotlin/naming/conventions",
                    format!("const `{name}` should be UPPER_SNAKE_CASE"),
                ));
            }
        }
    }
}

/// Check that Kotlin fun/class have KDoc comments.
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

        if KDOC_FUN_CLASS_RE.is_match(line) {
            // Skip private items
            if line.contains("private ") {
                continue;
            }
            if !has_kdoc_above(lines, line_num) {
                issues.push(Issue::new(
                    path, line_num + 1, 1, Severity::Error,
                    "kotlin/docs/doc-required",
                    "public fun/class missing KDoc comment (/** */)",
                ));
            }
        }
    }
}

fn has_kdoc_above(lines: &[&str], line_idx: usize) -> bool {
    if line_idx == 0 {
        return false;
    }
    let mut idx = line_idx - 1;
    loop {
        let trimmed = lines[idx].trim();
        if trimmed.ends_with("*/") || trimmed == "*/" {
            return true;
        }
        // Skip annotations like @Override
        if trimmed.starts_with('@') {
            if idx == 0 { return false; }
            idx -= 1;
            continue;
        }
        return false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Language;

    fn kt_ctx() -> FileContext {
        FileContext { language: Language::Kotlin, is_test_file: false, is_mother_file: false, is_definition_file: false }
    }

    fn test_ctx() -> FileContext {
        FileContext { language: Language::Kotlin, is_test_file: true, is_mother_file: false, is_definition_file: false }
    }

    // --- naming ---
    #[test]
    fn catches_lowercase_class() {
        let mut issues = Vec::new();
        check_naming(&kt_ctx(), &["class myService {"], &Config::default(), &mut issues, Path::new("a.kt"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("PascalCase"));
    }

    #[test]
    fn catches_uppercase_fun() {
        let mut issues = Vec::new();
        check_naming(&kt_ctx(), &["fun ProcessData() {"], &Config::default(), &mut issues, Path::new("a.kt"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("camelCase"));
    }

    #[test]
    fn catches_lowercase_const() {
        let mut issues = Vec::new();
        check_naming(&kt_ctx(), &["const val apiKey = \"abc\""], &Config::default(), &mut issues, Path::new("a.kt"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("UPPER_SNAKE"));
    }

    #[test]
    fn allows_correct_naming() {
        let mut issues = Vec::new();
        let lines = vec!["class MyService {", "fun processData() {", "const val API_KEY = \"abc\""];
        check_naming(&kt_ctx(), &lines, &Config::default(), &mut issues, Path::new("a.kt"));
        assert!(issues.is_empty());
    }

    // --- doc_required ---
    #[test]
    fn catches_undocumented_fun() {
        let mut issues = Vec::new();
        check_doc_required(&kt_ctx(), &["", "fun process() {"], &Config::default(), &mut issues, Path::new("a.kt"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn allows_documented_fun() {
        let mut issues = Vec::new();
        check_doc_required(&kt_ctx(), &["/** Process data. */", "fun process() {"], &Config::default(), &mut issues, Path::new("a.kt"));
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_test_kotlin() {
        let mut issues = Vec::new();
        check_doc_required(&test_ctx(), &["fun process() {"], &Config::default(), &mut issues, Path::new("test/AppTest.kt"));
        assert!(issues.is_empty());
    }
}
