use std::path::PathBuf;

use crate::config::Config;
use crate::context::{FileContext, Language};
use crate::issue::Issue;

/// Per-file check function signature.
pub type PerFileCheckFn = fn(ctx: &FileContext, lines: &[&str], cfg: &Config, issues: &mut Vec<Issue>, path: &std::path::Path);

/// Cross-file check function signature (operates on multiple file contents).
pub type CrossFileCheckFn = fn(contents: &[(PathBuf, String)], cfg: &Config, issues: &mut Vec<Issue>);

/// Tree-level check function signature (operates on file paths only).
pub type TreeCheckFn = fn(paths: &[PathBuf], cfg: &Config, issues: &mut Vec<Issue>);

/// The kind of check — determines how it is dispatched.
pub enum CheckKind {
    PerFile(PerFileCheckFn),
    CrossFile(CrossFileCheckFn),
    Tree(TreeCheckFn),
}

/// A registered check entry.
pub struct CheckEntry {
    pub id: String,
    pub languages: Vec<Language>,
    pub kind: CheckKind,
}

impl CheckEntry {
    pub fn per_file(
        id: impl Into<String>,
        languages: Vec<Language>,
        func: PerFileCheckFn,
    ) -> Self {
        Self {
            id: id.into(),
            languages,
            kind: CheckKind::PerFile(func),
        }
    }

    pub fn cross_file(
        id: impl Into<String>,
        languages: Vec<Language>,
        func: CrossFileCheckFn,
    ) -> Self {
        Self {
            id: id.into(),
            languages,
            kind: CheckKind::CrossFile(func),
        }
    }

    pub fn tree(
        id: impl Into<String>,
        languages: Vec<Language>,
        func: TreeCheckFn,
    ) -> Self {
        Self {
            id: id.into(),
            languages,
            kind: CheckKind::Tree(func),
        }
    }

    /// Check if this entry applies to the given language.
    pub fn applies_to(&self, lang: Language) -> bool {
        self.languages.is_empty() || self.languages.contains(&lang)
    }
}

/// Return all registered checks. Empty in phase 1 — checks are added in phase 2+.
pub fn registry() -> Vec<CheckEntry> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_registry() {
        let checks = registry();
        assert!(checks.is_empty());
    }

    #[test]
    fn check_entry_applies_to_specific_language() {
        fn dummy(_ctx: &FileContext, _lines: &[&str], _cfg: &Config, _issues: &mut Vec<Issue>, _path: &std::path::Path) {}

        let entry = CheckEntry::per_file(
            "rust/magic_numbers",
            vec![Language::Rust],
            dummy,
        );
        assert!(entry.applies_to(Language::Rust));
        assert!(!entry.applies_to(Language::Python));
    }

    #[test]
    fn check_entry_applies_to_all_when_empty() {
        fn dummy(_ctx: &FileContext, _lines: &[&str], _cfg: &Config, _issues: &mut Vec<Issue>, _path: &std::path::Path) {}

        let entry = CheckEntry::per_file(
            "global/nesting",
            vec![],
            dummy,
        );
        assert!(entry.applies_to(Language::Rust));
        assert!(entry.applies_to(Language::Python));
        assert!(entry.applies_to(Language::Slint));
    }

    #[test]
    fn check_entry_construction() {
        fn dummy_tree(_paths: &[PathBuf], _cfg: &Config, _issues: &mut Vec<Issue>) {}

        let entry = CheckEntry::tree(
            "rust/scanner_installed",
            vec![Language::Rust],
            dummy_tree,
        );
        assert_eq!(entry.id, "rust/scanner_installed");
        assert!(entry.applies_to(Language::Rust));
        assert!(matches!(entry.kind, CheckKind::Tree(_)));
    }
}
