use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::config::Config;
use crate::context::{is_comment, FileContext};
use crate::issue::{Issue, Severity};

static EXPORT_COMPONENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*export\s+(?:component|struct)\s+\w+").unwrap());
static HEX_COLOR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"#[0-9a-fA-F]{3,8}\b").unwrap());
static PX_VALUE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b\d+px\b").unwrap());
static CONTROL_FLOW_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(?:if|for|while)\s").unwrap());
static HARDCODED_TEXT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?:^|\s)text\s*:\s*"[^"]+""#).unwrap());

/// Check that exported components/structs have `///` doc comments.
pub fn check_doc_required(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    if file_ctx.is_definition_file {
        return;
    }

    for (line_num, line) in lines.iter().enumerate() {
        if EXPORT_COMPONENT_RE.is_match(line) {
            if !has_doc_above(lines, line_num) {
                issues.push(Issue::new(
                    path, line_num + 1, 1, Severity::Error,
                    "slint/docs/doc-required",
                    "exported component/struct missing `///` doc comment",
                ));
            }
        }
    }
}

/// Check for hardcoded hex colors and px values in component files.
pub fn check_zero_literal(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    // Only flag in component files, not definition/token files
    if file_ctx.is_definition_file {
        return;
    }

    for (line_num, line) in lines.iter().enumerate() {
        if is_comment(line, file_ctx.language) {
            continue;
        }

        if HEX_COLOR_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "slint/tokens/zero-literal",
                "hardcoded hex color — use a token from globals instead",
            ));
        }

        if PX_VALUE_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "slint/tokens/zero-literal",
                "hardcoded px value — use a spacing/size token instead",
            ));
        }
    }
}

/// Check that global/definition files have no control flow.
pub fn check_globals_structure(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    // Only applies to definition files (globals)
    if !file_ctx.is_definition_file {
        return;
    }

    for (line_num, line) in lines.iter().enumerate() {
        if is_comment(line, file_ctx.language) {
            continue;
        }
        if CONTROL_FLOW_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Warning,
                "slint/globals/structure",
                "control flow in global file — globals should only define tokens/properties",
            ));
        }
    }
}

/// Check for hardcoded text strings in component files.
pub fn check_no_hardcoded_string(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    if file_ctx.is_definition_file {
        return;
    }

    for (line_num, line) in lines.iter().enumerate() {
        if is_comment(line, file_ctx.language) {
            continue;
        }
        if HARDCODED_TEXT_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "slint/strings/no-hardcoded-string",
                "hardcoded text string — use a translatable string resource",
            ));
        }
    }
}

