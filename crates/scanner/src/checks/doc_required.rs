use std::path::Path;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::context::FileContext;
use crate::issue::{Issue, Severity};

static PUB_ITEM_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*pub(?:\([^)]+\))?\s+(?:async\s+)?(?:fn|struct|enum|trait|type|mod|const|static)\s+(\w+)").unwrap()
});

/// Check that all pub items have `///` doc comments.
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
        // Skip `pub use` re-exports
        if line.contains(" use ") {
            continue;
        }

        if let Some(caps) = PUB_ITEM_RE.captures(line) {
            let item_name = &caps[1];
            if !has_doc_comment(lines, line_num) {
                issues.push(Issue::new(
                    path,
                    line_num + 1,
                    1,
                    Severity::Error,
                    "rust/docs/doc-required",
                    format!("pub item `{item_name}` is missing a `///` doc comment"),
                ));
            }
        }
    }
}

/// Walk backwards from `line_idx`, skipping `#[...]` attributes, looking for `///`.
fn has_doc_comment(lines: &[&str], line_idx: usize) -> bool {
    if line_idx == 0 {
        return false;
    }
    let mut idx = line_idx - 1;
    loop {
        let trimmed = lines[idx].trim();

        if trimmed.starts_with("///") {
            return true;
        }
        // Skip attributes like #[derive(...)], #[cfg(...)], etc.
        if trimmed.starts_with("#[") {
            if idx == 0 {
                return false;
            }
            idx -= 1;
            continue;
        }
        // Any other line means no doc comment
        return false;
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
    fn documented_fn_ok() {
        let mut issues = Vec::new();
        let lines = vec!["/// Does something.", "pub fn foo() {}"];
        check(&prod_ctx(), &lines, &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn undocumented_fn_flagged() {
        let mut issues = Vec::new();
        let lines = vec!["", "pub fn foo() {}"];
        check(&prod_ctx(), &lines, &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("foo"));
    }

    #[test]
    fn doc_through_attributes() {
        let mut issues = Vec::new();
        let lines = vec!["/// Documented.", "#[derive(Debug)]", "pub struct Foo;"];
        check(&prod_ctx(), &lines, &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn pub_use_skipped() {
        let mut issues = Vec::new();
        let lines = vec!["pub use crate::other::Thing;"];
        check(&prod_ctx(), &lines, &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }
}
