use std::path::PathBuf;

pub fn cmd_gen(path: &PathBuf) {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
    let project_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
    rulestools_documenter::generate_docs(&root, project_name);
    println!("rulestools: man/ generated for {project_name}");
}
