use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::config::Config;
use crate::context::{is_comment, FileContext};
use crate::issue::{Issue, Severity};

static HEX_COLOR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"#[0-9a-fA-F]{3,8}\b").unwrap());
static PX_VALUE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b\d+px\b").unwrap());
static CUSTOM_PROP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"--[\w-]+\s*:").unwrap());
static ROOT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*:root\s*\{").unwrap());

/// Check for hardcoded hex colors and px values outside :root/custom properties.
pub fn check_zero_literal(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    let mut in_root = false;
    let mut root_depth: usize = 0;

    for (line_num, line) in lines.iter().enumerate() {
        if is_comment(line, file_ctx.language) {
            continue;
        }

        let trimmed = line.trim();

        // Track :root block
        if ROOT_RE.is_match(line) {
            in_root = true;
            root_depth = 1;
            continue;
        }
        if in_root {
            root_depth += trimmed.matches('{').count();
            root_depth = root_depth.saturating_sub(trimmed.matches('}').count());
            if root_depth == 0 {
                in_root = false;
            }
            continue; // Inside :root — tokens are allowed
        }

        // Skip custom property definitions (--var: value)
        if CUSTOM_PROP_RE.is_match(trimmed) {
            continue;
        }

        if HEX_COLOR_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Warning,
                "css/tokens/zero-literal",
                "hardcoded hex color — use a CSS custom property (var(--color))",
            ));
        }

        if PX_VALUE_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Warning,
                "css/tokens/zero-literal",
                "hardcoded px value — use a CSS custom property for spacing/sizing",
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Language;

    fn css_ctx() -> FileContext {
        FileContext { language: Language::Css, is_test_file: false, is_mother_file: false, is_definition_file: false }
    }

    #[test]
    fn catches_hardcoded_hex_color() {
        let mut issues = Vec::new();
        check_zero_literal(&css_ctx(), &[".btn { color: #ff0000; }"], &Config::default(), &mut issues, Path::new("style.css"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("hex color"));
    }

    #[test]
    fn catches_hardcoded_px() {
        let mut issues = Vec::new();
        check_zero_literal(&css_ctx(), &[".btn { padding: 16px; }"], &Config::default(), &mut issues, Path::new("style.css"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("px"));
    }

    #[test]
    fn allows_root_tokens() {
        let mut issues = Vec::new();
        let lines = vec![":root {", "  --primary: #ff0000;", "  --space: 16px;", "}"];
        check_zero_literal(&css_ctx(), &lines, &Config::default(), &mut issues, Path::new("style.css"));
        assert!(issues.is_empty());
    }
}
