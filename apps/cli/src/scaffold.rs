use std::path::Path;

use rulestools_scanner::project::{ProjectIdentity, ProjectKind};

// --- Types ---

/// Target platform for SlintApp/Super projects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    Desktop,
    Mobile,
    Small,
}

impl Platform {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "desktop" => Some(Self::Desktop),
            "mobile" => Some(Self::Mobile),
            "small" => Some(Self::Small),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Mobile => "mobile",
            Self::Small => "small",
        }
    }
}

/// Extra folders/crates to scaffold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Extra {
    Lib,
    Shared,
    Doc,
}

impl Extra {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "lib" => Some(Self::Lib),
            "shared" => Some(Self::Shared),
            "doc" => Some(Self::Doc),
            _ => None,
        }
    }
}

/// Options for `rulestools new`.
pub struct ScaffoldOptions {
    pub name: String,
    pub kind: ProjectKind,
    pub platforms: Vec<Platform>,
    pub themes: Vec<String>,
    pub mcp: bool,
    pub extras: Vec<Extra>,
    pub preview: bool,
}

/// Result of a scaffold/update operation.
pub struct ScaffoldResult {
    pub created: Vec<String>,
    pub skipped: Vec<String>,
    pub summary: String,
}

/// Options for `rulestools update`.
pub struct UpdateOptions {
    pub platforms: Vec<Platform>,
    pub themes: Vec<String>,
    pub crate_name: Option<String>,
    pub folders: Vec<Extra>,
    pub preview: bool,
}

/// Move guidance for project upgrades.
#[derive(Debug)]
pub struct MoveGuidance {
    pub from: String,
    pub to: String,
    pub reason: String,
}

/// Result of a project upgrade.
#[derive(Debug)]
pub struct UpgradeResult {
    pub from_kind: ProjectKind,
    pub to_kind: ProjectKind,
    pub created: Vec<String>,
    pub move_guidance: Vec<MoveGuidance>,
    pub manual_steps: Vec<String>,
}

// --- Writer (dry_run support) ---

struct Writer {
    dry_run: bool,
}

impl Writer {
    fn ensure_dir(&self, dir: &Path, created: &mut Vec<String>) -> Result<(), String> {
        if !dir.exists() {
            if !self.dry_run {
                std::fs::create_dir_all(dir)
                    .map_err(|e| format!("Cannot create {}: {e}", dir.display()))?;
            }
            created.push(format!("{}/", dir.display()));
        }
        Ok(())
    }

    fn write_if_missing(
        &self,
        dir: &Path,
        filename: &str,
        content: &str,
        created: &mut Vec<String>,
    ) -> Result<(), String> {
        let path = dir.join(filename);
        if !path.exists() {
            if !self.dry_run {
                std::fs::write(&path, content)
                    .map_err(|e| format!("Cannot write {}: {e}", path.display()))?;
            }
            created.push(format!("{}", path.display()));
        }
        Ok(())
    }

}

// --- Public API ---

/// Scaffold a full project structure for the given kind (backward compat).
pub fn scaffold_project(root: &Path, kind: ProjectKind, name: &str) -> Result<String, String> {
    let w = Writer { dry_run: false };
    let mut created = Vec::new();

    create_proj_files(&w, root, kind, name, &mut created)?;

    if kind != ProjectKind::Website {
        w.write_if_missing(root, ".gitignore", gitignore_content(), &mut created)?;
    }

    scaffold_kind(&w, root, kind, name, &mut created)?;

    Ok(format!(
        "Scaffolded {} ({:?}):\n  {}",
        name,
        kind,
        created.join("\n  ")
    ))
}

