use crate::config::Config;
use crate::context::FileContext;
use crate::issue::{Issue, Severity};
use std::path::Path;

/// Check Slint mother-child topology rules.
///
/// 1. Children must be stateless — no `in-out property` (except <=> delegation)
/// 2. Child files cannot import from sibling views
pub fn check(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    // Only check Slint files
    if file_ctx.language != crate::context::Language::Slint {
        return;
    }

    // Mother files (main.slint, *_view.slint) are exempt from child rules
    if file_ctx.is_mother_file {
        return;
    }

    // Definition files (_tokens.slint, globals/) are exempt
    if file_ctx.is_definition_file {
        return;
    }

    let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("");

    // Skip shared/common widgets
    let path_str = path.to_string_lossy();
    let normalized = path_str.replace('\\', "/");
    if normalized.contains("/shared/") || normalized.contains("/common/") {
        return;
    }

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Check 1: Children must be stateless — no `in-out property`
        // Exception: `<=>` delegation (binding to parent property)
        if trimmed.starts_with("in-out property") {
            // Check if this line or next has <=> (delegation)
            let has_delegation = trimmed.contains("<=>");
            let next_has_delegation = lines
                .get(i + 1)
                .map(|l| l.contains("<=>"))
                .unwrap_or(false);

            if !has_delegation && !next_has_delegation {
                issues.push(Issue::new(
                    path,
                    i + 1,
                    1,
                    Severity::Error,
                    "uiux/mother-child/child-has-state",
                    &format!(
                        "child component `{}` owns state via `in-out property` — children must be stateless, receive via `in property`",
                        filename,
                    ),
                ));
            }
        }

        // Check 2: import from sibling views
        // Pattern: import { SiblingView } from "sibling-view.slint";
        if trimmed.starts_with("import ") && trimmed.contains("from") {
            // Extract the source file
            if let Some(source) = extract_import_source(trimmed) {
                // If importing from a file in same directory that's not shared/globals
                if !source.contains('/') && !source.starts_with('_') && source != filename {
                    // Check if it looks like a sibling view (not a shared component)
                    if source.ends_with(".slint") && !source.contains("shared")
                        && source != "std-widgets.slint"
                    {
                        issues.push(Issue::new(
                            path,
                            i + 1,
                            1,
                            Severity::Warning,
                            "uiux/mother-child/sibling-import",
                            &format!(
                                "import from sibling `{}` — children should not import siblings, route through mother",
                                source,
                            ),
                        ));
                    }
                }
            }
        }
    }
}

/// Extract the source file from an import statement.
/// `import { Foo } from "bar.slint";` → Some("bar.slint")
fn extract_import_source(line: &str) -> Option<String> {
    let from_idx = line.find("from")?;
    let after_from = &line[from_idx + 4..];
    let quote_start = after_from.find('"')? + 1;
    let rest = &after_from[quote_start..];
    let quote_end = rest.find('"')?;
    Some(rest[..quote_end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Language;

    fn child_ctx() -> FileContext {
        FileContext {
            language: Language::Slint,
            is_test_file: false,
            is_mother_file: false,
            is_definition_file: false,
        }
    }

    fn mother_ctx() -> FileContext {
        FileContext {
            language: Language::Slint,
            is_test_file: false,
            is_mother_file: true,
            is_definition_file: false,
        }
    }

    #[test]
    fn child_with_in_out_property_error() {
        let lines = vec![
            "export component MyButton inherits Rectangle {",
            "    in-out property <string> label;",
            "}",
        ];
        let mut issues = Vec::new();
        check(&child_ctx(), &lines, &Config::default(), &mut issues, Path::new("ui/my-button.slint"));
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "uiux/mother-child/child-has-state");
    }

    #[test]
    fn child_with_in_property_ok() {
        let lines = vec![
            "export component MyButton inherits Rectangle {",
            "    in property <string> label;",
            "}",
        ];
        let mut issues = Vec::new();
        check(&child_ctx(), &lines, &Config::default(), &mut issues, Path::new("ui/my-button.slint"));
        assert!(issues.is_empty());
    }

    #[test]
    fn child_with_delegation_ok() {
        let lines = vec![
            "export component MyButton inherits Rectangle {",
            "    in-out property <string> label <=> root.label;",
            "}",
        ];
        let mut issues = Vec::new();
        check(&child_ctx(), &lines, &Config::default(), &mut issues, Path::new("ui/my-button.slint"));
        assert!(issues.is_empty());
    }

    #[test]
    fn mother_file_exempt() {
        let lines = vec![
            "export component MainView inherits Window {",
            "    in-out property <string> title;",
            "}",
        ];
        let mut issues = Vec::new();
        check(&mother_ctx(), &lines, &Config::default(), &mut issues, Path::new("ui/main_view.slint"));
        assert!(issues.is_empty());
    }

    #[test]
    fn sibling_import_warning() {
        let lines = vec![
            r#"import { SideNav } from "side-nav.slint";"#,
        ];
        let mut issues = Vec::new();
        check(&child_ctx(), &lines, &Config::default(), &mut issues, Path::new("ui/content.slint"));
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "uiux/mother-child/sibling-import");
    }

    #[test]
    fn import_from_shared_ok() {
        let lines = vec![
            r#"import { Button } from "../shared/button.slint";"#,
        ];
        let mut issues = Vec::new();
        check(&child_ctx(), &lines, &Config::default(), &mut issues, Path::new("ui/content.slint"));
        assert!(issues.is_empty()); // path contains '/' — not a sibling
    }

    #[test]
    fn import_globals_ok() {
        let lines = vec![
            r#"import { Theme } from "_tokens.slint";"#,
        ];
        let mut issues = Vec::new();
        check(&child_ctx(), &lines, &Config::default(), &mut issues, Path::new("ui/content.slint"));
        assert!(issues.is_empty()); // starts with _
    }

    #[test]
    fn extract_import_source_works() {
        assert_eq!(
            extract_import_source(r#"import { Foo } from "bar.slint";"#),
            Some("bar.slint".to_string())
        );
        assert_eq!(
            extract_import_source(r#"import { A, B } from "../shared/widgets.slint";"#),
            Some("../shared/widgets.slint".to_string())
        );
    }

    #[test]
    fn rust_file_skipped() {
        let ctx = FileContext {
            language: Language::Rust,
            is_test_file: false,
            is_mother_file: false,
            is_definition_file: false,
        };
        let lines = vec!["in-out property <string> label;"];
        let mut issues = Vec::new();
        check(&ctx, &lines, &Config::default(), &mut issues, Path::new("src/main.rs"));
        assert!(issues.is_empty());
    }
}
