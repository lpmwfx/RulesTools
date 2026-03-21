mod templates;
mod types;
mod proj_files;
mod kinds;
mod features;
mod upgrade;
mod update;

pub use types::*;
pub use upgrade::{upgrade_project, render_tree};
pub use update::update_project;

use std::path::Path;

use rulestools_scanner::project::ProjectKind;

use types::Writer;
use templates::gitignore_content;
use proj_files::create_proj_files;
use kinds::{scaffold_kind, scaffold_workspace_ui};
use features::{scaffold_platforms, scaffold_themes, scaffold_mcp_crate, scaffold_extras};

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
        // Workspace + platforms: add Slint UI layer (crates/ui + ui/main.slint)
        if opts.kind == ProjectKind::Super {
            scaffold_workspace_ui(&w, root, &opts.name, &mut created)?;
        }
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

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use templates::{CLI_TOPOLOGY, GUI_TOPOLOGY};

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
        assert_eq!(templates::to_pascal_case("win3ui-fluent"), "Win3uiFluent");
        assert_eq!(templates::to_pascal_case("macos"), "Macos");
        assert_eq!(templates::to_pascal_case("my-custom-theme"), "MyCustomTheme");
    }

    // --- Topology validation tests (TODO id:14-18) ---

    #[test]
    fn test_workspace_creates_crates_topology_no_slint() {
        let dir = temp_dir("val-ws-no-slint");
        let opts = ScaffoldOptions {
            name: "val-ws".into(),
            kind: ProjectKind::Super,
            platforms: vec![],
            themes: vec![],
            mcp: false,
            extras: vec![],
            preview: false,
        };
        scaffold_with_options(&dir, &opts).unwrap();

        // All CLI topology crates exist with lib.rs
        for layer in CLI_TOPOLOGY {
            assert!(
                dir.join(format!("crates/{layer}/src/lib.rs")).exists(),
                "crates/{layer}/src/lib.rs must exist"
            );
            assert!(
                dir.join(format!("crates/{layer}/Cargo.toml")).exists(),
                "crates/{layer}/Cargo.toml must exist"
            );
        }
        // app crate has main.rs (binary)
        assert!(dir.join("crates/app/src/main.rs").exists());

        // NO slint files
        assert!(!dir.join("ui/main.slint").exists(), "bare workspace must NOT have ui/main.slint");
        assert!(!dir.join("crates/ui").exists(), "bare workspace must NOT have crates/ui/");
        assert!(!dir.join("proj/UIUX").exists(), "bare workspace must NOT have proj/UIUX");

        // build.rs uses scanner, not slint_build
        let build_rs = std::fs::read_to_string(dir.join("build.rs")).unwrap();
        assert!(build_rs.contains("scan_project"), "build.rs must call scan_project");
        assert!(!build_rs.contains("slint_build"), "build.rs must NOT contain slint_build");

        // Workspace Cargo.toml has workspace members
        let cargo = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        assert!(cargo.contains("[workspace]"));
        assert!(
            cargo.contains("crates/*") || cargo.contains("crates/app"),
            "Cargo.toml must reference crates (glob or explicit)"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_workspace_with_platforms_creates_slint() {
        let dir = temp_dir("val-ws-plat");
        let opts = ScaffoldOptions {
            name: "val-ws-plat".into(),
            kind: ProjectKind::Super,
            platforms: vec![Platform::Desktop],
            themes: vec![],
            mcp: false,
            extras: vec![],
            preview: false,
        };
        scaffold_with_options(&dir, &opts).unwrap();

        // CLI topology crates still exist
        for layer in CLI_TOPOLOGY {
            assert!(
                dir.join(format!("crates/{layer}/src/lib.rs")).exists(),
                "crates/{layer}/src/lib.rs must exist"
            );
        }
        assert!(dir.join("crates/app/src/main.rs").exists());

        // Slint layers present
        assert!(dir.join("crates/ui/src/lib.rs").exists(), "workspace+platforms must have crates/ui/");
        assert!(dir.join("ui/main.slint").exists(), "workspace+platforms must have ui/main.slint");
        assert!(dir.join("proj/UIUX").exists(), "workspace+platforms must have proj/UIUX");

        // build.rs includes slint_build
        let build_rs = std::fs::read_to_string(dir.join("build.rs")).unwrap();
        assert!(build_rs.contains("slint_build"), "build.rs must contain slint_build with platforms");

        // Platform PAL stub
        assert!(dir.join("crates/pal/src/desktop.rs").exists(), "must have desktop.rs PAL stub");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_cli_creates_src_topology_with_all_folders() {
        let dir = temp_dir("val-cli-topo");
        let result = scaffold_project(&dir, ProjectKind::CliApp, "val-cli");
        assert!(result.is_ok());

        // All CLI topology folders in src/ with mod.rs
        for layer in CLI_TOPOLOGY {
            assert!(
                dir.join(format!("src/{layer}/mod.rs")).exists(),
                "src/{layer}/mod.rs must exist for CLI"
            );
        }

        // main.rs has mod declarations
        let main = std::fs::read_to_string(dir.join("src/main.rs")).unwrap();
        for layer in CLI_TOPOLOGY {
            assert!(main.contains(&format!("mod {layer};")), "main.rs must declare mod {layer}");
        }

        // No UIUX
        assert!(!dir.join("proj/UIUX").exists(), "CLI must NOT have proj/UIUX");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_tool_creates_src_topology_with_all_folders() {
        let dir = temp_dir("val-tool-topo");
        let result = scaffold_project(&dir, ProjectKind::Tool, "val-tool");
        assert!(result.is_ok());

        // All CLI topology folders in src/ with mod.rs
        for layer in CLI_TOPOLOGY {
            assert!(
                dir.join(format!("src/{layer}/mod.rs")).exists(),
                "src/{layer}/mod.rs must exist for Tool"
            );
        }

        // main.rs has mod declarations
        let main = std::fs::read_to_string(dir.join("src/main.rs")).unwrap();
        for layer in CLI_TOPOLOGY {
            assert!(main.contains(&format!("mod {layer};")), "main.rs must declare mod {layer}");
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_slint_app_creates_full_gui_topology() {
        let dir = temp_dir("val-slint-topo");
        let result = scaffold_project(&dir, ProjectKind::SlintApp, "val-slint");
        assert!(result.is_ok());

        // All GUI topology folders in src/ with mod.rs
        for layer in GUI_TOPOLOGY {
            assert!(
                dir.join(format!("src/{layer}/mod.rs")).exists(),
                "src/{layer}/mod.rs must exist for SlintApp"
            );
        }

        // main.rs has mod declarations for all layers
        let main = std::fs::read_to_string(dir.join("src/main.rs")).unwrap();
        for layer in GUI_TOPOLOGY {
            assert!(main.contains(&format!("mod {layer};")), "main.rs must declare mod {layer}");
        }

        // Slint files
        assert!(dir.join("ui/main.slint").exists(), "SlintApp must have ui/main.slint");
        assert!(dir.join("proj/UIUX").exists(), "SlintApp must have proj/UIUX");

        // build.rs has slint_build
        let build_rs = std::fs::read_to_string(dir.join("build.rs")).unwrap();
        assert!(build_rs.contains("slint_build"), "SlintApp build.rs must contain slint_build");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
