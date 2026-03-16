use std::path::Path;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::context::FileContext;
use crate::issue::{Issue, Severity};

static FN_DEF_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(?:pub(?:\([^)]+\))?\s+)?(?:async\s+)?fn\s+(\w+)").unwrap()
});
static STATE_DECL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:static|lazy_static|thread_local|OnceLock|LazyLock)\b").unwrap()
});
static IMPL_BLOCK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*impl\b").unwrap()
});

const MOTHER_FN_WARN: usize = 3;
const MOTHER_FN_ERROR: usize = 6;

/// Check mother-child patterns — mother files should be compositors, not accumulators.
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

    if file_ctx.is_mother_file {
        check_mother_fn_count(lines, issues, path);
    } else {
        check_child_owns_state(lines, issues, path);
    }
}

/// Mother files (mod.rs, main.rs, lib.rs) should primarily compose children.
/// Too many fn definitions means logic is accumulating in the mother.
fn check_mother_fn_count(lines: &[&str], issues: &mut Vec<Issue>, path: &Path) {
    let mut fn_count = 0;
    let mut fn_names: Vec<String> = Vec::new();
    let mut in_impl = false;
    let mut impl_depth: i32 = 0;

    for line in lines {
        // Track impl blocks — fns inside impl are exempt
        if IMPL_BLOCK_RE.is_match(line) {
            in_impl = true;
            impl_depth = 0;
        }
        if in_impl {
            impl_depth += line.matches('{').count() as i32;
            impl_depth -= line.matches('}').count() as i32;
            if impl_depth <= 0 {
                in_impl = false;
            }
            continue;
        }

        if let Some(caps) = FN_DEF_RE.captures(line) {
            let name = caps[1].to_string();
            // Skip main() — it's the entry point
            if name != "main" {
                fn_count += 1;
                fn_names.push(name);
            }
        }
    }

    if fn_count > MOTHER_FN_ERROR {
        issues.push(Issue::new(
            path, 1, 1, Severity::Error,
            "uiux/mother-child/mother-too-many-fns",
            format!(
                "mother file has {fn_count} fn definitions ({}) — extract to child modules",
                fn_names.join(", ")
            ),
        ));
    } else if fn_count > MOTHER_FN_WARN {
        issues.push(Issue::new(
            path, 1, 1, Severity::Warning,
            "uiux/mother-child/mother-too-many-fns",
            format!(
                "mother file has {fn_count} fn definitions ({}) — consider extracting to children",
                fn_names.join(", ")
            ),
        ));
    }
}

/// Child files should not own global state — state belongs in mother or state/ module.
fn check_child_owns_state(lines: &[&str], issues: &mut Vec<Issue>, path: &Path) {
    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }

        // Allow LazyLock<Regex> — scanner patterns are legitimate child state
        if trimmed.contains("Regex") {
            continue;
        }

        if STATE_DECL_RE.is_match(trimmed)
            && !trimmed.starts_with("use ")
            && !trimmed.starts_with("//")
        {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Warning,
                "uiux/mother-child/child-owns-state",
                "global state in child file — move to mother or state/ module",
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Language;

    fn mother_ctx() -> FileContext {
        FileContext { language: Language::Rust, is_test_file: false, is_mother_file: true, is_definition_file: false }
    }
    fn child_ctx() -> FileContext {
        FileContext { language: Language::Rust, is_test_file: false, is_mother_file: false, is_definition_file: false }
    }

    #[test]
    fn mother_few_fns_ok() {
        let mut issues = Vec::new();
        let lines = vec!["fn setup() {}", "fn teardown() {}"];
        check(&mother_ctx(), &lines, &Config::default(), &mut issues, Path::new("mod.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn mother_too_many_fns_warns() {
        let mut issues = Vec::new();
        let lines = vec![
            "fn a() {}", "fn b() {}", "fn c() {}", "fn d() {}",
        ];
        check(&mother_ctx(), &lines, &Config::default(), &mut issues, Path::new("mod.rs"));
        assert!(issues.iter().any(|i| i.severity == Severity::Warning));
    }

    #[test]
    fn mother_many_fns_errors() {
        let mut issues = Vec::new();
        let lines = vec![
            "fn a() {}", "fn b() {}", "fn c() {}",
            "fn d() {}", "fn e() {}", "fn f() {}", "fn g() {}",
        ];
        check(&mother_ctx(), &lines, &Config::default(), &mut issues, Path::new("mod.rs"));
        assert!(issues.iter().any(|i| i.severity == Severity::Error));
    }

    #[test]
    fn mother_impl_fns_exempt() {
        let mut issues = Vec::new();
        let lines = vec![
            "impl Foo {",
            "    fn a() {}", "    fn b() {}", "    fn c() {}",
            "    fn d() {}", "    fn e() {}", "    fn f() {}",
            "    fn g() {}", "    fn h() {}",
            "}",
        ];
        check(&mother_ctx(), &lines, &Config::default(), &mut issues, Path::new("mod.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn child_state_warns() {
        let mut issues = Vec::new();
        let lines = vec!["static COUNTER: AtomicUsize = AtomicUsize::new(0);"];
        check(&child_ctx(), &lines, &Config::default(), &mut issues, Path::new("worker.rs"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn child_regex_state_ok() {
        let mut issues = Vec::new();
        let lines = vec!["static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r\"x\").unwrap());"];
        check(&child_ctx(), &lines, &Config::default(), &mut issues, Path::new("checker.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn child_use_statement_ok() {
        let mut issues = Vec::new();
        let lines = vec!["use std::sync::OnceLock;"];
        check(&child_ctx(), &lines, &Config::default(), &mut issues, Path::new("worker.rs"));
        assert!(issues.is_empty());
    }
}
