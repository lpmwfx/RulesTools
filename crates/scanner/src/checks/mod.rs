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
pub mod hardcoded_paths;
pub mod hardcoded_durations;
pub mod hardcoded_urls;
pub mod clone_spam;
pub mod coupling;
pub mod threading;
pub mod mother_child;
pub mod shared_discovery;
pub mod placement;
pub mod shared_guard;
pub mod unsafe_comment;
pub mod layer_violation;
pub mod sibling_import;
pub mod slint_gateway;
pub mod slint_mother_child;
pub mod topology_naming;

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
        CheckEntry::per_file("rust/constants/no-hardcoded-path", vec![Language::Rust], hardcoded_paths::check),
        CheckEntry::per_file("rust/constants/no-hardcoded-duration", vec![Language::Rust], hardcoded_durations::check),
        CheckEntry::per_file("rust/constants/no-hardcoded-url", vec![Language::Rust], hardcoded_urls::check),
        CheckEntry::per_file("rust/ownership/clone-spam", vec![Language::Rust], clone_spam::check),
        CheckEntry::per_file("rust/modules/no-sibling-coupling", vec![Language::Rust], coupling::check),
        CheckEntry::per_file("rust/threading/no-static-mut", vec![Language::Rust], threading::check),
        CheckEntry::per_file("uiux/mother-child/mother-too-many-fns", vec![Language::Rust], mother_child::check),
        CheckEntry::cross_file("rust/modules/shared-candidate", vec![Language::Rust], shared_discovery::check),
        // Topology checks
        CheckEntry::per_file("topology/placement", vec![], placement::check),
        CheckEntry::per_file("rust/modules/shared-guard", vec![Language::Rust], shared_guard::check),
        CheckEntry::per_file("rust/safety/unsafe-needs-comment", vec![Language::Rust], unsafe_comment::check),
        CheckEntry::cross_file("topology/layer-violation", vec![Language::Rust], layer_violation::check),
        CheckEntry::cross_file("rust/modules/no-sibling-import", vec![Language::Rust], sibling_import::check),
        // Slint checks
        CheckEntry::tree("uiux/state-flow/single-gateway", vec![Language::Slint], slint_gateway::check),
        CheckEntry::per_file("uiux/mother-child/child-has-state", vec![Language::Slint], slint_mother_child::check),
        // Topology naming (Level 1: folder names)
        CheckEntry::tree("topology/naming", vec![], topology_naming::check),
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
