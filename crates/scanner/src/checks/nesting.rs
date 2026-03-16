use std::path::Path;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::context::{FileContext, Language};
use crate::issue::{Issue, Severity};

static STRING_DOUBLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#""(?:[^"\\]|\\.)*""#).unwrap()
});
static STRING_SINGLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"'(?:[^'\\]|\\.)*'").unwrap()
});

/// Max nesting depth per language.
fn max_depth(lang: Language) -> usize {
    match lang {
        Language::Rust => 5,
        Language::JavaScript | Language::TypeScript | Language::Css => 4,
        Language::Slint | Language::Kotlin => 6,
        Language::CSharp => 7,
        Language::Python => 8,
    }
}

/// Strip string literals and line comments from a line.
fn strip_strings_and_comments(line: &str) -> String {
    let mut stripped = STRING_DOUBLE_RE.replace_all(line, "").to_string();
    stripped = STRING_SINGLE_RE.replace_all(&stripped, "").to_string();
    if let Some(pos) = stripped.find("//") {
        stripped.truncate(pos);
    }
    stripped
}

/// Check brace nesting depth.
pub fn check(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    let limit = max_depth(file_ctx.language);
    let mut depth: usize = 0;

    for (line_num, line) in lines.iter().enumerate() {
        let cleaned = strip_strings_and_comments(line);

        let opens = cleaned.matches('{').count();
        let closes = cleaned.matches('}').count();

        if opens > 0 {
            depth = depth.saturating_add(opens);
        }

        // Only flag lines that open braces and push over limit
        if opens > closes && depth > limit {
            let sev = if depth > limit + 1 {
                Severity::Error
            } else {
                Severity::Warning
            };
            issues.push(Issue::new(
                path,
                line_num + 1,
                1,
                sev,
                "global/nesting",
                format!("nesting depth {depth} exceeds limit {limit} — extract a helper function"),
            ));
        }

        if closes > 0 {
            depth = depth.saturating_sub(closes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(lang: Language) -> FileContext {
        FileContext {
            language: lang,
            is_test_file: false,
            is_mother_file: false,
            is_definition_file: false,
        }
    }

    #[test]
    fn normal_nesting_ok() {
        let mut issues = Vec::new();
        let lines = vec![
            "fn main() {",
            "    if true {",
            "        println!(\"ok\");",
            "    }",
            "}",
        ];
        check(&make_ctx(Language::Rust), &lines, &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn deep_nesting_flagged() {
        let mut issues = Vec::new();
        // 6 levels deep — exceeds Rust limit of 5
        let lines = vec![
            "fn f() {",
            "  if a {",
            "    if b {",
            "      if c {",
            "        if d {",
            "          if e {",
            "          }",
            "        }",
            "      }",
            "    }",
            "  }",
            "}",
        ];
        check(&make_ctx(Language::Rust), &lines, &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(!issues.is_empty());
    }

    #[test]
    fn strings_not_counted() {
        let result = strip_strings_and_comments(r#"let s = "{ not a brace }";"#);
        assert!(!result.contains('{'));
    }
}
