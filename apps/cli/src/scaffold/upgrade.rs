use std::path::Path;

use rulestools_scanner::project::{ProjectIdentity, ProjectKind};

use super::types::{MoveGuidance, UpgradeResult, Writer};
use super::templates::*;
use super::kinds::{scaffold_workspace_bin, scaffold_workspace_lib};

/// Upgrade a project to a higher kind.
pub fn upgrade_project(
    root: &Path,
    to_kind: ProjectKind,
    preview: bool,
) -> Result<UpgradeResult, String> {
    let identity = ProjectIdentity::detect(root);
    let from = identity.kind;

    // Validate upgrade direction
    if to_kind == from {
        return Err(format!(
            "Already {:?} — nothing to upgrade",
            from
        ));
    }
    if to_kind.upgrade_ord() <= from.upgrade_ord() {
        return Err(format!(
            "Cannot downgrade from {:?} to {:?}",
            from, to_kind
        ));
    }

    let w = Writer { dry_run: preview };
    let mut created = Vec::new();
    let mut move_guidance = Vec::new();
    let mut manual_steps = Vec::new();

    let project_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();

    // Generate upgrade-specific scaffolding and guidance
    match (from, to_kind) {
        (ProjectKind::Tool, ProjectKind::CliApp) => {
            upgrade_tool_to_cli(&w, root, &project_name, &mut created, &mut manual_steps)?;
        }
        (ProjectKind::Tool, ProjectKind::SlintApp) => {
            upgrade_tool_to_slint(&w, root, &project_name, &mut created, &mut move_guidance, &mut manual_steps)?;
        }
        (ProjectKind::Tool, ProjectKind::Super) => {
            upgrade_to_workspace(&w, root, &project_name, from, &mut created, &mut move_guidance, &mut manual_steps)?;
        }
        (ProjectKind::Library, ProjectKind::CliApp) => {
            upgrade_library_to_cli(&w, root, &project_name, &mut created, &mut manual_steps)?;
        }
        (ProjectKind::Library, ProjectKind::SlintApp) => {
            upgrade_to_slint(&w, root, &project_name, &mut created, &mut move_guidance, &mut manual_steps)?;
        }
        (ProjectKind::Library, ProjectKind::Super) => {
            upgrade_to_workspace(&w, root, &project_name, from, &mut created, &mut move_guidance, &mut manual_steps)?;
        }
        (ProjectKind::CliApp, ProjectKind::SlintApp) => {
            upgrade_cli_to_slint(&w, root, &project_name, &mut created, &mut move_guidance, &mut manual_steps)?;
        }
        (ProjectKind::CliApp, ProjectKind::Super) => {
            upgrade_to_workspace(&w, root, &project_name, from, &mut created, &mut move_guidance, &mut manual_steps)?;
        }
        (ProjectKind::SlintApp, ProjectKind::Super) => {
            upgrade_to_workspace(&w, root, &project_name, from, &mut created, &mut move_guidance, &mut manual_steps)?;
        }
        _ => {
            return Err(format!(
                "Upgrade path {:?} -> {:?} is not supported",
                from, to_kind
            ));
        }
    }

    // Update proj/rulestools.toml kind
    let toml_path = root.join("proj").join("rulestools.toml");
    if toml_path.exists() && !preview {
        let content = format!("[project]\nkind = \"{}\"\n", to_kind.as_str());
        std::fs::write(&toml_path, content)
            .map_err(|e| format!("Cannot update rulestools.toml: {e}"))?;
        created.push(format!("{} (updated)", toml_path.display()));
    } else if !toml_path.exists() {
        w.ensure_dir(&root.join("proj"), &mut created)?;
        w.write_if_missing(
            &root.join("proj"),
            "rulestools.toml",
            &format!("[project]\nkind = \"{}\"\n", to_kind.as_str()),
            &mut created,
        )?;
    }

    Ok(UpgradeResult {
        from_kind: from,
        to_kind,
        created,
        move_guidance,
        manual_steps,
    })
}

/// Render a simple directory tree from a list of created paths.
pub fn render_tree(root_name: &str, paths: &[String]) -> String {
    let mut lines = vec![format!("{root_name}/")];
    let mut sorted: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    sorted.sort();

    for (i, path) in sorted.iter().enumerate() {
        let is_last = i == sorted.len() - 1;
        let prefix = if is_last { "└── " } else { "├── " };
        // Show relative path after root
        let display = path
            .replace('\\', "/")
            .rsplit_once('/')
            .map(|(_, f)| f.to_string())
            .unwrap_or_else(|| path.to_string());
        lines.push(format!("{prefix}{display}"));
    }
    lines.join("\n")
}

