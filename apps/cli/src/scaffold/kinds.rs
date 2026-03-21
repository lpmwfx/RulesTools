use std::path::Path;

use rulestools_scanner::project::ProjectKind;

use super::types::Writer;
use super::templates::*;
use super::proj_files::create_proj_files;

pub(super) fn scaffold_kind(
    w: &Writer,
    root: &Path,
    kind: ProjectKind,
    name: &str,
    created: &mut Vec<String>,
) -> Result<(), String> {
    match kind {
        ProjectKind::Tool => scaffold_tool(w, root, name, created),
        ProjectKind::CliApp => scaffold_cli(w, root, name, created),
        ProjectKind::Library => scaffold_library(w, root, name, created),
        ProjectKind::Website => scaffold_website(w, root, name, created),
        ProjectKind::SlintApp => scaffold_slint_app(w, root, name, created),
        ProjectKind::Super => scaffold_workspace(w, root, name, created),
    }
}

fn scaffold_tool(w: &Writer, root: &Path, name: &str, created: &mut Vec<String>) -> Result<(), String> {
    let src = root.join("src");
    w.ensure_dir(&src, created)?;

    // Topology folders
    for folder in CLI_TOPOLOGY {
        let dir = src.join(folder);
        w.ensure_dir(&dir, created)?;
        w.write_if_missing(&dir, "mod.rs", &format!("//! {folder} layer.\n"), created)?;
    }

    w.write_if_missing(
        &src,
        "main.rs",
        &format!(
            "mod core;\nmod adapter;\nmod gateway;\nmod pal;\nmod shared;\n\n\
             fn main() {{\n\
                 println!(\"{name} ready\");\n\
             }}\n"
        ),
        created,
    )?;
    w.write_if_missing(
        root,
        "Cargo.toml",
        &cargo_toml_bin(name, &[], &[SCANNER_BUILD_DEP]),
        created,
    )?;
    w.write_if_missing(root, "build.rs", BUILD_RS_SCANNER, created)?;
    Ok(())
}

