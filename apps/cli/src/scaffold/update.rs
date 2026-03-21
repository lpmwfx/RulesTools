use std::path::Path;

use rulestools_scanner::project::{ProjectIdentity, ProjectKind};

use super::types::{ScaffoldResult, UpdateOptions, Writer};
use super::templates::*;
use super::features::{scaffold_platforms, scaffold_themes, scaffold_extras};
use super::kinds::scaffold_workspace_lib;

/// Update an existing project — add features within current kind.
///
/// Also checks for missing integration components (hooks, build.rs, topology)
/// and adds them if absent.
pub fn update_project(root: &Path, opts: &UpdateOptions) -> Result<ScaffoldResult, String> {
    let identity = ProjectIdentity::detect(root);
    let w = Writer {
        dry_run: opts.preview,
    };
    let mut created = Vec::new();
    let mut skipped = Vec::new();

    // --- Integrity checks: add missing components ---
    let project_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    // .claude/settings.json
    let claude_dir = root.join(".claude");
    w.ensure_dir(&claude_dir, &mut created)?;
    w.write_if_missing(&claude_dir, "settings.json", CLAUDE_SETTINGS, &mut created)?;

    // Pre-commit hook
    let hooks_dir = root.join(".git").join("hooks");
    if root.join(".git").exists() {
        w.ensure_dir(&hooks_dir, &mut created)?;
        let hook_path = hooks_dir.join("pre-commit");
        if !hook_path.exists() {
            if !w.dry_run {
                std::fs::write(&hook_path, PRE_COMMIT_HOOK)
                    .map_err(|e| format!("Cannot write pre-commit hook: {e}"))?;
            }
            created.push(format!("{}", hook_path.display()));
        }
    }

    // proj/rulestools.toml — ensure [project].kind exists
    let toml_path = root.join("proj").join("rulestools.toml");
    if toml_path.exists() {
        let content = std::fs::read_to_string(&toml_path).unwrap_or_default();
        if !content.contains("kind") {
            if !w.dry_run {
                let new_content = format!("[project]\nkind = \"{}\"\n\n{content}", identity.kind.as_str());
                std::fs::write(&toml_path, new_content)
                    .map_err(|e| format!("Cannot update rulestools.toml: {e}"))?;
            }
            created.push(format!("{} (added [project].kind)", toml_path.display()));
        }
    }

    // build.rs — ensure it exists with scanner
    let build_rs = root.join("build.rs");
    if !build_rs.exists() && root.join("Cargo.toml").exists() {
        let is_slint = identity.kind == ProjectKind::SlintApp
            || (identity.kind == ProjectKind::Super && root.join("ui").exists());
        let content = if is_slint { BUILD_RS_SCANNER_SLINT } else { BUILD_RS_SCANNER };
        if !w.dry_run {
            std::fs::write(&build_rs, content)
                .map_err(|e| format!("Cannot write build.rs: {e}"))?;
        }
        created.push(format!("{}", build_rs.display()));
    }

    // Topology folders (SlintApp/Super only)
    if identity.kind == ProjectKind::SlintApp {
        let src = root.join("src");
        for folder in &["app", "core", "adapter", "gateway", "pal", "ui", "shared", "state"] {
            let dir = src.join(folder);
            w.ensure_dir(&dir, &mut created)?;
            w.write_if_missing(&dir, "mod.rs", &format!("//! {folder} layer.\n"), &mut created)?;
        }
    }

    // --- Feature additions (user-requested) ---

    // Platforms (SlintApp/Super only)
    if !opts.platforms.is_empty() {
        if identity.kind == ProjectKind::SlintApp || identity.kind == ProjectKind::Super {
            scaffold_platforms(&w, root, identity.kind, &opts.platforms, &mut created)?;
        } else {
            skipped.push(format!(
                "Platforms ignored — only SlintApp/Super support platforms (detected: {:?})",
                identity.kind
            ));
        }
    }

    // Themes (SlintApp/Super only)
    if !opts.themes.is_empty() {
        if identity.kind == ProjectKind::SlintApp || identity.kind == ProjectKind::Super {
            scaffold_themes(&w, root, identity.kind, &opts.themes, &mut created)?;
        } else {
            skipped.push(format!(
                "Themes ignored — only SlintApp/Super support themes (detected: {:?})",
                identity.kind
            ));
        }
    }

    // Add crate (workspace only)
    if let Some(ref crate_name) = opts.crate_name {
        if identity.kind == ProjectKind::Super {
            let crates_dir = root.join("crates");
            scaffold_workspace_lib(&w, &crates_dir, crate_name, project_name, &mut created)?;
        } else {
            skipped.push(format!(
                "Crate '{}' ignored — only workspace projects support add-crate (detected: {:?})",
                crate_name, identity.kind
            ));
        }
    }

    // Extras
    scaffold_extras(&w, root, identity.kind, project_name, &opts.folders, &mut created)?;

    let summary = if opts.preview {
        format!(
            "Preview — update {:?}: {} files would be created",
            identity.kind,
            created.len()
        )
    } else {
        format!(
            "Updated {:?}: {} files created",
            identity.kind,
            created.len()
        )
    };

    Ok(ScaffoldResult {
        created,
        skipped,
        summary,
    })
}
