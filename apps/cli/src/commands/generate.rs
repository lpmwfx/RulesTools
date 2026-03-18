use std::path::{Path, PathBuf};

/// Run documenter: insert /// stubs for undocumented items + generate man/ files.
///
/// Returns a summary string with coverage stats.
pub fn gen_internal(root: &Path) -> String {
    let project_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    // Phase 1: auto-insert /// stubs for undocumented pub items
    let docgen_result = rulestools_documenter::docgen::document_project(root);

    // Phase 2: generate man/ directory (JSON + Markdown)
    let docs = rulestools_documenter::parser::collect_docs(root);
    rulestools_documenter::generator::generate(root, project_name, &docs);

    // Build summary
    let total: usize = docs.iter().flat_map(|sd| &sd.items).count();
    let undoc: usize = docs.iter()
        .flat_map(|sd| &sd.items)
        .filter(|item| item.doc.is_empty())
        .count();
    let coverage = if total > 0 { ((total - undoc) as f64 / total as f64 * 100.0) as u32 } else { 100 };

    let mut out = format!("rustdocumenter: {total} items, {undoc} undocumented ({coverage}% coverage)\n");
    if docgen_result.generated > 0 {
        out.push_str(&format!("rustdocumenter: {} stub(s) inserted\n", docgen_result.generated));
    }
    out.push_str(&format!("rulestools: man/ generated for {project_name}"));
    out
}

/// fn `cmd_gen`.
pub fn cmd_gen(path: &PathBuf) {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
    println!("{}", gen_internal(&root));
}