// --- Upgrade helpers ---

fn upgrade_tool_to_cli(
    w: &Writer,
    root: &Path,
    name: &str,
    created: &mut Vec<String>,
    manual: &mut Vec<String>,
) -> Result<(), String> {
    let src = root.join("src");
    w.ensure_dir(&src, created)?;

    let shared = src.join("shared");
    w.ensure_dir(&shared, created)?;
    w.write_if_missing(&shared, "mod.rs", "// Shared utilities\n", created)?;

    let doc = root.join("doc");
    w.ensure_dir(&doc, created)?;

    // Cargo.toml — only if missing
    w.write_if_missing(
        root,
        "Cargo.toml",
        &cargo_toml_bin(name, &["clap = { version = \"4\", features = [\"derive\"] }"], &[SCANNER_BUILD_DEP]),
        created,
    )?;

    w.write_if_missing(root, "build.rs", BUILD_RS_SCANNER, created)?;

    manual.push("Add `use clap::Parser;` and `#[derive(Parser)] struct Cli {}` to main.rs".into());
    manual.push("Add `clap` dependency to Cargo.toml if not present".into());
    Ok(())
}

fn upgrade_library_to_cli(
    w: &Writer,
    root: &Path,
    name: &str,
    created: &mut Vec<String>,
    manual: &mut Vec<String>,
) -> Result<(), String> {
    let src = root.join("src");
    w.ensure_dir(&src, created)?;
    w.write_if_missing(
        &src,
        "main.rs",
        &format!(
            "use clap::Parser;\n\n\
             #[derive(Parser)]\n\
             #[command(name = \"{name}\", version, about)]\n\
             struct Cli {{\n\
             }}\n\n\
             fn main() {{\n\
                 let _cli = Cli::parse();\n\
             }}\n"
        ),
        created,
    )?;

    let shared = src.join("shared");
    w.ensure_dir(&shared, created)?;
    w.write_if_missing(&shared, "mod.rs", "// Shared utilities\n", created)?;

    let doc = root.join("doc");
    w.ensure_dir(&doc, created)?;

    manual.push("Add `clap` dependency to Cargo.toml".into());
    manual.push("Add `[[bin]]` section to Cargo.toml if using lib.rs + main.rs".into());
    Ok(())
}

fn upgrade_to_slint(
    w: &Writer,
    root: &Path,
    name: &str,
    created: &mut Vec<String>,
    guidance: &mut Vec<MoveGuidance>,
    manual: &mut Vec<String>,
) -> Result<(), String> {
    let src = root.join("src");
    w.ensure_dir(&src, created)?;

    // Topology folders
    for folder in &["app", "core", "adapter", "gateway", "pal", "ui"] {
        let dir = src.join(folder);
        w.ensure_dir(&dir, created)?;
        w.write_if_missing(&dir, "mod.rs", &format!("//! {folder} layer.\n"), created)?;
    }

    // ui/ slint files
    let ui_dir = root.join("ui");
    w.ensure_dir(&ui_dir, created)?;
    w.write_if_missing(
        &ui_dir,
        "main.slint",
        &slint_main_content(name),
        created,
    )?;

    w.write_if_missing(root, "build.rs", BUILD_RS_SCANNER_SLINT, created)?;

    guidance.push(MoveGuidance {
        from: "src/lib.rs".into(),
        to: "src/core/mod.rs".into(),
        reason: "Library logic becomes core layer".into(),
    });

    manual.push("Add `slint` + `slint-build` dependencies to Cargo.toml".into());
    manual.push("Add mod declarations for topology layers to main.rs".into());
    Ok(())
}

fn upgrade_tool_to_slint(
    w: &Writer,
    root: &Path,
    name: &str,
    created: &mut Vec<String>,
    guidance: &mut Vec<MoveGuidance>,
    manual: &mut Vec<String>,
) -> Result<(), String> {
    guidance.push(MoveGuidance {
        from: "src/main.rs".into(),
        to: "src/core/".into(),
        reason: "Move logic from main.rs to core layer".into(),
    });
    upgrade_to_slint(w, root, name, created, guidance, manual)
}

