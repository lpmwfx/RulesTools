use crate::config::Config;
use crate::context::FileContext;
use crate::issue::{Issue, Severity};
use std::path::Path;

/// Check that every `unsafe` block or `unsafe fn` has a `// SAFETY:` comment.
pub fn check(
    _file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }

        // Skip string literals and assertions
        if trimmed.contains('"') && !trimmed.starts_with("unsafe") {
            continue;
        }

        let has_unsafe = trimmed.starts_with("unsafe ")
            || trimmed.starts_with("unsafe{")
            || trimmed.contains(" unsafe {")
            || trimmed.contains(" unsafe{");

        if !has_unsafe {
            continue;
        }

        // Check preceding lines for // SAFETY: comment
        let has_safety_comment = (0..i)
            .rev()
            .take(3)
            .any(|j| {
                let prev = lines[j].trim();
                prev.starts_with("// SAFETY:") || prev.starts_with("/// SAFETY:")
            });

        if !has_safety_comment {
            issues.push(Issue::new(
                path,
                i + 1,
                1,
                Severity::Error,
                "rust/safety/unsafe-needs-comment",
                "unsafe block without // SAFETY: comment",
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Language;

    fn ctx() -> FileContext {
        FileContext {
            language: Language::Rust,
            is_test_file: false,
            is_mother_file: false,
            is_definition_file: false,
        }
    }

    #[test]
    fn unsafe_with_safety_comment() {
        let lines = vec![
            "// SAFETY: pointer is valid for lifetime of struct",
            "unsafe { ptr::read(p) }",
        ];
        let mut issues = Vec::new();
        check(&ctx(), &lines, &Config::default(), &mut issues, Path::new("src/pal/ffi.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn unsafe_without_comment() {
        let lines = vec![
            "let val = unsafe { ptr::read(p) };",
        ];
        let mut issues = Vec::new();
        check(&ctx(), &lines, &Config::default(), &mut issues, Path::new("src/pal/ffi.rs"));
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "rust/safety/unsafe-needs-comment");
    }

    #[test]
    fn unsafe_fn_with_comment() {
        let lines = vec![
            "// SAFETY: caller must ensure alignment",
            "unsafe fn raw_op() {}",
        ];
        let mut issues = Vec::new();
        check(&ctx(), &lines, &Config::default(), &mut issues, Path::new("src/pal/ffi.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn unsafe_fn_without_comment() {
        let lines = vec![
            "",
            "unsafe fn raw_op() {}",
        ];
        let mut issues = Vec::new();
        check(&ctx(), &lines, &Config::default(), &mut issues, Path::new("src/pal/ffi.rs"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn no_unsafe_no_issues() {
        let lines = vec![
            "fn safe_fn() {",
            "    let x = 42;",
            "}",
        ];
        let mut issues = Vec::new();
        check(&ctx(), &lines, &Config::default(), &mut issues, Path::new("src/core/calc.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn safety_comment_two_lines_above() {
        let lines = vec![
            "// SAFETY: raw handle valid until drop",
            "",
            "unsafe { CloseHandle(h) }",
        ];
        let mut issues = Vec::new();
        check(&ctx(), &lines, &Config::default(), &mut issues, Path::new("src/pal/win.rs"));
        assert!(issues.is_empty());
    }
}
