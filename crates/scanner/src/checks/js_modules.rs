use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::config::Config;
use crate::context::{is_comment, FileContext};
use crate::issue::{Issue, Severity};

static EXPORT_FN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*export\s+(?:default\s+)?(?:async\s+)?function\s+\w+").unwrap());
static EXPORT_CONST_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*export\s+(?:default\s+)?const\s+\w+").unwrap());
static REQUIRE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\brequire\s*\(").unwrap());
static MODULE_EXPORTS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bmodule\.exports\b").unwrap());

/// Check that exported functions/consts have JSDoc comments.
pub fn check_jsdoc_required(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    if file_ctx.is_test_file {
        return;
    }
    // Skip .d.ts declaration files
    let path_str = path.to_string_lossy();
    if path_str.ends_with(".d.ts") {
        return;
    }

    for (line_num, line) in lines.iter().enumerate() {
        let is_export = EXPORT_FN_RE.is_match(line) || EXPORT_CONST_RE.is_match(line);
        if is_export && !has_jsdoc_above(lines, line_num) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "js/jsdoc/type-required",
                "exported item missing JSDoc comment (/** ... */)",
            ));
        }
    }
}

/// Check for CommonJS require() and module.exports — use ES modules.
pub fn check_no_require(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    if file_ctx.is_test_file {
        return;
    }
    // Allow .cjs files
    let path_str = path.to_string_lossy();
    if path_str.ends_with(".cjs") {
        return;
    }

    for (line_num, line) in lines.iter().enumerate() {
        if is_comment(line, file_ctx.language) {
            continue;
        }

        if REQUIRE_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "js/modules/no-require",
                "require() is CommonJS — use ES module import instead",
            ));
        }

        if MODULE_EXPORTS_RE.is_match(line) {
            issues.push(Issue::new(
                path, line_num + 1, 1, Severity::Error,
                "js/modules/no-require",
                "module.exports is CommonJS — use ES module export instead",
            ));
        }
    }
}

fn has_jsdoc_above(lines: &[&str], line_idx: usize) -> bool {
    if line_idx == 0 {
        return false;
    }
    // Walk backwards looking for `*/` (end of JSDoc block)
    let mut idx = line_idx - 1;
    loop {
        let trimmed = lines[idx].trim();
        if trimmed.ends_with("*/") || trimmed == "*/" {
            return true;
        }
        // Skip empty lines between doc and export
        if trimmed.is_empty() {
            if idx == 0 { return false; }
            idx -= 1;
            continue;
        }
        return false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Language;

    fn js_ctx() -> FileContext {
        FileContext { language: Language::JavaScript, is_test_file: false, is_mother_file: false, is_definition_file: false }
    }

    fn test_ctx() -> FileContext {
        FileContext { language: Language::JavaScript, is_test_file: true, is_mother_file: false, is_definition_file: false }
    }

    // --- jsdoc_required ---
    #[test]
    fn catches_undocumented_export_function() {
        let mut issues = Vec::new();
        check_jsdoc_required(&js_ctx(), &["", "export function process() {"], &Config::default(), &mut issues, Path::new("src/app.js"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].rule_id.contains("jsdoc"));
    }

    #[test]
    fn allows_documented_export() {
        let mut issues = Vec::new();
        let lines = vec!["/** Process data. */", "export function process() {"];
        check_jsdoc_required(&js_ctx(), &lines, &Config::default(), &mut issues, Path::new("src/app.js"));
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_test_file_jsdoc() {
        let mut issues = Vec::new();
        check_jsdoc_required(&test_ctx(), &["export function helper() {"], &Config::default(), &mut issues, Path::new("src/app.test.js"));
        assert!(issues.is_empty());
    }

    // --- no_require ---
    #[test]
    fn catches_require() {
        let mut issues = Vec::new();
        check_no_require(&js_ctx(), &["const fs = require('fs');"], &Config::default(), &mut issues, Path::new("src/app.js"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("require"));
    }

    #[test]
    fn catches_module_exports() {
        let mut issues = Vec::new();
        check_no_require(&js_ctx(), &["module.exports = { foo };"], &Config::default(), &mut issues, Path::new("src/app.js"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn allows_cjs_files() {
        let mut issues = Vec::new();
        check_no_require(&js_ctx(), &["const fs = require('fs');"], &Config::default(), &mut issues, Path::new("config.cjs"));
        assert!(issues.is_empty());
    }
}
