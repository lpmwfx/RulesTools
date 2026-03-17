use std::path::Path;

use rulestools_scanner::project::ProjectKind;

/// Scaffold a full project structure for the given kind.
///
/// Creates all directories, stub source files, Cargo.toml, and proj/ files.
/// Returns a human-readable summary of what was created.
pub fn scaffold_project(root: &Path, kind: ProjectKind, name: &str) -> Result<String, String> {
    let mut created = Vec::new();

    // proj/ files (common to all kinds)
    create_proj_files(root, kind, name, &mut created)?;

    // .gitignore (per-kind: Website writes its own, others get the Rust default)
    if kind != ProjectKind::Website {
        write_if_missing(root, ".gitignore", gitignore_content(), &mut created)?;
    }

    match kind {
        ProjectKind::Tool => scaffold_tool(root, name, &mut created)?,
        ProjectKind::CliApp => scaffold_cli(root, name, &mut created)?,
        ProjectKind::Library => scaffold_library(root, name, &mut created)?,
        ProjectKind::Website => scaffold_website(root, name, &mut created)?,
        ProjectKind::SlintApp => scaffold_slint_app(root, name, &mut created)?,
        ProjectKind::Super => scaffold_workspace(root, name, &mut created)?,
    }

    Ok(format!("Scaffolded {} ({:?}):\n  {}", name, kind, created.join("\n  ")))
}

/// Create proj/ directory with standard files.
fn create_proj_files(
    root: &Path,
    kind: ProjectKind,
    name: &str,
    created: &mut Vec<String>,
) -> Result<(), String> {
    let proj = root.join("proj");
    ensure_dir(&proj, created)?;

    let kind_str = kind_to_str(kind);

    write_if_missing(
        &proj,
        "PROJECT",
        &format!(
            "# PROJECT: {name}\n\n\
             ## Identity\n\n\
             Name:    {name}\n\
             Kind:    {kind_str}\n\
             Phase:   1 (setup)\n\n\
             ## Current\n\n\
             See proj/TODO for tasks.\n"
        ),
        created,
    )?;
    write_if_missing(&proj, "TODO", "# TODO\n\n(empty — add tasks here)\n", created)?;
    write_if_missing(
        &proj,
        "RULES",
        "# RULES\n\nRun `mcp__rules__get_context` for active rules.\n",
        created,
    )?;
    write_if_missing(&proj, "FIXES", "# FIXES\n\n(no known issues)\n", created)?;
    write_if_missing(
        &proj,
        "rulestools.toml",
        &format!("[project]\nkind = \"{kind_str}\"\n"),
        created,
    )?;

    Ok(())
}

// --- Per-kind scaffolding ---

fn scaffold_tool(root: &Path, _name: &str, created: &mut Vec<String>) -> Result<(), String> {
    let src = root.join("src");
    ensure_dir(&src, created)?;
    write_if_missing(&src, "main.rs", "fn main() {\n    println!(\"Hello, world!\");\n}\n", created)?;
    Ok(())
}

fn scaffold_cli(root: &Path, name: &str, created: &mut Vec<String>) -> Result<(), String> {
    let src = root.join("src");
    ensure_dir(&src, created)?;

    let shared = src.join("shared");
    ensure_dir(&shared, created)?;

    write_if_missing(
        &src,
        "main.rs",
        &format!(
            "use clap::Parser;\n\n\
             mod shared;\n\n\
             #[derive(Parser)]\n\
             #[command(name = \"{name}\", version, about)]\n\
             struct Cli {{\n\
             }}\n\n\
             fn main() {{\n\
                 let _cli = Cli::parse();\n\
                 println!(\"{name} ready\");\n\
             }}\n"
        ),
        created,
    )?;
    write_if_missing(&shared, "mod.rs", "// Shared utilities\n", created)?;

    let doc = root.join("doc");
    ensure_dir(&doc, created)?;

    write_if_missing(
        root,
        "Cargo.toml",
        &cargo_toml_bin(name, &["clap = { version = \"4\", features = [\"derive\"] }"]),
        created,
    )?;

    Ok(())
}

fn scaffold_library(root: &Path, name: &str, created: &mut Vec<String>) -> Result<(), String> {
    let src = root.join("src");
    ensure_dir(&src, created)?;
    write_if_missing(
        &src,
        "lib.rs",
        &format!("//! {name} — library crate.\n\npub fn hello() -> &'static str {{\n    \"{name}\"\n}}\n"),
        created,
    )?;
    write_if_missing(root, "Cargo.toml", &cargo_toml_lib(name, &[]), created)?;

    Ok(())
}

