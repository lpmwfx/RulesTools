use std::path::Path;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::context::{self, FileContext};
use crate::issue::{Issue, Severity};

static UNWRAP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bunwrap\s*\(\s*\)").unwrap()
});
static EXPECT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bexpect\s*\(").unwrap()
});
static PANIC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bpanic!\s*[(\[]").unwrap()
});
static TODO_UNIMPL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:todo|unimplemented)!\s*[(\[]").unwrap()
});

/// Check for unwrap/expect/panic/todo in non-test Rust code.
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

    // build.rs returns () — panic!/expect/unwrap are the only way to abort a build script
    let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
    if filename == "build.rs" {
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
        if context::is_const_def(line) {
            continue;
        }

        if UNWRAP_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "rust/errors/no-unwrap",
                "unwrap() in non-test code — use ? or handle the error explicitly",
            ));
        }
        if EXPECT_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "rust/errors/no-expect",
                "expect() in non-test code — use ? or return a typed error",
            ));
        }
        if PANIC_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "rust/errors/no-panic",
                "panic!() for recoverable error — return Err(...) instead",
            ));
        }
        if TODO_UNIMPL_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "rust/errors/no-todo",
                "todo!/unimplemented! — remove before shipping",
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
    fn catches_unwrap() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["let x = foo.unwrap();"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].rule_id.contains("unwrap"));
    }

    #[test]
    fn skips_test_file() {
        let mut issues = Vec::new();
        check(&test_ctx(), &["let x = foo.unwrap();"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_const_def() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r\"x\").unwrap());"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn catches_panic() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["panic!(\"boom\");"], &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn skips_build_rs() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["panic!(\"build failed: {e}\");"], &Config::default(), &mut issues, Path::new("build.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_build_rs_expect() {
        let mut issues = Vec::new();
        check(&prod_ctx(), &["let x = foo.expect(\"build config\");"], &Config::default(), &mut issues, Path::new("build.rs"));
        assert!(issues.is_empty());
    }
}
