mod inserter;

use std::path::Path;

use crate::parser;

/// Result of auto-documentation generation.
pub struct DocGenResult {
    pub generated: usize,
    pub skipped: usize,
    pub log: Vec<String>,
}

/// Run auto-documentation on a project: find undocumented items, call AI, insert `///`.
pub fn document_project(root: &Path) -> DocGenResult {
    let docs = parser::collect_docs(root);
    inserter::process_all(root, &docs)
}
