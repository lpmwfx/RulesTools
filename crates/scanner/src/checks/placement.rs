use crate::config::Config;
use crate::context::FileContext;
use crate::issue::{Issue, Severity};
use std::path::Path;

/// Check that code files are not in metadata/documentation folders.
pub fn check(
    _file_ctx: &FileContext,
    _lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    let is_code_ext = matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs" | "slint")
    );
    if !is_code_ext {
        return;
    }

    let normalized = path.to_string_lossy().replace('\\', "/");

    if is_toplevel_metadata(&normalized, "proj") {
        issues.push(Issue::new(
            path, 0, 0, Severity::Error,
            "topology/placement",
            "code file in proj/ — proj/ is for metadata only",
        ));
    }

    if (is_toplevel_metadata(&normalized, "doc") || is_toplevel_metadata(&normalized, "docs"))
        && !normalized.contains("/examples/")
    {
        issues.push(Issue::new(
            path, 0, 0, Severity::Error,
            "topology/placement",
            "code file in doc/ — doc/ is for documentation only",
        ));
    }

    if is_toplevel_metadata(&normalized, "man") {
        issues.push(Issue::new(
            path, 0, 0, Severity::Error,
            "topology/placement",
            "code file in man/ — man/ is for generated documentation only",
        ));
    }
}

/// Check if a path has a top-level metadata directory (not nested inside src/, ui/, etc.).
fn is_toplevel_metadata(path: &str, segment: &str) -> bool {
    // Split path into segments
    let parts: Vec<&str> = path.split('/').collect();

    // Find position of the segment in path
    let seg_idx = match parts.iter().position(|&p| p == segment) {
        Some(i) => i,
        None => return false,
    };

    // If segment is preceded by any code directory, it's nested — not metadata
    let code_dirs = ["src", "ui", "crates", "examples", "lib"];
    for i in 0..seg_idx {
        if code_dirs.contains(&parts[i]) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Language;

    fn make_ctx() -> FileContext {
        FileContext {
            language: Language::Rust,
            is_test_file: false,
            is_mother_file: false,
            is_definition_file: false,
        }
    }

    #[test]
    fn code_in_src_ok() {
        let ctx = make_ctx();
        let mut issues = Vec::new();
        check(&ctx, &[], &Config::default(), &mut issues, Path::new("src/main.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn code_in_proj_error() {
        let ctx = make_ctx();
        let mut issues = Vec::new();
        check(&ctx, &[], &Config::default(), &mut issues, Path::new("some/proj/hack.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("proj/"));
    }

    #[test]
    fn code_in_doc_error() {
        let ctx = make_ctx();
        let mut issues = Vec::new();
        check(&ctx, &[], &Config::default(), &mut issues, Path::new("x/doc/snippet.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("doc/"));
    }

    #[test]
    fn code_in_man_error() {
        let ctx = make_ctx();
        let mut issues = Vec::new();
        check(&ctx, &[], &Config::default(), &mut issues, Path::new("x/man/gen.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("man/"));
    }

    #[test]
    fn non_code_in_proj_ok() {
        let ctx = make_ctx();
        let mut issues = Vec::new();
        check(&ctx, &[], &Config::default(), &mut issues, Path::new("proj/TODO.md"));
        assert!(issues.is_empty()); // .md is not code
    }

    #[test]
    fn slint_in_doc_error() {
        let ctx = FileContext {
            language: Language::Slint,
            is_test_file: false,
            is_mother_file: false,
            is_definition_file: false,
        };
        let mut issues = Vec::new();
        check(&ctx, &[], &Config::default(), &mut issues, Path::new("x/doc/widget.slint"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn nested_docs_in_src_ok() {
        let ctx = make_ctx();
        let mut issues = Vec::new();
        // src/docs/ is a code module, not a metadata folder
        check(&ctx, &[], &Config::default(), &mut issues, Path::new("src/docs/mod.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn nested_docs_in_ui_ok() {
        let ctx = FileContext {
            language: Language::Slint,
            is_test_file: false,
            is_mother_file: false,
            is_definition_file: false,
        };
        let mut issues = Vec::new();
        // ui/docs/ is a UI component folder, not metadata
        check(&ctx, &[], &Config::default(), &mut issues, Path::new("ui/docs/markdown-view.slint"));
        assert!(issues.is_empty());
    }

    #[test]
    fn toplevel_doc_still_error() {
        let ctx = make_ctx();
        let mut issues = Vec::new();
        check(&ctx, &[], &Config::default(), &mut issues, Path::new("doc/snippet.rs"));
        assert_eq!(issues.len(), 1);
    }
}