fn scaffold_website(root: &Path, name: &str, created: &mut Vec<String>) -> Result<(), String> {
    let src = root.join("src");
    ensure_dir(&src, created)?;

    write_if_missing(
        root,
        "package.json",
        &format!(
            "{{\n\
             \x20 \"name\": \"{name}\",\n\
             \x20 \"version\": \"0.1.0\",\n\
             \x20 \"private\": true,\n\
             \x20 \"scripts\": {{\n\
             \x20   \"dev\": \"echo 'Add dev server command'\",\n\
             \x20   \"build\": \"echo 'Add build command'\"\n\
             \x20 }}\n\
             }}\n"
        ),
        created,
    )?;

    write_if_missing(
        root,
        "index.html",
        &format!(
            "<!DOCTYPE html>\n\
             <html lang=\"en\">\n\
             <head>\n\
             \x20 <meta charset=\"UTF-8\">\n\
             \x20 <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n\
             \x20 <title>{name}</title>\n\
             \x20 <link rel=\"stylesheet\" href=\"src/style.css\">\n\
             </head>\n\
             <body>\n\
             \x20 <h1>{name}</h1>\n\
             \x20 <script type=\"module\" src=\"src/main.js\"></script>\n\
             </body>\n\
             </html>\n"
        ),
        created,
    )?;

    write_if_missing(
        &src,
        "main.js",
        &format!("console.log('{name} ready');\n"),
        created,
    )?;

    write_if_missing(
        &src,
        "style.css",
        "*, *::before, *::after {\n\
         \x20 box-sizing: border-box;\n\
         \x20 margin: 0;\n\
         \x20 padding: 0;\n\
         }\n",
        created,
    )?;

    // Override .gitignore for web projects
    let gitignore_path = root.join(".gitignore");
    if !gitignore_path.exists() {
        std::fs::write(&gitignore_path, "node_modules/\ndist/\n.env\n")
            .map_err(|e| format!("Cannot write {}: {e}", gitignore_path.display()))?;
        created.push(format!("{}", gitignore_path.display()));
    }

    Ok(())
}

fn scaffold_slint_app(root: &Path, name: &str, created: &mut Vec<String>) -> Result<(), String> {
    let src = root.join("src");
    ensure_dir(&src, created)?;

    // Topology folders
    for folder in &["app", "core", "adapter", "gateway", "pal", "ui"] {
        let dir = src.join(folder);
        ensure_dir(&dir, created)?;
        write_if_missing(&dir, "mod.rs", &format!("//! {folder} layer.\n"), created)?;
    }

    write_if_missing(
        &src,
        "main.rs",
        &format!(
            "mod app;\nmod core;\nmod adapter;\nmod gateway;\nmod pal;\nmod ui;\n\n\
             fn main() {{\n\
                 let app = {name}::App::new().unwrap();\n\
                 app.run().unwrap();\n\
             }}\n"
        ),
        created,
    )?;

    // ui/ slint files
    let ui_dir = root.join("ui");
    ensure_dir(&ui_dir, created)?;
    write_if_missing(
        &ui_dir,
        "main.slint",
        &format!(
            "import {{ Button, VerticalBox }} from \"std-widgets.slint\";\n\n\
             export component App inherits Window {{\n\
                 title: \"{name}\";\n\
                 width: 800px;\n\
                 height: 600px;\n\n\
                 VerticalBox {{\n\
                     Button {{ text: \"Hello\"; }}\n\
                 }}\n\
             }}\n"
        ),
        created,
    )?;

    // build.rs
    write_if_missing(
        root,
        "build.rs",
        "fn main() {\n    slint_build::compile(\"ui/main.slint\").unwrap();\n}\n",
        created,
    )?;

    // Cargo.toml with slint deps
    write_if_missing(
        root,
        "Cargo.toml",
        &cargo_toml_bin(
            name,
            &[
                "slint = \"1\"",
                "[build-dependencies]",
                "slint-build = \"1\"",
            ],
        ),
        created,
    )?;

    Ok(())
}

