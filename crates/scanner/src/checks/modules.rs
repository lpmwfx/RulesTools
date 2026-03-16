use std::path::Path;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::context::FileContext;
use crate::issue::{Issue, Severity};

static UTILS_FILE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|[/\\])(?:utils|helpers|misc|common)\.(rs|py|ts|js)$").unwrap()
});
static INLINE_MOD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(?:pub\s+)?mod\s+(\w+)\s*\{").unwrap()
});

/// Check module structure rules.
pub fn check(
    _file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    let path_str = path.to_string_lossy();

    // utils.rs / helpers.rs — banned module names
    if UTILS_FILE_RE.is_match(&path_str) {
        issues.push(Issue::new(
            path, 1, 1, Severity::Error,
            "rust/modules/no-utils",
            "utils/helpers/misc modules are banned — name modules by what they do",
        ));
    }

    // Inline mod blocks
    for (line_num, line) in lines.iter().enumerate() {
        if let Some(caps) = INLINE_MOD_RE.captures(line) {
            let mod_name = &caps[1];
            // Count lines in the inline module
            let mod_lines = count_inline_mod_lines(lines, line_num);
            if mod_lines > 0 {
                issues.push(Issue::new(
                    path, line_num + 1, 1, Severity::Warning,
                    "rust/modules/no-inline-mod",
                    format!("inline mod `{mod_name}` ({mod_lines} lines) — extract to {mod_name}.rs"),
                ));
            }
        }
    }
}

/// Count lines in an inline mod block by tracking brace depth.
fn count_inline_mod_lines(lines: &[&str], start: usize) -> usize {
    let mut depth: i32 = 0;
    for (offset, line) in lines[start..].iter().enumerate() {
        depth += line.matches('{').count() as i32;
        depth -= line.matches('}').count() as i32;
        if depth == 0 && offset > 0 {
            return offset;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Language;

    fn make_ctx() -> FileContext {
        FileContext { language: Language::Rust, is_test_file: false, is_mother_file: false, is_definition_file: false }
    }

    #[test]
    fn catches_utils_file() {
        let mut issues = Vec::new();
        check(&make_ctx(), &["// content"], &Config::default(), &mut issues, Path::new("src/utils.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].rule_id.contains("utils"));
    }

    #[test]
    fn normal_module_ok() {
        let mut issues = Vec::new();
        check(&make_ctx(), &["// content"], &Config::default(), &mut issues, Path::new("src/scanner.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn inline_mod_flagged() {
        let mut issues = Vec::new();
        let lines = vec![
            "mod inner {",
            "    fn foo() {}",
            "    fn bar() {}",
            "}",
        ];
        check(&make_ctx(), &lines, &Config::default(), &mut issues, Path::new("src/lib.rs"));
        assert!(issues.iter().any(|i| i.rule_id.contains("inline-mod")));
    }
}