/// Scaffold a project with full options (new_project).
pub fn scaffold_with_options(root: &Path, opts: &ScaffoldOptions) -> Result<ScaffoldResult, String> {
    let w = Writer {
        dry_run: opts.preview,
    };
    let mut created = Vec::new();

    // Base scaffold
    create_proj_files(&w, root, opts.kind, &opts.name, &mut created)?;

    if opts.kind != ProjectKind::Website {
        w.write_if_missing(root, ".gitignore", gitignore_content(), &mut created)?;
    }

    scaffold_kind(&w, root, opts.kind, &opts.name, &mut created)?;

    // Platform scaffolding (SlintApp/Super only)
    if !opts.platforms.is_empty() {
        scaffold_platforms(&w, root, opts.kind, &opts.platforms, &mut created)?;
    }

    // Theme scaffolding (SlintApp/Super only)
    if !opts.themes.is_empty() {
        scaffold_themes(&w, root, opts.kind, &opts.themes, &mut created)?;
    }

    // MCP crate (workspace only)
    if opts.mcp {
        scaffold_mcp_crate(&w, root, opts.kind, &opts.name, &mut created)?;
    }

    // Extras
    scaffold_extras(&w, root, opts.kind, &opts.name, &opts.extras, &mut created)?;

    let summary = if opts.preview {
        format!(
            "Preview — {} ({:?}): {} files would be created",
            opts.name,
            opts.kind,
            created.len()
        )
    } else {
        format!(
            "Scaffolded {} ({:?}): {} files created",
            opts.name,
            opts.kind,
            created.len()
        )
    };

    Ok(ScaffoldResult {
        created,
        skipped: Vec::new(),
        summary,
    })
}

/// Update an existing project — add features within current kind.
pub fn update_project(root: &Path, opts: &UpdateOptions) -> Result<ScaffoldResult, String> {
    let identity = ProjectIdentity::detect(root);
    let w = Writer {
        dry_run: opts.preview,
    };
    let mut created = Vec::new();
    let mut skipped = Vec::new();

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
            let project_name = root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project");
            scaffold_workspace_lib(&w, &crates_dir, crate_name, project_name, &mut created)?;
        } else {
            skipped.push(format!(
                "Crate '{}' ignored — only workspace projects support add-crate (detected: {:?})",
                crate_name, identity.kind
            ));
        }
    }

    // Extras
    let project_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
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
        &cargo_toml_bin(name, &["clap = { version = \"4\", features = [\"derive\"] }"]),
        created,
    )?;

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

    w.write_if_missing(
        root,
        "build.rs",
        "fn main() {\n    slint_build::compile(\"ui/main.slint\").unwrap();\n}\n",
        created,
    )?;

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

    w.write_if_missing(
        root,
        "build.rs",
        "fn main() {\n    slint_build::compile(\"ui/main.slint\").unwrap();\n}\n",
        created,
    )?;

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

    w.write_if_missing(
        root,
        "build.rs",
        "fn main() {\n    slint_build::compile(\"ui/main.slint\").unwrap();\n}\n",
        created,
    )?;

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

// --- Platform scaffolding ---

fn scaffold_platforms(
    w: &Writer,
    root: &Path,
    kind: ProjectKind,
    platforms: &[Platform],
    created: &mut Vec<String>,
) -> Result<(), String> {
    if kind != ProjectKind::SlintApp && kind != ProjectKind::Super {
        return Ok(());
    }

    let is_workspace = kind == ProjectKind::Super
        || root.join("crates").exists();

    for platform in platforms {
        if is_workspace {
            let pal_src = root.join("crates").join("pal").join("src");
            w.ensure_dir(&pal_src, created)?;
            w.write_if_missing(
                &pal_src,
                &format!("{}.rs", platform.name()),
                &format!(
                    "//! Platform abstraction — {}.\n\n\
                     /// Initialize {} platform.\n\
                     pub fn init() {{\n\
                         // TODO: platform-specific initialization\n\
                     }}\n",
                    platform.name(),
                    platform.name()
                ),
                created,
            )?;
        } else {
            let pal_dir = root.join("src").join("pal");
            w.ensure_dir(&pal_dir, created)?;
            w.write_if_missing(
                &pal_dir,
                &format!("{}.rs", platform.name()),
                &format!(
                    "//! Platform abstraction — {}.\n\n\
                     /// Initialize {} platform.\n\
                     pub fn init() {{\n\
                         // TODO: platform-specific initialization\n\
                     }}\n",
                    platform.name(),
                    platform.name()
                ),
                created,
            )?;
        }
    }
    Ok(())
}

// --- Theme scaffolding ---