fn has_doc_above(lines: &[&str], line_idx: usize) -> bool {
    if line_idx == 0 {
        return false;
    }
    let trimmed = lines[line_idx - 1].trim();
    trimmed.starts_with("///")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Language;

    fn component_ctx() -> FileContext {
        FileContext { language: Language::Slint, is_test_file: false, is_mother_file: false, is_definition_file: false }
    }

    fn definition_ctx() -> FileContext {
        FileContext { language: Language::Slint, is_test_file: false, is_mother_file: false, is_definition_file: true }
    }

    // --- doc_required ---
    #[test]
    fn catches_undocumented_export() {
        let mut issues = Vec::new();
        check_doc_required(&component_ctx(), &["", "export component Foo inherits Rectangle {"], &Config::default(), &mut issues, Path::new("a.slint"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].rule_id.contains("doc-required"));
    }

    #[test]
    fn allows_documented_export() {
        let mut issues = Vec::new();
        check_doc_required(&component_ctx(), &["/// A button.", "export component Foo inherits Rectangle {"], &Config::default(), &mut issues, Path::new("a.slint"));
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_definition_file_docs() {
        let mut issues = Vec::new();
        check_doc_required(&definition_ctx(), &["", "export struct Theme {"], &Config::default(), &mut issues, Path::new("_tokens.slint"));
        assert!(issues.is_empty());
    }

    // --- zero_literal ---
    #[test]
    fn catches_hex_color() {
        let mut issues = Vec::new();
        check_zero_literal(&component_ctx(), &["    background: #ff0000;"], &Config::default(), &mut issues, Path::new("a.slint"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("hex color"));
    }

    #[test]
    fn catches_px_value() {
        let mut issues = Vec::new();
        check_zero_literal(&component_ctx(), &["    width: 100px;"], &Config::default(), &mut issues, Path::new("a.slint"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("px"));
    }

    #[test]
    fn allows_tokens_in_definition() {
        let mut issues = Vec::new();
        check_zero_literal(&definition_ctx(), &["    out property <color> primary: #ff0000;"], &Config::default(), &mut issues, Path::new("_tokens.slint"));
        assert!(issues.is_empty());
    }

    // --- globals_structure ---
    #[test]
    fn catches_control_flow_in_global() {
        let mut issues = Vec::new();
        check_globals_structure(&definition_ctx(), &["    if condition {"], &Config::default(), &mut issues, Path::new("globals/theme.slint"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn allows_control_flow_in_component() {
        let mut issues = Vec::new();
        check_globals_structure(&component_ctx(), &["    if condition {"], &Config::default(), &mut issues, Path::new("a.slint"));
        assert!(issues.is_empty());
    }

    #[test]
    fn allows_properties_in_global() {
        let mut issues = Vec::new();
        check_globals_structure(&definition_ctx(), &["    out property <color> primary: blue;"], &Config::default(), &mut issues, Path::new("_tokens.slint"));
        assert!(issues.is_empty());
    }

    // --- no_hardcoded_string ---
    #[test]
    fn catches_hardcoded_text() {
        let mut issues = Vec::new();
        check_no_hardcoded_string(&component_ctx(), &["    Text { text: \"Hello World\"; }"], &Config::default(), &mut issues, Path::new("a.slint"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn allows_bound_text() {
        let mut issues = Vec::new();
        check_no_hardcoded_string(&component_ctx(), &["    Text { text: root.label; }"], &Config::default(), &mut issues, Path::new("a.slint"));
        assert!(issues.is_empty());
    }

    #[test]
    fn allows_empty_string_default() {
        let mut issues = Vec::new();
        check_no_hardcoded_string(&component_ctx(), &["    in property <string> breadcrumb-text: \"\";"], &Config::default(), &mut issues, Path::new("a.slint"));
        assert!(issues.is_empty());
    }

    #[test]
    fn allows_property_name_containing_text() {
        let mut issues = Vec::new();
        check_no_hardcoded_string(&component_ctx(), &["    in property <string> node-text: \"\";"], &Config::default(), &mut issues, Path::new("a.slint"));
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_definition_strings() {
        let mut issues = Vec::new();
        check_no_hardcoded_string(&definition_ctx(), &["    text: \"default\";"], &Config::default(), &mut issues, Path::new("_strings.slint"));
        assert!(issues.is_empty());
    }

    // --- comment regression tests (#79) ---
    #[test]
    fn skips_doc_comment_with_text() {
        let mut issues = Vec::new();
        check_no_hardcoded_string(
            &component_ctx(),
            &["/// text: \"Some doc example\""],
            &Config::default(), &mut issues, Path::new("a.slint"),
        );
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_doc_comment_hex_color() {
        let mut issues = Vec::new();
        check_zero_literal(
            &component_ctx(),
            &["/// Default color is #ff0000."],
            &Config::default(), &mut issues, Path::new("a.slint"),
        );
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_double_slash_comment_hex() {
        let mut issues = Vec::new();
        check_zero_literal(
            &component_ctx(),
            &["// background: #1a1a1a;"],
            &Config::default(), &mut issues, Path::new("a.slint"),
        );
        assert!(issues.is_empty());
    }
}