fn scaffold_workspace(root: &Path, name: &str, created: &mut Vec<String>) -> Result<(), String> {
    // Root Cargo.toml (workspace)
    write_if_missing(
        root,
        "Cargo.toml",
        &format!(
            "[workspace]\n\
             resolver = \"2\"\n\
             members = [\"crates/*\"]\n\n\
             [workspace.package]\n\
             version = \"0.1.0\"\n\
             edition = \"2024\"\n\
             repository = \"\"\n\n\
             [workspace.dependencies]\n\
             clap = {{ version = \"4\", features = [\"derive\"] }}\n\
             serde = {{ version = \"1\", features = [\"derive\"] }}\n\
             serde_json = \"1\"\n"
        ),
        created,
    )?;

    // Workspace crates
    let crates_dir = root.join("crates");
    ensure_dir(&crates_dir, created)?;

    // app crate (binary)
    scaffold_workspace_bin(&crates_dir, "app", name, created)?;

    // core, adapter, gateway, pal, ui (library crates)
    for crate_name in &["core", "adapter", "gateway", "pal", "ui"] {
        scaffold_workspace_lib(&crates_dir, crate_name, name, created)?;
    }

    // ui/ slint files
    let ui_dir = root.join("ui");
    ensure_dir(&ui_dir, created)?;
    write_if_missing(
        &ui_dir,
        "main.slint",
        &format!(
            "import {{ Button, VerticalBox }} from \"std-widgets.slint\";\n\n\
             export component App inherits Window {{\n\
                 title: \"{name}\";\n\
                 width: 800px;\n\
                 height: 600px;\n\n\
                 VerticalBox {{\n\
                     Button {{ text: \"Hello\"; }}\n\
                 }}\n\
             }}\n"
        ),
        created,
    )?;

    // build.rs
    write_if_missing(
        root,
        "build.rs",
        "fn main() {\n    slint_build::compile(\"ui/main.slint\").unwrap();\n}\n",
        created,
    )?;

    Ok(())
}

fn scaffold_workspace_bin(
    crates_dir: &Path,
    crate_name: &str,
    project_name: &str,
    created: &mut Vec<String>,
) -> Result<(), String> {
    let crate_dir = crates_dir.join(crate_name);
    ensure_dir(&crate_dir, created)?;
    let src = crate_dir.join("src");
    ensure_dir(&src, created)?;

    write_if_missing(
        &crate_dir,
        "Cargo.toml",
        &format!(
            "[package]\n\
             name = \"{project_name}-{crate_name}\"\n\
             version.workspace = true\n\
             edition.workspace = true\n\n\
             [[bin]]\n\
             name = \"{project_name}\"\n\
             path = \"src/main.rs\"\n\n\
             [dependencies]\n\
             clap = {{ workspace = true }}\n"
        ),
        created,
    )?;

    write_if_missing(
        &src,
        "main.rs",
        &format!(
            "use clap::Parser;\n\n\
             #[derive(Parser)]\n\
             #[command(name = \"{project_name}\", version, about)]\n\
             struct Cli {{\n\
             }}\n\n\
             fn main() {{\n\
                 let _cli = Cli::parse();\n\
                 println!(\"{project_name} ready\");\n\
             }}\n"
        ),
        created,
    )?;

    Ok(())
}

fn scaffold_workspace_lib(
    crates_dir: &Path,
    crate_name: &str,
    project_name: &str,
    created: &mut Vec<String>,
) -> Result<(), String> {
    let crate_dir = crates_dir.join(crate_name);
    ensure_dir(&crate_dir, created)?;
    let src = crate_dir.join("src");
    ensure_dir(&src, created)?;

    write_if_missing(
        &crate_dir,
        "Cargo.toml",
        &format!(
            "[package]\n\
             name = \"{project_name}-{crate_name}\"\n\
             version.workspace = true\n\
             edition.workspace = true\n\n\
             [dependencies]\n"
        ),
        created,
    )?;

    write_if_missing(
        &src,
        "lib.rs",
        &format!("//! {project_name}-{crate_name} — {crate_name} layer.\n"),
        created,
    )?;

    Ok(())
}

// --- Helpers ---

fn kind_to_str(kind: ProjectKind) -> &'static str {
    match kind {
        ProjectKind::SlintApp => "slint-app",
        ProjectKind::CliApp => "cli",
        ProjectKind::Library => "library",
        ProjectKind::Website => "website",
        ProjectKind::Tool => "tool",
        ProjectKind::Super => "super",
    }
}

fn ensure_dir(dir: &Path, created: &mut Vec<String>) -> Result<(), String> {
    if !dir.exists() {
        std::fs::create_dir_all(dir).map_err(|e| format!("Cannot create {}: {e}", dir.display()))?;
        created.push(format!("{}/", dir.display()));
    }
    Ok(())
}

fn write_if_missing(
    dir: &Path,
    filename: &str,
    content: &str,
    created: &mut Vec<String>,
) -> Result<(), String> {
    let path = dir.join(filename);
    if !path.exists() {
        std::fs::write(&path, content)
            .map_err(|e| format!("Cannot write {}: {e}", path.display()))?;
        created.push(format!("{}", path.display()));
    }
    Ok(())
}

fn gitignore_content() -> &'static str {
    "/target\n*.swp\n*.swo\n*~\n.DS_Store\nThumbs.db\n"
}