fn scaffold_themes(
    w: &Writer,
    root: &Path,
    kind: ProjectKind,
    themes: &[String],
    created: &mut Vec<String>,
) -> Result<(), String> {
    if kind != ProjectKind::SlintApp && kind != ProjectKind::Super {
        return Ok(());
    }

    let themes_dir = root.join("ui").join("themes");
    w.ensure_dir(&root.join("ui"), created)?;
    w.ensure_dir(&themes_dir, created)?;

    for theme in themes {
        w.write_if_missing(
            &themes_dir,
            &format!("{theme}.slint"),
            &format!(
                "// Theme: {theme}\n\n\
                 export global {theme_pascal}Theme {{\n\
                     // Token stubs — fill in per design system\n\
                     in-out property <color> primary: #0078d4;\n\
                     in-out property <color> background: #ffffff;\n\
                     in-out property <length> spacing: 8px;\n\
                 }}\n",
                theme_pascal = to_pascal_case(theme)
            ),
            created,
        )?;
    }
    Ok(())
}

// --- MCP crate scaffolding ---

fn scaffold_mcp_crate(
    w: &Writer,
    root: &Path,
    kind: ProjectKind,
    name: &str,
    created: &mut Vec<String>,
) -> Result<(), String> {
    if kind != ProjectKind::Super {
        return Ok(());
    }

    let mcp_dir = root.join("crates").join("mcp");
    w.ensure_dir(&mcp_dir, created)?;
    let mcp_src = mcp_dir.join("src");
    w.ensure_dir(&mcp_src, created)?;

    w.write_if_missing(
        &mcp_dir,
        "Cargo.toml",
        &format!(
            "[package]\n\
             name = \"{name}-mcp\"\n\
             version.workspace = true\n\
             edition.workspace = true\n\n\
             [[bin]]\n\
             name = \"{name}-mcp\"\n\
             path = \"src/main.rs\"\n\n\
             [dependencies]\n\
             serde = {{ workspace = true }}\n\
             serde_json = {{ workspace = true }}\n"
        ),
        created,
    )?;

    w.write_if_missing(
        &mcp_src,
        "main.rs",
        &format!(
            "//! {name}-mcp — MCP server.\n\n\
             mod tools;\n\n\
             fn main() {{\n\
                 eprintln!(\"{name}-mcp ready\");\n\
             }}\n"
        ),
        created,
    )?;

    w.write_if_missing(
        &mcp_src,
        "tools.rs",
        "//! MCP tool definitions.\n",
        created,
    )?;

    Ok(())
}

// --- Extras scaffolding ---

fn scaffold_extras(
    w: &Writer,
    root: &Path,
    kind: ProjectKind,
    name: &str,
    extras: &[Extra],
    created: &mut Vec<String>,
) -> Result<(), String> {
    for extra in extras {
        match extra {
            Extra::Doc => {
                let doc = root.join("doc");
                w.ensure_dir(&doc, created)?;
            }
            Extra::Shared => {
                if kind == ProjectKind::Super {
                    let shared_dir = root.join("crates").join("shared");
                    w.ensure_dir(&shared_dir, created)?;
                    let shared_src = shared_dir.join("src");
                    w.ensure_dir(&shared_src, created)?;
                    w.write_if_missing(
                        &shared_dir,
                        "Cargo.toml",
                        &format!(
                            "[package]\n\
                             name = \"{name}-shared\"\n\
                             version.workspace = true\n\
                             edition.workspace = true\n\n\
                             [dependencies]\n"
                        ),
                        created,
                    )?;
                    w.write_if_missing(
                        &shared_src,
                        "lib.rs",
                        &format!("//! {name}-shared — shared utilities.\n"),
                        created,
                    )?;
                } else {
                    let shared = root.join("src").join("shared");
                    w.ensure_dir(&root.join("src"), created)?;
                    w.ensure_dir(&shared, created)?;
                    w.write_if_missing(&shared, "mod.rs", "// Shared utilities\n", created)?;
                }
            }
            Extra::Lib => {
                if kind == ProjectKind::Super {
                    let lib_dir = root.join("crates").join("lib");
                    scaffold_workspace_lib(w, &root.join("crates"), "lib", name, created)?;
                    let _ = lib_dir; // just to be explicit
                } else {
                    // Add lib.rs alongside main.rs
                    let src = root.join("src");
                    w.ensure_dir(&src, created)?;
                    w.write_if_missing(
                        &src,
                        "lib.rs",
                        &format!("//! {name} — library module.\n"),
                        created,
                    )?;
                }
            }
        }
    }
    Ok(())
}

