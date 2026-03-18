use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::config::Config;
use crate::context::{is_comment, FileContext};
use crate::issue::{Issue, Severity};

static DEF_NO_HINTS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*def\s+\w+\s*\(([^)]+)\)").unwrap());
static OPTIONAL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bOptional\[").unwrap());
static GLOBAL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*global\s+\w+").unwrap());
static CLASS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*class\s+(\w+)").unwrap());
static DEF_NAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(?:async\s+)?def\s+(\w+)").unwrap());
static JSON_LOADS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bjson\.loads\s*\(").unwrap());

/// Check for missing type annotations, Optional[], and global keyword.
pub fn check_missing_annotations(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    if file_ctx.is_test_file {
        return;
    }
    let path_str = path.to_string_lossy();
    if path_str.contains("/tools/") || path_str.contains("\\tools\\") {
        return;
    }

    for (line_num, line) in lines.iter().enumerate() {
        if is_comment(line, file_ctx.language) {
            continue;
        }

        // Check for def without type hints
        if let Some(caps) = DEF_NO_HINTS_RE.captures(line) {
            let params = &caps[1];
            // Skip self/cls, check remaining params for missing `: type`
            for param in params.split(',') {
                let p = param.trim();
                if p == "self" || p == "cls" || p == "*args" || p == "**kwargs" || p.is_empty() {
                    continue;
                }
                if !p.contains(':') {
                    issues.push(Issue::new(
                        path, line_num + 1, 1, Severity::Error,
                        "python/types/missing-annotations",
                        format!("parameter `{p}` missing type annotation"),
                    ));
                    break; // one issue per function
                }
            }
        }

        // Check for Optional[]
        if OPTIONAL_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "python/types/missing-annotations",
                "use `X | None` instead of `Optional[X]`",
            ));
        }

        // Check for global keyword
        if GLOBAL_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "python/types/missing-annotations",
                "avoid `global` — pass state explicitly or use a class",
            ));
        }
    }
}

/// Check Python naming conventions: PascalCase classes, snake_case functions.
pub fn check_naming_conventions(
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

        // Class names should be PascalCase
        if let Some(caps) = CLASS_RE.captures(line) {
            let name = &caps[1];
            if name.contains('_') || name.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) {
                issues.push(Issue::new(
                    path, line_num + 1, 1, Severity::Warning,
                    "python/naming/conventions",
                    format!("class `{name}` should be PascalCase"),
                ));
            }
        }

        // Function names should be snake_case
        if let Some(caps) = DEF_NAME_RE.captures(line) {
            let name = &caps[1];
            // Skip dunder methods and private
            if name.starts_with("__") && name.ends_with("__") {
                continue;
            }
            // Check for camelCase or PascalCase
            if name.chars().any(|c| c.is_uppercase()) && !name.chars().all(|c| c.is_uppercase() || c == '_') {
                // Has mixed case — likely camelCase
                if !name.contains('_') {
                    issues.push(Issue::new(
                        path, line_num + 1, 1, Severity::Warning,
                        "python/naming/conventions",
                        format!("function `{name}` should be snake_case"),
                    ));
                }
            }
        }
    }
}

/// Check for json.loads without model validation nearby.
pub fn check_boundary_validation(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    if file_ctx.is_test_file {
        return;
    }
    let path_str = path.to_string_lossy();
    if path_str.contains("/tools/") || path_str.contains("\\tools\\") {
        return;
    }

    for (line_num, line) in lines.iter().enumerate() {
        if is_comment(line, file_ctx.language) {
            continue;
        }
        if JSON_LOADS_RE.is_match(line) {
            // Check ±5 lines for validation patterns
            let start = line_num.saturating_sub(5);
            let end = (line_num + 6).min(lines.len());
            let nearby = &lines[start..end];
            let has_validation = nearby.iter().any(|l| {
                l.contains("model_validate") || l.contains("TypeAdapter")
                    || l.contains("pydantic") || l.contains("schema")
                    || l.contains("validate") || l.contains("dataclass")
            });
            if !has_validation {
                issues.push(Issue::new(
                    path, line_num + 1, 1, Severity::Error,
                    "python/validation/boundary-check",
                    "json.loads() without model validation — parse into a typed model",
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
        FileContext { language: Language::Python, is_test_file: false, is_mother_file: false, is_definition_file: false }
    }

    fn test_ctx() -> FileContext {
        FileContext { language: Language::Python, is_test_file: true, is_mother_file: false, is_definition_file: false }
    }

    // --- missing_annotations ---
    #[test]
    fn catches_missing_type_hint() {
        let mut issues = Vec::new();
        check_missing_annotations(&prod_ctx(), &["def foo(x, y):"], &Config::default(), &mut issues, Path::new("src/app.py"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("type annotation"));
    }

    #[test]
    fn allows_typed_params() {
        let mut issues = Vec::new();
        check_missing_annotations(&prod_ctx(), &["def foo(x: int, y: str):"], &Config::default(), &mut issues, Path::new("src/app.py"));
        assert!(issues.is_empty());
    }

    #[test]
    fn catches_optional() {
        let mut issues = Vec::new();
        check_missing_annotations(&prod_ctx(), &["x: Optional[int] = None"], &Config::default(), &mut issues, Path::new("src/app.py"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("Optional"));
    }

    // --- naming_conventions ---
    #[test]
    fn catches_non_pascal_class() {
        let mut issues = Vec::new();
        check_naming_conventions(&prod_ctx(), &["class my_class:"], &Config::default(), &mut issues, Path::new("app.py"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("PascalCase"));
    }

    #[test]
    fn catches_camel_case_function() {
        let mut issues = Vec::new();
        check_naming_conventions(&prod_ctx(), &["def processData(self):"], &Config::default(), &mut issues, Path::new("app.py"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("snake_case"));
    }

    #[test]
    fn allows_snake_case_function() {
        let mut issues = Vec::new();
        check_naming_conventions(&prod_ctx(), &["def process_data(self):"], &Config::default(), &mut issues, Path::new("app.py"));
        assert!(issues.is_empty());
    }

    // --- boundary_validation ---
    #[test]
    fn catches_unvalidated_json_loads() {
        let mut issues = Vec::new();
        check_boundary_validation(&prod_ctx(), &["data = json.loads(response.text)"], &Config::default(), &mut issues, Path::new("src/api.py"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn allows_validated_json_loads() {
        let mut issues = Vec::new();
        let lines = vec!["data = json.loads(response.text)", "result = Model.model_validate(data)"];
        check_boundary_validation(&prod_ctx(), &lines, &Config::default(), &mut issues, Path::new("src/api.py"));
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_test_files() {
        let mut issues = Vec::new();
        check_boundary_validation(&test_ctx(), &["data = json.loads(response.text)"], &Config::default(), &mut issues, Path::new("tests/test_api.py"));
        assert!(issues.is_empty());
    }
}
