use std::path::Path;

use crate::manifest::{Manifest, ManifestEntry, SourceDoc};

/// Generate `man/` directory with JSON + Markdown for all source docs.
pub fn generate(root: &Path, project_name: &str, docs: &[SourceDoc]) {
    let man_dir = root.join("man");
    let _ = std::fs::create_dir_all(&man_dir);

    let mut entries = Vec::new();
    let mut total_items = 0usize;
    let mut total_undoc = 0usize;

    for doc in docs {
        let item_count = doc.items.len();
        let undoc_count = doc.items.iter().filter(|i| !i.is_documented()).count();
        total_items += item_count;
        total_undoc += undoc_count;

        // Write per-file JSON
        let json_path = man_dir.join(format!("{}.json", doc.source.replace('/', "_")));
        if let Ok(json) = serde_json::to_string_pretty(doc) {
            let _ = std::fs::write(&json_path, json);
        }

        // Write per-file Markdown
        let md_path = man_dir.join(format!("{}.md", doc.source.replace('/', "_")));
        let md = render_source_md(doc);
        let _ = std::fs::write(&md_path, md);

        entries.push(ManifestEntry {
            source: doc.source.clone(),
            item_count,
            undocumented: undoc_count,
        });
    }

    // Write MANIFEST.json
    let manifest = Manifest {
        project: project_name.to_string(),
        files: entries,
    };
    if let Ok(json) = serde_json::to_string_pretty(&manifest) {
        let _ = std::fs::write(man_dir.join("MANIFEST.json"), json);
    }

    // Write MANIFEST.md
    let md = render_manifest_md(&manifest, total_items, total_undoc);
    let _ = std::fs::write(man_dir.join("MANIFEST.md"), md);

    let coverage = if total_items > 0 {
        ((total_items - total_undoc) as f64 / total_items as f64 * 100.0) as usize
    } else {
        100
    };
    eprintln!("rustdocumenter: {total_items} items, {total_undoc} undocumented ({coverage}% coverage)");
}

/// Render Markdown for one source file.
fn render_source_md(doc: &SourceDoc) -> String {
    let mut md = format!("# {}\n\n", doc.source);
    for item in &doc.items {
        md.push_str(&format!("## `{}`\n\n", item.signature));
        md.push_str(&format!("*Line {} · {}*\n\n", item.line, item.kind.label()));
        if item.is_documented() {
            md.push_str(&item.doc);
        } else {
            md.push_str("**undocumented**");
        }
        md.push_str("\n\n---\n\n");
    }
    md
}

/// Render the top-level MANIFEST.md.
fn render_manifest_md(manifest: &Manifest, total: usize, undoc: usize) -> String {
    let coverage = if total > 0 {
        ((total - undoc) as f64 / total as f64 * 100.0) as usize
    } else {
        100
    };

    let mut md = format!("# {} — Documentation Manifest\n\n", manifest.project);
    md.push_str(&format!("**Coverage:** {coverage}% ({} / {} items documented)\n\n", total - undoc, total));
    md.push_str("| Source | Items | Undocumented |\n|---|---|---|\n");

    for entry in &manifest.files {
        let status = if entry.undocumented == 0 { "ok" } else { "missing" };
        md.push_str(&format!("| {} | {} | {} {} |\n", entry.source, entry.item_count, entry.undocumented, status));
    }

    md
}
