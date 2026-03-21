use std::path::{Path, PathBuf};

/// Report documentation coverage for a project (no file modification).
pub fn gen_internal(root: &Path) -> String {
    let docs = rulestools_documenter::parser::collect_docs(root);

    let total: usize = docs.iter().flat_map(|sd| &sd.items).count();
    let undoc: usize = docs.iter()
        .flat_map(|sd| &sd.items)
        .filter(|item| item.doc.is_empty())
        .count();
    let coverage = if total > 0 { ((total - undoc) as f64 / total as f64 * 100.0) as u32 } else { 100 };

    format!("rustdocumenter: {total} items, {undoc} undocumented ({coverage}% coverage)")
}

/// fn `cmd_gen`.
pub fn cmd_gen(path: &PathBuf) {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
    println!("{}", gen_internal(&root));
}
