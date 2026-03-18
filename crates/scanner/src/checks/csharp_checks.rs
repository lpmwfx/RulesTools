use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::config::Config;
use crate::context::{is_comment, FileContext};
use crate::issue::{Issue, Severity};

static CLASS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(?:public|internal|protected)?\s*(?:static\s+|abstract\s+|sealed\s+)*class\s+(\w+)").unwrap());
static METHOD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(?:public|internal|protected|private)\s+(?:static\s+|virtual\s+|override\s+|async\s+)*\w+\s+(\w+)\s*\(").unwrap());
static PRIVATE_FIELD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*private\s+\w+\s+(\w+)\s*[;=]").unwrap());
static PUBLIC_DECL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*public\s+(?:static\s+|virtual\s+|override\s+|async\s+|abstract\s+)*(?:class|interface|void|int|bool|string|Task|IActionResult)\s").unwrap());

/// Check C# naming conventions.
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
                    "csharp/naming/conventions",
                    format!("class `{name}` should be PascalCase"),
                ));
            }
        }

        // Public methods should be PascalCase
        if let Some(caps) = METHOD_RE.captures(line) {
            let name = &caps[1];
            if line.contains("public ") || line.contains("internal ") || line.contains("protected ") {
                if !line.contains("private ") && name.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) {
                    issues.push(Issue::new(
                        path, line_num + 1, 1, Severity::Warning,
                        "csharp/naming/conventions",
                        format!("public method `{name}` should be PascalCase"),
                    ));
                }
            }
        }

        // Private fields should be camelCase (with optional _ prefix)
        if let Some(caps) = PRIVATE_FIELD_RE.captures(line) {
            let name = &caps[1];
            let check_name = name.strip_prefix('_').unwrap_or(name);
            if !check_name.is_empty() && check_name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                issues.push(Issue::new(
                    path, line_num + 1, 1, Severity::Warning,
                    "csharp/naming/conventions",
                    format!("private field `{name}` should be camelCase or _camelCase"),
                ));
            }
        }
    }
}

/// Check C# public items have XML doc comments (///).
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

        if PUBLIC_DECL_RE.is_match(line) {
            if !has_xml_doc_above(lines, line_num) {
                issues.push(Issue::new(
                    path, line_num + 1, 1, Severity::Error,
                    "csharp/docs/doc-required",
                    "public declaration missing XML doc comment (/// <summary>)",
                ));
            }
        }
    }
}

fn has_xml_doc_above(lines: &[&str], line_idx: usize) -> bool {
    if line_idx == 0 {
        return false;
    }
    let mut idx = line_idx - 1;
    loop {
        let trimmed = lines[idx].trim();
        if trimmed.starts_with("///") {
            return true;
        }
        // Skip attributes like [Authorize], [HttpGet]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
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

    fn cs_ctx() -> FileContext {
        FileContext { language: Language::CSharp, is_test_file: false, is_mother_file: false, is_definition_file: false }
    }

    fn test_ctx() -> FileContext {
        FileContext { language: Language::CSharp, is_test_file: true, is_mother_file: false, is_definition_file: false }
    }

    // --- naming ---
    #[test]
    fn catches_lowercase_class() {
        let mut issues = Vec::new();
        check_naming(&cs_ctx(), &["public class myService {"], &Config::default(), &mut issues, Path::new("a.cs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("PascalCase"));
    }

    #[test]
    fn catches_lowercase_public_method() {
        let mut issues = Vec::new();
        check_naming(&cs_ctx(), &["    public void processData() {"], &Config::default(), &mut issues, Path::new("a.cs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("PascalCase"));
    }

    #[test]
    fn allows_correct_naming() {
        let mut issues = Vec::new();
        let lines = vec!["public class MyService {", "    public void ProcessData() {"];
        check_naming(&cs_ctx(), &lines, &Config::default(), &mut issues, Path::new("a.cs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn catches_uppercase_private_field() {
        let mut issues = Vec::new();
        check_naming(&cs_ctx(), &["    private int Counter;"], &Config::default(), &mut issues, Path::new("a.cs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("camelCase"));
    }

    // --- doc_required ---
    #[test]
    fn catches_undocumented_public_class() {
        let mut issues = Vec::new();
        check_doc_required(&cs_ctx(), &["", "public class Service {"], &Config::default(), &mut issues, Path::new("a.cs"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn allows_documented_public_class() {
        let mut issues = Vec::new();
        check_doc_required(&cs_ctx(), &["/// <summary>Service.</summary>", "public class Service {"], &Config::default(), &mut issues, Path::new("a.cs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_test_files() {
        let mut issues = Vec::new();
        check_doc_required(&test_ctx(), &["public class ServiceTest {"], &Config::default(), &mut issues, Path::new("ServiceTest.cs"));
        assert!(issues.is_empty());
    }
}
