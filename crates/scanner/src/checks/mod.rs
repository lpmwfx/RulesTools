use std::path::PathBuf;

use crate::config::Config;
use crate::context::{FileContext, Language};
use crate::issue::Issue;

pub mod file_size;
pub mod nesting;
pub mod debt;
pub mod secrets;
pub mod magic_numbers;
pub mod errors;
pub mod doc_required;
pub mod string_states;
pub mod naming;
pub mod modules;

/// Per-file check function signature.
pub type PerFileCheckFn = fn(
    file_ctx: &FileContext,
    lines: &[&str],
    cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &std::path::Path,
);

/// Cross-file check function signature.
pub type CrossFileCheckFn = fn(
    contents: &[(PathBuf, String)],
    cfg: &Config,
    issues: &mut Vec<Issue>,
);

/// Tree-level check function signature.
pub type TreeCheckFn = fn(
    paths: &[PathBuf],
    cfg: &Config,
    issues: &mut Vec<Issue>,
);

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
    /// Create a per-file check for specific languages.
    pub fn per_file(id: impl Into<String>, languages: Vec<Language>, func: PerFileCheckFn) -> Self {
        Self { id: id.into(), languages, kind: CheckKind::PerFile(func) }
    }

    /// Create a cross-file check.
    pub fn cross_file(id: impl Into<String>, languages: Vec<Language>, func: CrossFileCheckFn) -> Self {
        Self { id: id.into(), languages, kind: CheckKind::CrossFile(func) }
    }

    /// Create a tree-level check.
    pub fn tree(id: impl Into<String>, languages: Vec<Language>, func: TreeCheckFn) -> Self {
        Self { id: id.into(), languages, kind: CheckKind::Tree(func) }
    }

    /// Check if this entry applies to the given language.
    pub fn applies_to(&self, lang: Language) -> bool {
        self.languages.is_empty() || self.languages.contains(&lang)
    }
}

/// Return all registered checks.
pub fn registry() -> Vec<CheckEntry> {
    vec![
        // Common checks (all languages)
        CheckEntry::per_file("global/file-limits", vec![], file_size::check),
        CheckEntry::per_file("global/nesting", vec![], nesting::check),
        CheckEntry::per_file("global/tech-debt", vec![], debt::check),
        CheckEntry::per_file("global/secrets", vec![], secrets::check),
        // Rust checks
        CheckEntry::per_file("rust/constants/no-magic-number", vec![Language::Rust], magic_numbers::check),
        CheckEntry::per_file("rust/errors/no-unwrap", vec![Language::Rust], errors::check),
        CheckEntry::per_file("rust/docs/doc-required", vec![Language::Rust], doc_required::check),
        CheckEntry::per_file("rust/types/no-string-match", vec![Language::Rust], string_states::check),
        CheckEntry::per_file("rust/naming/no-noise-names", vec![Language::Rust], naming::check),
        CheckEntry::per_file("rust/modules/no-utils", vec![Language::Rust], modules::check),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_checks() {
        let checks = registry();
        assert!(checks.len() >= 10);
    }

    #[test]
    fn check_entry_applies_to_specific_language() {
        fn dummy(_: &FileContext, _: &[&str], _: &Config, _: &mut Vec<Issue>, _: &std::path::Path) {}
        let entry = CheckEntry::per_file("rust/test", vec![Language::Rust], dummy);
        assert!(entry.applies_to(Language::Rust));
        assert!(!entry.applies_to(Language::Python));
    }

    #[test]
    fn check_entry_applies_to_all_when_empty() {
        fn dummy(_: &FileContext, _: &[&str], _: &Config, _: &mut Vec<Issue>, _: &std::path::Path) {}
        let entry = CheckEntry::per_file("global/test", vec![], dummy);
        assert!(entry.applies_to(Language::Rust));
        assert!(entry.applies_to(Language::Python));
    }

    #[test]
    fn check_entry_construction() {
        fn dummy_tree(_: &[PathBuf], _: &Config, _: &mut Vec<Issue>) {}
        let entry = CheckEntry::tree("rust/scanner_installed", vec![Language::Rust], dummy_tree);
        assert_eq!(entry.id, "rust/scanner_installed");
        assert!(matches!(entry.kind, CheckKind::Tree(_)));
    }
}
