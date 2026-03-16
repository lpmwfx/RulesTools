use std::path::Path;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::context::{self, FileContext};
use crate::issue::{Issue, Severity};

static CLONE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\.clone\(\)").unwrap()
});
static FN_START_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)").unwrap()
});

const CLONE_THRESHOLD: usize = 3;

/// Check for excessive .clone() calls within a single function.
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

    let mut current_fn: Option<(String, usize)> = None;
    let mut clone_count: usize = 0;
    let mut brace_depth: i32 = 0;

    for (line_num, line) in lines.iter().enumerate() {
        if context::is_test_context(lines, line_num) {
            continue;
        }

        if let Some(caps) = FN_START_RE.captures(line) {
            // Emit issue for previous fn if needed
            if let Some((ref fn_name, fn_line)) = current_fn {
                if clone_count > CLONE_THRESHOLD {
                    issues.push(Issue::new(
                        path, fn_line, 1, Severity::Warning,
                        "rust/ownership/clone-spam",
                        format!("fn `{fn_name}` has {clone_count} .clone() calls — consider borrowing or restructuring"),
                    ));
                }
            }
            current_fn = Some((caps[1].to_string(), line_num + 1));
            clone_count = 0;
            brace_depth = 0;
        }

        if current_fn.is_some() {
            brace_depth += line.matches('{').count() as i32;
            brace_depth -= line.matches('}').count() as i32;

            clone_count += CLONE_RE.find_iter(line).count();

            if brace_depth <= 0 && line.contains('}') {
                if let Some((ref fn_name, fn_line)) = current_fn {
                    if clone_count > CLONE_THRESHOLD {
                        issues.push(Issue::new(
                            path, fn_line, 1, Severity::Warning,
                            "rust/ownership/clone-spam",
                            format!("fn `{fn_name}` has {clone_count} .clone() calls — consider borrowing or restructuring"),
                        ));
                    }
                }
                current_fn = None;
                clone_count = 0;
            }
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
    fn allows_few_clones() {
        let mut issues = Vec::new();
        let lines = vec![
            "fn foo() {",
            "    let a = x.clone();",
            "    let b = y.clone();",
            "}",
        ];
        check(&prod_ctx(), &lines, &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn catches_clone_spam() {
        let mut issues = Vec::new();
        let lines = vec![
            "fn bar() {",
            "    let a = x.clone();",
            "    let b = y.clone();",
            "    let c = z.clone();",
            "    let d = w.clone();",
            "}",
        ];
        check(&prod_ctx(), &lines, &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("4"));
    }
}
