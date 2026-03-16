use std::path::Path;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::context::{self, FileContext};
use crate::issue::{Issue, Severity};

static FLOAT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(\d+\.\d+)\b").unwrap()
});
static INT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(\d+)\b").unwrap()
});
static STRING_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#""[^"]*""#).unwrap()
});
static FORMAT_MACRO_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(format|println|eprintln|print|eprint|write|writeln|panic|log|debug|info|warn|error|trace|tracing)\s*!\s*\(").unwrap()
});
static ENUM_VARIANT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*\w+\s*=\s*\d+").unwrap()
});

/// Check for magic numbers in Rust code.
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
        if trimmed.starts_with("//") || trimmed.is_empty() {
            continue;
        }
        if context::is_const_def(line) {
            continue;
        }
        if context::is_test_context(lines, line_num) {
            continue;
        }
        if ENUM_VARIANT_RE.is_match(trimmed) {
            continue;
        }
        if FORMAT_MACRO_RE.is_match(line) {
            continue;
        }

        // Strip string literals to avoid false positives
        let stripped = STRING_RE.replace_all(line, "\"\"");

        // Check floats
        for cap in FLOAT_RE.captures_iter(&stripped) {
            let val = &cap[1];
            if val == "0.0" || val == "1.0" {
                continue;
            }
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "rust/constants/no-magic-number",
                format!("magic number {val} — use named const or _cfg field"),
            ));
        }

        // Check integers (skip those that are part of floats)
        let no_floats = FLOAT_RE.replace_all(&stripped, " ");
        for cap in INT_RE.captures_iter(&no_floats) {
            let val_str = &cap[1];
            let val: i64 = match val_str.parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            if val < 2 {
                continue;
            }
            // Skip if preceded by identifier char (part of a name like `v2`)
            let start = cap.get(1).unwrap().start();
            if start > 0 {
                let prev = no_floats.as_bytes()[start - 1];
                if prev == b'_' || prev.is_ascii_alphanumeric() {
                    continue;
                }
            }
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "rust/constants/no-magic-number",
                format!("magic number {val} — use named const or _cfg field"),
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
    fn catches_magic_int() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["let x = 42;"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("42"));
    }

    #[test]
    fn allows_zero_and_one() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["let x = 0;", "let y = 1;"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn allows_const_def() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["const LIMIT: usize = 100;"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn catches_magic_float() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["let pi = 3.14;"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn allows_zero_float() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["let x = 0.0;", "let y = 1.0;"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_format_macros() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["println!(\"got {} items\", 5);"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_enum_variants() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["    Variant = 42,"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }
}
