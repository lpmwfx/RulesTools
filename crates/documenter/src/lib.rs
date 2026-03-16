/// Serde types for documentation items and manifests.
pub mod manifest;
/// Source file parser — extracts pub items and `///` doc comments.
pub mod parser;
/// `man/` directory generator — writes JSON + Markdown.
pub mod generator;
/// Auto-documentation via AI or stub generation.
pub mod docgen;

use std::path::Path;

/// Build-time entry point — call from `build.rs`.
///
/// Scans for undocumented pub items, generates stubs,
/// emits `cargo:warning` for each documented item.
///
/// ```ignore
/// fn main() {
///     rulestools_documenter::document_project();
/// }
/// ```
pub fn document_project() {
    let root = resolve_build_root();
    let result = docgen::document_project(&root);

    for line in &result.log {
        println!("cargo:warning=rustdocumenter: {line}");
    }

    if result.generated > 0 {
        println!(
            "cargo:warning=rustdocumenter: {} item(s) documented",
            result.generated
        );
    }
}

/// Generate `man/` directory for a project.
pub fn generate_docs(root: &Path, project_name: &str) {
    let docs = parser::collect_docs(root);
    generator::generate(root, project_name, &docs);
}

/// Resolve the workspace root from `CARGO_MANIFEST_DIR`.
fn resolve_build_root() -> std::path::PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());

    // Walk up to find workspace root
    let mut dir = manifest_dir.clone();
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
            if content.contains("[workspace]") {
                return dir;
            }
        }
        if !dir.pop() {
            return manifest_dir;
        }
    }
}
