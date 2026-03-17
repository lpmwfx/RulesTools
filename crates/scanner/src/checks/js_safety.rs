use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::config::Config;
use crate::context::{is_comment, FileContext};
use crate::issue::{Issue, Severity};

static VAR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bvar\s+\w+").unwrap());
static CONSOLE_LOG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bconsole\s*\.\s*log\s*\(").unwrap());
static EVAL_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\beval\s*\(").unwrap());

/// Check for `var` declarations — use `let` or `const` instead.
pub fn check_no_var(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    if file_ctx.is_test_file {
        return;
    }
    for (i, line) in lines.iter().enumerate() {
        if is_comment(line, file_ctx.language) {
            continue;
        }
        if VAR_RE.is_match(line) {
            issues.push(Issue::new(
                path,
                i + 1,
                1,
                Severity::Warning,
                "js/safety/no-var",
                "use let or const instead of var",
            ));
        }
    }
}

/// Check for `console.log` — use a structured logger.
pub fn check_no_console_log(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    if file_ctx.is_test_file {
        return;
    }
    for (i, line) in lines.iter().enumerate() {
        if is_comment(line, file_ctx.language) {
            continue;
        }
        if CONSOLE_LOG_RE.is_match(line) {
            issues.push(Issue::new(
                path,
                i + 1,
                1,
                Severity::Warning,
                "js/safety/no-console-log",
                "remove console.log — use a structured logger",
            ));
        }
    }
}

/// Check for `eval()` — security risk.
pub fn check_no_eval(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    if file_ctx.is_test_file {
        return;
    }
    for (i, line) in lines.iter().enumerate() {
        if is_comment(line, file_ctx.language) {
            continue;
        }
        if EVAL_RE.is_match(line) {
            issues.push(Issue::new(
                path,
                i + 1,
                1,
                Severity::Error,
                "js/safety/no-eval",
                "eval() is a security risk — use JSON.parse or a parser",
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checks::Language;

    fn make_js_ctx() -> FileContext {
        FileContext {
            language: Language::JavaScript,
            is_test_file: false,
            is_mother_file: false,
            is_definition_file: false,
        }
    }

    fn make_test_ctx() -> FileContext {
        FileContext {
            language: Language::JavaScript,
            is_test_file: true,
            is_mother_file: false,
            is_definition_file: false,
        }
    }

    #[test]
    fn catches_var_declaration() {
        let mut issues = Vec::new();
        let lines = vec!["var x = 1;"];
        check_no_var(&make_js_ctx(), &lines, &Config::default(), &mut issues, Path::new("src/app.js"));
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "js/safety/no-var");
    }

    #[test]
    fn allows_let_const() {
        let mut issues = Vec::new();
        let lines = vec!["let x = 1;", "const y = 2;"];
        check_no_var(&make_js_ctx(), &lines, &Config::default(), &mut issues, Path::new("src/app.js"));
        assert!(issues.is_empty());
    }

    #[test]
    fn catches_console_log() {
        let mut issues = Vec::new();
        let lines = vec!["console.log(\"debug\");"];
        check_no_console_log(&make_js_ctx(), &lines, &Config::default(), &mut issues, Path::new("src/app.js"));
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "js/safety/no-console-log");
    }

    #[test]
    fn allows_console_error() {
        let mut issues = Vec::new();
        let lines = vec!["console.error(\"fail\");"];
        check_no_console_log(&make_js_ctx(), &lines, &Config::default(), &mut issues, Path::new("src/app.js"));
        assert!(issues.is_empty());
    }

    #[test]
    fn catches_eval() {
        let mut issues = Vec::new();
        let lines = vec!["eval(\"code\")"];
        check_no_eval(&make_js_ctx(), &lines, &Config::default(), &mut issues, Path::new("src/app.js"));
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "js/safety/no-eval");
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn skips_test_file() {
        let mut issues = Vec::new();
        let lines = vec!["var x = 1;", "console.log(\"x\");", "eval(\"y\")"];
        check_no_var(&make_test_ctx(), &lines, &Config::default(), &mut issues, Path::new("src/app.test.js"));
        check_no_console_log(&make_test_ctx(), &lines, &Config::default(), &mut issues, Path::new("src/app.test.js"));
        check_no_eval(&make_test_ctx(), &lines, &Config::default(), &mut issues, Path::new("src/app.test.js"));
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_comments() {
        let mut issues = Vec::new();
        let lines = vec!["// var x = 1;", "/* console.log(\"x\") */", "// eval(\"y\")"];
        check_no_var(&make_js_ctx(), &lines, &Config::default(), &mut issues, Path::new("src/app.js"));
        check_no_console_log(&make_js_ctx(), &lines, &Config::default(), &mut issues, Path::new("src/app.js"));
        check_no_eval(&make_js_ctx(), &lines, &Config::default(), &mut issues, Path::new("src/app.js"));
        assert!(issues.is_empty());
    }

    #[test]
    fn typescript_also_checked() {
        let ctx = FileContext {
            language: Language::TypeScript,
            is_test_file: false,
            is_mother_file: false,
            is_definition_file: false,
        };
        let mut issues = Vec::new();
        let lines = vec!["var x: number = 1;"];
        check_no_var(&ctx, &lines, &Config::default(), &mut issues, Path::new("src/app.ts"));
        assert_eq!(issues.len(), 1);
    }
}
