use crate::config::Config;
use crate::context::FileContext;
use crate::issue::{Issue, Severity};
use std::path::Path;

/// Check that files in shared/ have no internal project imports.
///
/// shared/ must be dependency-free — only std and external crates allowed.
pub fn check(
    _file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    let normalized = path.to_string_lossy().replace('\\', "/");
    if !normalized.contains("/shared/") && !normalized.contains("/common/") {
        return;
    }

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }

        if trimmed.starts_with("use crate::")
            || trimmed.starts_with("use super::")
            || trimmed.starts_with("pub use crate::")
            || trimmed.starts_with("pub use super::")
        {
            // Skip imports inside #[cfg(test)] modules — test code can import from parent
            if crate::context::is_test_context(lines, i) {
                continue;
            }
            issues.push(Issue::new(
                path,
                i + 1,
                1,
                Severity::Error,
                "rust/modules/shared-guard",
                &format!(
                    "internal import in shared/ — shared modules must be dependency-free: {}",
                    trimmed
                ),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Language;

    fn ctx() -> FileContext {
        FileContext {
            language: Language::Rust,
            is_test_file: false,
            is_mother_file: false,
            is_definition_file: false,
        }
    }

    #[test]
    fn clean_shared_file() {
        let lines = vec![
            "use std::collections::HashMap;",
            "use serde::Serialize;",
            "",
            "pub fn helper() -> bool { true }",
        ];
        let mut issues = Vec::new();
        check(&ctx(), &lines, &Config::default(), &mut issues, Path::new("src/shared/utils.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn internal_import_error() {
        let lines = vec!["use crate::gateway::db;"];
        let mut issues = Vec::new();
        check(&ctx(), &lines, &Config::default(), &mut issues, Path::new("src/shared/utils.rs"));
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "rust/modules/shared-guard");
    }

    #[test]
    fn super_import_error() {
        let lines = vec!["use super::something;"];
        let mut issues = Vec::new();
        check(&ctx(), &lines, &Config::default(), &mut issues, Path::new("src/shared/helpers.rs"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn not_in_shared_ok() {
        let lines = vec!["use crate::gateway::db;"];
        let mut issues = Vec::new();
        check(&ctx(), &lines, &Config::default(), &mut issues, Path::new("src/core/logic.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn common_dir_also_checked() {
        let lines = vec!["use crate::something;"];
        let mut issues = Vec::new();
        check(&ctx(), &lines, &Config::default(), &mut issues, Path::new("src/common/types.rs"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn pub_reexport_error() {
        let lines = vec!["pub use crate::core::Engine;"];
        let mut issues = Vec::new();
        check(&ctx(), &lines, &Config::default(), &mut issues, Path::new("src/shared/mod.rs"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn comment_ignored() {
        let lines = vec!["// use crate::gateway::db;"];
        let mut issues = Vec::new();
        check(&ctx(), &lines, &Config::default(), &mut issues, Path::new("src/shared/utils.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn use_super_in_cfg_test_ok() {
        let lines = vec![
            "pub fn helper() -> bool { true }",
            "",
            "#[cfg(test)]",
            "mod tests {",
            "    use super::*;",
            "    #[test]",
            "    fn test_helper() {}",
            "}",
        ];
        let mut issues = Vec::new();
        check(&ctx(), &lines, &Config::default(), &mut issues, Path::new("src/shared/utils.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn use_crate_in_cfg_test_ok() {
        let lines = vec![
            "pub fn helper() -> bool { true }",
            "",
            "#[cfg(test)]",
            "mod tests {",
            "    use crate::shared::other_module::Thing;",
            "}",
        ];
        let mut issues = Vec::new();
        check(&ctx(), &lines, &Config::default(), &mut issues, Path::new("src/shared/utils.rs"));
        assert!(issues.is_empty());
    }
}