// --- Per-kind scaffolding (shared between scaffold_project and scaffold_with_options) ---

fn scaffold_kind(
    w: &Writer,
    root: &Path,
    kind: ProjectKind,
    name: &str,
    created: &mut Vec<String>,
) -> Result<(), String> {
    match kind {
        ProjectKind::Tool => scaffold_tool(w, root, created),
        ProjectKind::CliApp => scaffold_cli(w, root, name, created),
        ProjectKind::Library => scaffold_library(w, root, name, created),
        ProjectKind::Website => scaffold_website(w, root, name, created),
        ProjectKind::SlintApp => scaffold_slint_app(w, root, name, created),
        ProjectKind::Super => scaffold_workspace(w, root, name, created),
    }
}

fn create_proj_files(
    w: &Writer,
    root: &Path,
    kind: ProjectKind,
    name: &str,
    created: &mut Vec<String>,
) -> Result<(), String> {
    let proj = root.join("proj");
    w.ensure_dir(&proj, created)?;

    let kind_str = kind.as_str();
    let is_gui = matches!(kind, ProjectKind::SlintApp | ProjectKind::Super);

    // Determine stack and structure based on kind
    let (stack, structure, method) = match kind {
        ProjectKind::Tool => (
            format!("- Language: Rust 2021\n- Type: CLI tool"),
            format!("- src: src/\n- proj: proj/"),
            format!("- Workflow: PROJECT → TODO → code → test\n- Testing: cargo test"),
        ),
        ProjectKind::CliApp => (
            format!("- Language: Rust 2021\n- Framework: clap 4\n- Type: CLI application"),
            format!("- src: src/\n- shared: src/shared/\n- doc: doc/\n- proj: proj/"),
            format!("- Workflow: PROJECT → PHASES → TODO → code → test → DONE\n- Testing: cargo test"),
        ),
        ProjectKind::Library => (
            format!("- Language: Rust 2021\n- Type: Library crate"),
            format!("- src: src/\n- proj: proj/"),
            format!("- Workflow: PROJECT → TODO → code → test\n- Testing: cargo test"),
        ),
        ProjectKind::Website => (
            format!("- Language: JavaScript/TypeScript\n- Type: Web project"),
            format!("- src: src/\n- styles: src/styles/\n- proj: proj/"),
            format!("- Workflow: PROJECT → TODO → code → test\n- Testing: browser"),
        ),
        ProjectKind::SlintApp => (
            format!("- Language: Rust 2021\n- UI: Slint 1.x\n- Type: GUI application"),
            format!("- src: src/ (topology: app/, core/, adapter/, gateway/, pal/, ui/)\n- slint: ui/\n- proj: proj/"),
            format!("- Workflow: PROJECT → PHASES → TODO → code → test → DONE\n- Testing: cargo test\n- UI: Slint previewer"),
        ),
        ProjectKind::Super => (
            format!("- Language: Rust 2021\n- UI: Slint 1.x\n- Type: Workspace (multi-crate)"),
            format!("- crates: crates/ (app, core, adapter, gateway, pal, ui)\n- slint: ui/\n- proj: proj/"),
            format!("- Workflow: PROJECT → PHASES → TODO → code → test → DONE\n- Testing: cargo test\n- UI: Slint previewer"),
        ),
    };

    // PROJECT — full format per project-files/project-file.md
    w.write_if_missing(
        &proj,
        "PROJECT",
        &format!(
            "# PROJECT: {name}\n\n\
             ## Goal\n\n\
             (Define the vision — what this project achieves. 2-5 sentences.)\n\n\
             ## Stack\n\n\
             {stack}\n\n\
             ## Structure\n\n\
             {structure}\n\n\
             ## Method\n\n\
             {method}\n\n\
             ## Patterns\n\n\
             (Recurring patterns discovered during development.)\n\n\
             ## Current\n\n\
             - phase: 1\n\
             - id: setup\n\
             - status: development\n\n\
             ## History\n\n\
             (None yet.)\n"
        ),
        created,
    )?;

    // PHASES — per project-files/phases-file.md
    w.write_if_missing(
        &proj,
        "PHASES",
        &format!(
            "# PHASES: {name}\n\n\
             ## Active\n\n\
             - phase: 1\n\
             \x20 id: setup\n\
             \x20 title: \"Project setup\"\n\
             \x20 milestone: \"Project scaffolded, builds, scans clean\"\n\
             \x20 delivers:\n\
             \x20   - Project structure\n\
             \x20   - Build configuration\n\
             \x20   - Scanner integration\n\
             \x20 status: active\n\n\
             ## Planned\n\n\
             - phase: 2\n\
             \x20 id: core\n\
             \x20 title: \"Core functionality\"\n\
             \x20 milestone: \"Primary feature functional\"\n\
             \x20 delivers:\n\
             \x20   - (define deliverables)\n\
             \x20 status: planned\n\n\
             # --- DONES ---\n"
        ),
        created,
    )?;

    // TODO — per project-files/todo-file.md
    w.write_if_missing(
        &proj,
        "TODO",
        &format!(
            "# TODO — {name}\n\n\
             ## Phase: 1 (setup)\n\n\
             - [ ] Verify build: `cargo build` / `npm run build`\n\
             - [ ] Verify scan: `rulestools scan .`\n\
             - [ ] Define Goal in proj/PROJECT\n\
             - [ ] Define phases in proj/PHASES\n"
        ),
        created,
    )?;

    // RULES — list active rules based on kind
    let rules_content = match kind {
        ProjectKind::SlintApp | ProjectKind::Super => format!(
            "# RULES — {name}\n\n\
             ## Active Rules\n\n\
             ### Global\n\
             - `get_rule(\"global/topology.md\")` — layer architecture\n\
             - `get_rule(\"global/file-limits.md\")` — file size limits\n\
             - `get_rule(\"global/nesting.md\")` — max nesting depth\n\n\
             ### Rust\n\
             - `get_context([\"rust\"])` — all Rust rules\n\n\
             ### UI\n\
             - `get_context([\"uiux\"])` — all UI/UX rules\n\
             - `get_rule(\"uiux/mother-child.md\")` — composition pattern\n\
             - `get_rule(\"uiux/tokens.md\")` — no literal values\n\
             - `get_rule(\"uiux/state-flow.md\")` — state-in, events-out\n\
             - `get_rule(\"uiux/components.md\")` — one file, one component\n\n\
             ### Slint\n\
             - `get_context([\"slint\"])` — Slint-specific rules\n"
        ),
        ProjectKind::Website => format!(
            "# RULES — {name}\n\n\
             ## Active Rules\n\n\
             ### Global\n\
             - `get_rule(\"global/file-limits.md\")` — file size limits\n\
             - `get_rule(\"global/nesting.md\")` — max nesting depth\n\n\
             ### Web\n\
             - `get_context([\"js\"])` — JavaScript rules\n\
             - `get_context([\"css\"])` — CSS rules\n"
        ),
        _ => format!(
            "# RULES — {name}\n\n\
             ## Active Rules\n\n\
             ### Global\n\
             - `get_rule(\"global/file-limits.md\")` — file size limits\n\
             - `get_rule(\"global/nesting.md\")` — max nesting depth\n\n\
             ### Rust\n\
             - `get_context([\"rust\"])` — all Rust rules\n"
        ),
    };
    w.write_if_missing(&proj, "RULES", &rules_content, created)?;

    // FIXES
    w.write_if_missing(
        &proj,
        "FIXES",
        &format!("# FIXES — {name}\n\n(no known issues)\n"),
        created,
    )?;

    // UIUX — required for ALL GUI projects
    if is_gui {
        let toolkit = if kind == ProjectKind::Super {
            "Slint 1.x (workspace)"
        } else {
            "Slint 1.x"
        };
        w.write_if_missing(
            &proj,
            "UIUX",
            &format!(
                "# UIUX: {name}\n\n\
                 ## Goal\n\n\
                 (Define the UI/UX vision — what the user experience should feel like.)\n\n\
                 ## Platform\n\n\
                 - Toolkit: {toolkit}\n\
                 - Entry: ui/main.slint\n\n\
                 ## UI Foundation Rules\n\n\
                 | Rule | What it enforces |\n\
                 |------|------------------|\n\
                 | uiux/tokens.md | Zero literal values — all values are named tokens |\n\
                 | uiux/components.md | One file per component, one responsibility |\n\
                 | uiux/state-flow.md | State-in from Adapter, events-out |\n\
                 | uiux/mother-child.md | Mother owns layout, children are self-contained |\n\
                 | uiux/theming.md | System light/dark — live switching |\n\
                 | uiux/keyboard.md | Standard shortcuts, keyboard navigation |\n\n\
                 ## UI Architecture\n\n\
                 - Entry point: src/main.rs → ui/main.slint\n\
                 - Topology: app → gateway → adapter → core, adapter → pal\n\
                 - UI layer: src/ui/ (Rust) + ui/ (Slint)\n\
                 - State: AdapterState in src/adapter/ — UI reads, never writes directly\n\n\
                 ## Component Conventions\n\n\
                 (Add conventions as patterns are discovered.)\n\n\
                 ## User Flows\n\n\
                 ### Primary Flow\n\n\
                 (Define the main user workflow.)\n\n\
                 ## Layout\n\n\
                 ### Main Window\n\n\
                 (Define the window layout.)\n"
            ),
            created,
        )?;
    }

    // rulestools.toml
    w.write_if_missing(
        &proj,
        "rulestools.toml",
        &format!("[project]\nkind = \"{kind_str}\"\n"),
        created,
    )?;

    Ok(())
}