fn scaffold_cli(
    w: &Writer,
    root: &Path,
    name: &str,
    created: &mut Vec<String>,
) -> Result<(), String> {
    let src = root.join("src");
    w.ensure_dir(&src, created)?;

    // Topology folders
    for folder in CLI_TOPOLOGY {
        let dir = src.join(folder);
        w.ensure_dir(&dir, created)?;
        w.write_if_missing(&dir, "mod.rs", &format!("//! {folder} layer.\n"), created)?;
    }

    w.write_if_missing(
        &src,
        "main.rs",
        &format!(
            "use clap::Parser;\n\n\
             mod core;\nmod adapter;\nmod gateway;\nmod pal;\nmod shared;\n\n\
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

    let doc = root.join("doc");
    w.ensure_dir(&doc, created)?;

    w.write_if_missing(
        root,
        "Cargo.toml",
        &cargo_toml_bin(name, &["clap = { version = \"4\", features = [\"derive\"] }"], &[SCANNER_BUILD_DEP]),
        created,
    )?;

    w.write_if_missing(root, "build.rs", BUILD_RS_SCANNER, created)?;

    Ok(())
}

fn scaffold_library(
    w: &Writer,
    root: &Path,
    name: &str,
    created: &mut Vec<String>,
) -> Result<(), String> {
    let src = root.join("src");
    w.ensure_dir(&src, created)?;

    // Topology folders (lib = minimal: core + shared)
    for folder in &["core", "shared"] {
        let dir = src.join(folder);
        w.ensure_dir(&dir, created)?;
        w.write_if_missing(&dir, "mod.rs", &format!("//! {folder} layer.\n"), created)?;
    }

    w.write_if_missing(
        &src,
        "lib.rs",
        &format!(
            "//! {name} — library crate.\n\nmod core;\nmod shared;\n\npub fn hello() -> &'static str {{\n    \"{name}\"\n}}\n"
        ),
        created,
    )?;
    w.write_if_missing(root, "Cargo.toml", &cargo_toml_lib(name, &[], &[SCANNER_BUILD_DEP]), created)?;
    w.write_if_missing(root, "build.rs", BUILD_RS_SCANNER, created)?;

    Ok(())
}

fn scaffold_website(
    w: &Writer,
    root: &Path,
    name: &str,
    created: &mut Vec<String>,
) -> Result<(), String> {
    let src = root.join("src");
    w.ensure_dir(&src, created)?;

    w.write_if_missing(
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

    w.write_if_missing(
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

    w.write_if_missing(
        &src,
        "main.js",
        &format!("console.log('{name} ready');\n"),
        created,
    )?;

    w.write_if_missing(
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
        if !w.dry_run {
            std::fs::write(&gitignore_path, "node_modules/\ndist/\n.env\n")
                .map_err(|e| format!("Cannot write {}: {e}", gitignore_path.display()))?;
        }
        created.push(format!("{}", gitignore_path.display()));
    }

    Ok(())
}

fn scaffold_slint_app(
    w: &Writer,
    root: &Path,
    name: &str,
    created: &mut Vec<String>,
) -> Result<(), String> {
    let src = root.join("src");
    w.ensure_dir(&src, created)?;

    // Topology folders (GUI: CLI topology + ui)
    for folder in GUI_TOPOLOGY {
        let dir = src.join(folder);
        w.ensure_dir(&dir, created)?;
        w.write_if_missing(&dir, "mod.rs", &format!("//! {folder} layer.\n"), created)?;
    }

    w.write_if_missing(
        &src,
        "main.rs",
        &format!(
            "mod app;\nmod core;\nmod adapter;\nmod gateway;\nmod pal;\nmod shared;\nmod ui;\n\n\
             fn main() {{\n\
                 let app = {name}::App::new().unwrap();\n\
                 app.run().unwrap();\n\
             }}\n"
        ),
        created,
    )?;

    // ui/ slint files
    let ui_dir = root.join("ui");
    w.ensure_dir(&ui_dir, created)?;
    w.write_if_missing(&ui_dir, "main.slint", &slint_main_content(name), created)?;

    // build.rs
    w.write_if_missing(root, "build.rs", BUILD_RS_SCANNER_SLINT, created)?;

    // Cargo.toml with slint deps
    w.write_if_missing(
        root,
        "Cargo.toml",
        &cargo_toml_bin(
            name,
            &["slint = \"1\""],
            &["slint-build = \"1\"", SCANNER_BUILD_DEP],
        ),
        created,
    )?;

    Ok(())
}

fn scaffold_workspace(
    w: &Writer,
    root: &Path,
    name: &str,
    created: &mut Vec<String>,
) -> Result<(), String> {
    // Root Cargo.toml (workspace)
    w.write_if_missing(
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

    // Workspace crates = topology
    let crates_dir = root.join("crates");
    w.ensure_dir(&crates_dir, created)?;

    // app crate (binary entry point)
    scaffold_workspace_bin(w, &crates_dir, "app", name, created)?;

    // CLI topology: core, adapter, gateway, pal, shared (library crates)
    for crate_name in CLI_TOPOLOGY {
        scaffold_workspace_lib(w, &crates_dir, crate_name, name, created)?;
    }

    // build.rs (scanner only — no slint)
    w.write_if_missing(root, "build.rs", BUILD_RS_SCANNER, created)?;

    Ok(())
}

pub(super) fn scaffold_workspace_bin(
    w: &Writer,
    crates_dir: &Path,
    crate_name: &str,
    project_name: &str,
    created: &mut Vec<String>,
) -> Result<(), String> {
    let crate_dir = crates_dir.join(crate_name);
    w.ensure_dir(&crate_dir, created)?;
    let src = crate_dir.join("src");
    w.ensure_dir(&src, created)?;

    w.write_if_missing(
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

    w.write_if_missing(
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

pub(super) fn scaffold_workspace_lib(
    w: &Writer,
    crates_dir: &Path,
    crate_name: &str,
    project_name: &str,
    created: &mut Vec<String>,
) -> Result<(), String> {
    let crate_dir = crates_dir.join(crate_name);
    w.ensure_dir(&crate_dir, created)?;
    let src = crate_dir.join("src");
    w.ensure_dir(&src, created)?;

    w.write_if_missing(
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

    w.write_if_missing(
        &src,
        "lib.rs",
        &format!("//! {project_name}-{crate_name} — {crate_name} layer.\n"),
        created,
    )?;

    Ok(())
}

/// Add Slint UI layer to a workspace (only when platforms specified).
pub(super) fn scaffold_workspace_ui(
    w: &Writer,
    root: &Path,
    name: &str,
    created: &mut Vec<String>,
) -> Result<(), String> {
    let crates_dir = root.join("crates");

    // crates/ui/ lib crate
    scaffold_workspace_lib(w, &crates_dir, "ui", name, created)?;

    // ui/ slint files
    let ui_dir = root.join("ui");
    w.ensure_dir(&ui_dir, created)?;
    w.write_if_missing(&ui_dir, "main.slint", &slint_main_content(name), created)?;

    // Overwrite build.rs to include slint_build
    let build_rs = root.join("build.rs");
    if build_rs.exists() {
        let content = std::fs::read_to_string(&build_rs).unwrap_or_default();
        if !content.contains("slint_build") {
            if !w.dry_run {
                std::fs::write(&build_rs, BUILD_RS_SCANNER_SLINT)
                    .map_err(|e| format!("Cannot write build.rs: {e}"))?;
            }
            created.push(format!("{} (updated)", build_rs.display()));
        }
    } else {
        w.write_if_missing(root, "build.rs", BUILD_RS_SCANNER_SLINT, created)?;
    }

    // proj/UIUX for workspace with UI
    let proj = root.join("proj");
    w.write_if_missing(
        &proj,
        "UIUX",
        &format!(
            "# UIUX: {name}\n\n\
             ## Goal\n\n\
             (Define the UI/UX vision.)\n\n\
             ## Platform\n\n\
             - Toolkit: Slint 1.x (workspace)\n\
             - Entry: ui/main.slint\n\n\
             ## UI Architecture\n\n\
             - Entry point: crates/app/ → ui/main.slint\n\
             - Topology: crates/app → crates/adapter → crates/core, crates/adapter → crates/pal\n\
             - UI layer: crates/ui/ (Rust) + ui/ (Slint)\n"
        ),
        created,
    )?;

    Ok(())
}
