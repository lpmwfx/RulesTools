use std::path::Path;

use rulestools_scanner::project::ProjectKind;

use super::types::Writer;
use super::templates::*;

pub(super) fn create_proj_files(
    w: &Writer,
    root: &Path,
    kind: ProjectKind,
    name: &str,
    created: &mut Vec<String>,
) -> Result<(), String> {
    let proj = root.join("proj");
    w.ensure_dir(&proj, created)?;

    let kind_str = kind.as_str();
    let is_gui = kind == ProjectKind::SlintApp;

    // Determine stack and structure based on kind
    let (stack, structure, method) = match kind {
        ProjectKind::Tool => (
            format!("- Language: Rust 2021\n- Type: CLI tool"),
            format!("- src: src/ (topology: core/, adapter/, gateway/, pal/, shared/)\n- proj: proj/"),
            format!("- Workflow: PROJECT → TODO → code → test\n- Testing: cargo test"),
        ),
        ProjectKind::CliApp => (
            format!("- Language: Rust 2021\n- Framework: clap 4\n- Type: CLI application"),
            format!("- src: src/ (topology: core/, adapter/, gateway/, pal/, shared/)\n- doc: doc/\n- proj: proj/"),
            format!("- Workflow: PROJECT → PHASES → TODO → code → test → DONE\n- Testing: cargo test"),
        ),
        ProjectKind::Library => (
            format!("- Language: Rust 2021\n- Type: Library crate"),
            format!("- src: src/ (topology: core/, shared/)\n- proj: proj/"),
            format!("- Workflow: PROJECT → TODO → code → test\n- Testing: cargo test"),
        ),
        ProjectKind::Website => (
            format!("- Language: JavaScript/TypeScript\n- Type: Web project"),
            format!("- src: src/\n- styles: src/styles/\n- proj: proj/"),
            format!("- Workflow: PROJECT → TODO → code → test\n- Testing: browser"),
        ),
        ProjectKind::SlintApp => (
            format!("- Language: Rust 2021\n- UI: Slint 1.x\n- Type: GUI application"),
            format!("- src: src/ (topology: app/, core/, adapter/, gateway/, pal/, shared/, ui/)\n- slint: ui/\n- proj: proj/"),
            format!("- Workflow: PROJECT → PHASES → TODO → code → test → DONE\n- Testing: cargo test\n- UI: Slint previewer"),
        ),
        ProjectKind::Super => (
            format!("- Language: Rust 2021\n- Type: Workspace (multi-crate)"),
            format!("- crates: crates/ (topology: app/, core/, adapter/, gateway/, pal/, shared/)\n- proj: proj/"),
            format!("- Workflow: PROJECT → PHASES → TODO → code → test → DONE\n- Testing: cargo test"),
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
        ProjectKind::SlintApp => format!(
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

    // .claude/settings.json — PostToolUse hook for AI-session scanning
    let claude_dir = root.join(".claude");
    w.ensure_dir(&claude_dir, created)?;
    w.write_if_missing(&claude_dir, "settings.json", CLAUDE_SETTINGS, created)?;

    // Pre-commit hook
    let hooks_dir = root.join(".git").join("hooks");
    if root.join(".git").exists() {
        w.ensure_dir(&hooks_dir, created)?;
        let hook_path = hooks_dir.join("pre-commit");
        if !hook_path.exists() {
            if !w.dry_run {
                std::fs::write(&hook_path, PRE_COMMIT_HOOK)
                    .map_err(|e| format!("Cannot write pre-commit hook: {e}"))?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o755));
                }
            }
            created.push(format!("{}", hook_path.display()));
        }
    }

    Ok(())
}
