use std::path::Path;

use rulestools_scanner::project::ProjectKind;

use super::types::{Extra, Platform, Writer};
use super::templates::*;
use super::kinds::scaffold_workspace_lib;

pub(super) fn scaffold_platforms(
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

pub(super) fn scaffold_themes(
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

pub(super) fn scaffold_mcp_crate(
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

pub(super) fn scaffold_extras(
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