fn upgrade_cli_to_slint(
    w: &Writer,
    root: &Path,
    name: &str,
    created: &mut Vec<String>,
    guidance: &mut Vec<MoveGuidance>,
    manual: &mut Vec<String>,
) -> Result<(), String> {
    let src = root.join("src");
    w.ensure_dir(&src, created)?;

    // Topology folders
    for folder in &["app", "core", "adapter", "gateway", "pal", "ui"] {
        let dir = src.join(folder);
        w.ensure_dir(&dir, created)?;
        w.write_if_missing(&dir, "mod.rs", &format!("//! {folder} layer.\n"), created)?;
    }

    // ui/ slint files
    let ui_dir = root.join("ui");
    w.ensure_dir(&ui_dir, created)?;
    w.write_if_missing(
        &ui_dir,
        "main.slint",
        &slint_main_content(name),
        created,
    )?;

    w.write_if_missing(root, "build.rs", BUILD_RS_SCANNER_SLINT, created)?;

    guidance.push(MoveGuidance {
        from: "src/shared/".into(),
        to: "src/core/".into(),
        reason: "Shared utilities become core layer".into(),
    });
    guidance.push(MoveGuidance {
        from: "CLI logic in main.rs".into(),
        to: "src/app/".into(),
        reason: "CLI dispatch becomes app layer".into(),
    });

    manual.push("Add `slint` + `slint-build` dependencies to Cargo.toml".into());
    manual.push("Add mod declarations for topology layers to main.rs".into());
    manual.push("Add UI entry point in src/app/".into());
    Ok(())
}

fn upgrade_to_workspace(
    w: &Writer,
    root: &Path,
    name: &str,
    from: ProjectKind,
    created: &mut Vec<String>,
    guidance: &mut Vec<MoveGuidance>,
    manual: &mut Vec<String>,
) -> Result<(), String> {
    let crates_dir = root.join("crates");
    w.ensure_dir(&crates_dir, created)?;

    // App crate (binary)
    scaffold_workspace_bin(w, &crates_dir, "app", name, created)?;

    // Core, adapter, gateway, pal, ui
    for crate_name in &["core", "adapter", "gateway", "pal", "ui"] {
        scaffold_workspace_lib(w, &crates_dir, crate_name, name, created)?;
    }

    // ui/ slint files
    let ui_dir = root.join("ui");
    w.ensure_dir(&ui_dir, created)?;
    w.write_if_missing(
        &ui_dir,
        "main.slint",
        &slint_main_content(name),
        created,
    )?;

    w.write_if_missing(root, "build.rs", BUILD_RS_SCANNER_SLINT, created)?;

    // Move guidance based on source kind
    match from {
        ProjectKind::Tool => {
            guidance.push(MoveGuidance {
                from: "src/main.rs".into(),
                to: "crates/app/src/main.rs".into(),
                reason: "Main entry point moves to app crate".into(),
            });
        }
        ProjectKind::CliApp => {
            guidance.push(MoveGuidance {
                from: "src/main.rs".into(),
                to: "crates/app/src/main.rs".into(),
                reason: "CLI entry point moves to app crate".into(),
            });
            guidance.push(MoveGuidance {
                from: "src/shared/".into(),
                to: "crates/core/src/".into(),
                reason: "Shared code moves to core crate".into(),
            });
        }
        ProjectKind::Library => {
            guidance.push(MoveGuidance {
                from: "src/lib.rs".into(),
                to: "crates/core/src/lib.rs".into(),
                reason: "Library code moves to core crate".into(),
            });
        }
        ProjectKind::SlintApp => {
            guidance.push(MoveGuidance {
                from: "src/main.rs".into(),
                to: "crates/app/src/main.rs".into(),
                reason: "Main entry point moves to app crate".into(),
            });
            guidance.push(MoveGuidance {
                from: "src/core/".into(),
                to: "crates/core/src/".into(),
                reason: "Core layer moves to core crate".into(),
            });
            guidance.push(MoveGuidance {
                from: "src/pal/".into(),
                to: "crates/pal/src/".into(),
                reason: "PAL layer moves to pal crate".into(),
            });
        }
        _ => {}
    }

    manual.push("Replace root Cargo.toml with [workspace] definition".into());
    manual.push("Update member paths in workspace Cargo.toml".into());
    manual.push("Move source files according to guidance above".into());
    Ok(())
}