fn scaffold_tool(w: &Writer, root: &Path, created: &mut Vec<String>) -> Result<(), String> {
    let src = root.join("src");
    w.ensure_dir(&src, created)?;
    w.write_if_missing(
        &src,
        "main.rs",
        "fn main() {\n    println!(\"Hello, world!\");\n}\n",
        created,
    )?;
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

    let shared = src.join("shared");
    w.ensure_dir(&shared, created)?;

    w.write_if_missing(
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
    w.write_if_missing(&shared, "mod.rs", "// Shared utilities\n", created)?;

    let doc = root.join("doc");
    w.ensure_dir(&doc, created)?;

    w.write_if_missing(
        root,
        "Cargo.toml",
        &cargo_toml_bin(name, &["clap = { version = \"4\", features = [\"derive\"] }"]),
        created,
    )?;

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
    w.write_if_missing(
        &src,
        "lib.rs",
        &format!(
            "//! {name} — library crate.\n\npub fn hello() -> &'static str {{\n    \"{name}\"\n}}\n"
        ),
        created,
    )?;
    w.write_if_missing(root, "Cargo.toml", &cargo_toml_lib(name, &[]), created)?;

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

    // Topology folders
    for folder in &["app", "core", "adapter", "gateway", "pal", "ui"] {
        let dir = src.join(folder);
        w.ensure_dir(&dir, created)?;
        w.write_if_missing(&dir, "mod.rs", &format!("//! {folder} layer.\n"), created)?;
    }

    w.write_if_missing(
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
    w.ensure_dir(&ui_dir, created)?;
    w.write_if_missing(&ui_dir, "main.slint", &slint_main_content(name), created)?;

    // build.rs
    w.write_if_missing(
        root,
        "build.rs",
        "fn main() {\n    slint_build::compile(\"ui/main.slint\").unwrap();\n}\n",
        created,
    )?;

    // Cargo.toml with slint deps
    w.write_if_missing(
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

    // Workspace crates
    let crates_dir = root.join("crates");
    w.ensure_dir(&crates_dir, created)?;

    // app crate (binary)
    scaffold_workspace_bin(w, &crates_dir, "app", name, created)?;

    // core, adapter, gateway, pal, ui (library crates)
    for crate_name in &["core", "adapter", "gateway", "pal", "ui"] {
        scaffold_workspace_lib(w, &crates_dir, crate_name, name, created)?;
    }

    // ui/ slint files
    let ui_dir = root.join("ui");
    w.ensure_dir(&ui_dir, created)?;
    w.write_if_missing(&ui_dir, "main.slint", &slint_main_content(name), created)?;

    // build.rs
    w.write_if_missing(
        root,
        "build.rs",
        "fn main() {\n    slint_build::compile(\"ui/main.slint\").unwrap();\n}\n",
        created,
    )?;

    Ok(())
}

fn scaffold_workspace_bin(
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

fn scaffold_workspace_lib(
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

// --- Helpers ---

fn slint_main_content(name: &str) -> String {
    format!(
        "import {{ Button, VerticalBox }} from \"std-widgets.slint\";\n\n\
         export component App inherits Window {{\n\
             title: \"{name}\";\n\
             width: 800px;\n\
             height: 600px;\n\n\
             VerticalBox {{\n\
                 Button {{ text: \"Hello\"; }}\n\
             }}\n\
         }}\n"
    )
}

fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c == '-' || c == '_' || c == ' ')
        .filter(|w| !w.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    upper + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect()
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

// --- Tests ---

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

    // --- Existing tests (backward compat) ---

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
        let content = std::fs::read_to_string(proj.join("PROJECT")).unwrap();
        assert_eq!(content, "custom content");
        assert!(!result.contains("PROJECT"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // --- Preview mode tests ---

    #[test]
    fn test_preview_creates_nothing() {
        let dir = temp_dir("preview");
        let opts = ScaffoldOptions {
            name: "test-preview".into(),
            kind: ProjectKind::CliApp,
            platforms: vec![],
            themes: vec![],
            mcp: false,
            extras: vec![],
            preview: true,
        };
        let result = scaffold_with_options(&dir, &opts).unwrap();
        assert!(!result.created.is_empty(), "Should report what would be created");
        // Nothing should actually be written
        assert!(!dir.join("proj/PROJECT").exists());
        assert!(!dir.join("src/main.rs").exists());
        assert!(!dir.join("Cargo.toml").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    // --- Platform tests ---

    #[test]
    fn test_new_with_platforms() {
        let dir = temp_dir("platforms");
        let opts = ScaffoldOptions {
            name: "test-plat".into(),
            kind: ProjectKind::SlintApp,
            platforms: vec![Platform::Mobile, Platform::Desktop],
            themes: vec![],
            mcp: false,
            extras: vec![],
            preview: false,
        };
        let result = scaffold_with_options(&dir, &opts).unwrap();
        assert!(dir.join("src/pal/mobile.rs").exists());
        assert!(dir.join("src/pal/desktop.rs").exists());
        assert!(result.created.iter().any(|p| p.contains("mobile.rs")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // --- MCP crate test ---

    #[test]
    fn test_new_with_mcp() {
        let dir = temp_dir("mcp");
        let opts = ScaffoldOptions {
            name: "test-mcp".into(),
            kind: ProjectKind::Super,
            platforms: vec![],
            themes: vec![],
            mcp: true,
            extras: vec![],
            preview: false,
        };
        let result = scaffold_with_options(&dir, &opts).unwrap();
        assert!(dir.join("crates/mcp/Cargo.toml").exists());
        assert!(dir.join("crates/mcp/src/main.rs").exists());
        assert!(dir.join("crates/mcp/src/tools.rs").exists());
        assert!(result.created.iter().any(|p| p.contains("mcp")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // --- Theme test ---

    #[test]
    fn test_new_with_themes() {
        let dir = temp_dir("themes");
        let opts = ScaffoldOptions {
            name: "test-theme".into(),
            kind: ProjectKind::SlintApp,
            platforms: vec![],
            themes: vec!["win3ui-fluent".into(), "macos".into()],
            mcp: false,
            extras: vec![],
            preview: false,
        };
        let _ = scaffold_with_options(&dir, &opts).unwrap();
        assert!(dir.join("ui/themes/win3ui-fluent.slint").exists());
        assert!(dir.join("ui/themes/macos.slint").exists());
        let content = std::fs::read_to_string(dir.join("ui/themes/macos.slint")).unwrap();
        assert!(content.contains("MacosTheme"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // --- Update tests ---

    #[test]
    fn test_update_add_platform() {
        let dir = temp_dir("update-plat");
        // First create a SlintApp
        scaffold_project(&dir, ProjectKind::SlintApp, "test-upd").unwrap();
        assert!(dir.join("src/pal/mod.rs").exists());

        // Now update: add mobile platform
        let opts = UpdateOptions {
            platforms: vec![Platform::Mobile],
            themes: vec![],
            crate_name: None,
            folders: vec![],
            preview: false,
        };
        let result = update_project(&dir, &opts).unwrap();
        assert!(dir.join("src/pal/mobile.rs").exists());
        assert!(result.created.iter().any(|p| p.contains("mobile.rs")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_update_preserves_kind() {
        let dir = temp_dir("update-kind");
        scaffold_project(&dir, ProjectKind::SlintApp, "test-kind").unwrap();

        let opts = UpdateOptions {
            platforms: vec![Platform::Mobile],
            themes: vec![],
            crate_name: None,
            folders: vec![],
            preview: false,
        };
        update_project(&dir, &opts).unwrap();

        // Kind should still be slint-app
        let toml = std::fs::read_to_string(dir.join("proj/rulestools.toml")).unwrap();
        assert!(toml.contains("slint-app"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // --- Upgrade tests ---

    #[test]
    fn test_upgrade_tool_to_cli() {
        let dir = temp_dir("upg-tool-cli");
        scaffold_project(&dir, ProjectKind::Tool, "test-upg").unwrap();

        let result = upgrade_project(&dir, ProjectKind::CliApp, false).unwrap();
        assert_eq!(result.from_kind, ProjectKind::Tool);
        assert_eq!(result.to_kind, ProjectKind::CliApp);
        assert!(dir.join("src/shared/mod.rs").exists());
        assert!(dir.join("doc").exists());
        // rulestools.toml should be updated
        let toml = std::fs::read_to_string(dir.join("proj/rulestools.toml")).unwrap();
        assert!(toml.contains("cli"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_upgrade_cli_to_slint() {
        let dir = temp_dir("upg-cli-slint");
        scaffold_project(&dir, ProjectKind::CliApp, "test-upg").unwrap();

        let result = upgrade_project(&dir, ProjectKind::SlintApp, false).unwrap();
        assert_eq!(result.from_kind, ProjectKind::CliApp);
        assert_eq!(result.to_kind, ProjectKind::SlintApp);
        assert!(dir.join("ui/main.slint").exists());
        assert!(dir.join("src/app/mod.rs").exists());
        assert!(!result.move_guidance.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_upgrade_downgrade_rejected() {
        let dir = temp_dir("upg-down");
        scaffold_project(&dir, ProjectKind::SlintApp, "test-down").unwrap();

        let result = upgrade_project(&dir, ProjectKind::Tool, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot downgrade"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_upgrade_same_kind_rejected() {
        let dir = temp_dir("upg-same");
        scaffold_project(&dir, ProjectKind::CliApp, "test-same").unwrap();

        let result = upgrade_project(&dir, ProjectKind::CliApp, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Already"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_upgrade_move_guidance() {
        let dir = temp_dir("upg-guidance");
        scaffold_project(&dir, ProjectKind::CliApp, "test-guid").unwrap();

        let result = upgrade_project(&dir, ProjectKind::Super, false).unwrap();
        assert!(!result.move_guidance.is_empty());
        // Should have guidance about moving main.rs to crates/app/
        assert!(result
            .move_guidance
            .iter()
            .any(|g| g.to.contains("crates/app")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_pascal_case() {
        assert_eq!(to_pascal_case("win3ui-fluent"), "Win3uiFluent");
        assert_eq!(to_pascal_case("macos"), "Macos");
        assert_eq!(to_pascal_case("my-custom-theme"), "MyCustomTheme");
    }
}