fn cargo_toml_bin(name: &str, extra_deps: &[&str]) -> String {
    let mut s = format!(
        "[package]\n\
         name = \"{name}\"\n\
         version = \"0.1.0\"\n\
         edition = \"2024\"\n\n\
         [dependencies]\n"
    );
    for dep in extra_deps {
        s.push_str(dep);
        s.push('\n');
    }
    s
}

fn cargo_toml_lib(name: &str, extra_deps: &[&str]) -> String {
    let mut s = format!(
        "[package]\n\
         name = \"{name}\"\n\
         version = \"0.1.0\"\n\
         edition = \"2024\"\n\n\
         [lib]\n\
         name = \"{}\"\n\
         path = \"src/lib.rs\"\n\n\
         [dependencies]\n",
        name.replace('-', "_"),
    );
    for dep in extra_deps {
        s.push_str(dep);
        s.push('\n');
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_dir(suffix: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("rulestools-scaffold-test-{suffix}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn scaffold_tool_creates_minimal() {
        let dir = temp_dir("tool");
        let result = scaffold_project(&dir, ProjectKind::Tool, "my-tool");
        assert!(result.is_ok());
        assert!(dir.join("proj/PROJECT").exists());
        assert!(dir.join("proj/rulestools.toml").exists());
        assert!(dir.join("src/main.rs").exists());
        assert!(dir.join(".gitignore").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scaffold_cli_creates_clap_project() {
        let dir = temp_dir("cli");
        let result = scaffold_project(&dir, ProjectKind::CliApp, "my-cli");
        assert!(result.is_ok());
        assert!(dir.join("Cargo.toml").exists());
        assert!(dir.join("src/main.rs").exists());
        assert!(dir.join("src/shared/mod.rs").exists());
        assert!(dir.join("doc").exists());
        let cargo = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        assert!(cargo.contains("clap"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scaffold_library_creates_lib() {
        let dir = temp_dir("lib");
        let result = scaffold_project(&dir, ProjectKind::Library, "my-lib");
        assert!(result.is_ok());
        assert!(dir.join("src/lib.rs").exists());
        assert!(!dir.join("src/main.rs").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scaffold_slint_creates_topology() {
        let dir = temp_dir("slint");
        let result = scaffold_project(&dir, ProjectKind::SlintApp, "my-slint");
        assert!(result.is_ok());
        assert!(dir.join("src/main.rs").exists());
        assert!(dir.join("src/app/mod.rs").exists());
        assert!(dir.join("src/core/mod.rs").exists());
        assert!(dir.join("src/adapter/mod.rs").exists());
        assert!(dir.join("src/gateway/mod.rs").exists());
        assert!(dir.join("src/pal/mod.rs").exists());
        assert!(dir.join("src/ui/mod.rs").exists());
        assert!(dir.join("ui/main.slint").exists());
        assert!(dir.join("build.rs").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scaffold_workspace_creates_crates() {
        let dir = temp_dir("workspace");
        let result = scaffold_project(&dir, ProjectKind::Super, "my-ws");
        assert!(result.is_ok());
        let cargo = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        assert!(cargo.contains("[workspace]"));
        assert!(dir.join("crates/app/src/main.rs").exists());
        assert!(dir.join("crates/core/src/lib.rs").exists());
        assert!(dir.join("crates/adapter/src/lib.rs").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scaffold_website_creates_structure() {
        let dir = temp_dir("website");
        let result = scaffold_project(&dir, ProjectKind::Website, "my-site");
        assert!(result.is_ok());
        assert!(dir.join("proj/PROJECT").exists());
        assert!(dir.join("proj/rulestools.toml").exists());
        assert!(dir.join("package.json").exists());
        assert!(dir.join("index.html").exists());
        assert!(dir.join("src/main.js").exists());
        assert!(dir.join("src/style.css").exists());
        assert!(dir.join(".gitignore").exists());
        let pkg = std::fs::read_to_string(dir.join("package.json")).unwrap();
        assert!(pkg.contains("my-site"));
        let gitignore = std::fs::read_to_string(dir.join(".gitignore")).unwrap();
        assert!(gitignore.contains("node_modules"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scaffold_skips_existing_files() {
        let dir = temp_dir("existing");
        let proj = dir.join("proj");
        std::fs::create_dir_all(&proj).unwrap();
        std::fs::write(proj.join("PROJECT"), "custom content").unwrap();
        let result = scaffold_project(&dir, ProjectKind::Tool, "test").unwrap();
        // Should not overwrite existing PROJECT
        let content = std::fs::read_to_string(proj.join("PROJECT")).unwrap();
        assert_eq!(content, "custom content");
        // But should still report what was created
        assert!(!result.contains("PROJECT"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
