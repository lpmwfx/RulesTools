use std::path::PathBuf;

use crate::scaffold;

pub fn cmd_init(path: &PathBuf, kind_str: &str, name: Option<&str>) {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());

    let kind = match rulestools_scanner::project::ProjectKind::from_str(kind_str) {
        Some(k) => k,
        None => {
            eprintln!(
                "Unknown kind: {kind_str}\nValid kinds: tool, cli, library, website, slint-app, workspace"
            );
            std::process::exit(1);
        }
    };

    let project_name = name
        .map(String::from)
        .unwrap_or_else(|| {
            root.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project")
                .to_string()
        });

    match scaffold::scaffold_project(&root, kind, &project_name) {
        Ok(summary) => {
            println!("{summary}");
            println!();
            cmd_detect(path);
        }
        Err(e) => {
            eprintln!("Scaffold failed: {e}");
            std::process::exit(1);
        }
    }
}

pub fn init_internal(path: &std::path::Path, kind_str: &str, name: Option<&str>) -> Result<String, String> {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    let kind = rulestools_scanner::project::ProjectKind::from_str(kind_str)
        .ok_or_else(|| format!("Unknown kind: {kind_str}\nValid kinds: tool, cli, library, website, slint-app, workspace"))?;

    let project_name = name
        .map(String::from)
        .unwrap_or_else(|| {
            root.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project")
                .to_string()
        });

    let summary = scaffold::scaffold_project(&root, kind, &project_name)?;
    let setup = setup_internal(&root).unwrap_or_default();
    let detect = detect_internal(path);
    Ok(format!("{summary}\n\n{setup}\n\n{detect}"))
}

pub fn cmd_new(
    path: &PathBuf,
    kind_str: &str,
    name: Option<&str>,
    platforms_str: &str,
    themes_str: &str,
    mcp: bool,
    extras_str: &str,
    preview: bool,
    format: &str,
) {
    match new_internal(path, kind_str, name, platforms_str, themes_str, mcp, extras_str, preview, format) {
        Ok(output) => print!("{output}"),
        Err(e) => {
            eprintln!("Scaffold failed: {e}");
            std::process::exit(1);
        }
    }
}

pub fn new_internal(
    path: &std::path::Path,
    kind_str: &str,
    name: Option<&str>,
    platforms_str: &str,
    themes_str: &str,
    mcp: bool,
    extras_str: &str,
    preview: bool,
    format: &str,
) -> Result<String, String> {
    let root = if path.exists() {
        std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    } else {
        std::fs::create_dir_all(path).map_err(|e| format!("Cannot create directory: {e}"))?;
        std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    };

    let kind = rulestools_scanner::project::ProjectKind::from_str(kind_str)
        .ok_or_else(|| format!("Unknown kind: {kind_str}\nValid kinds: tool, cli, library, website, slint-app, workspace"))?;

    let project_name = name
        .map(String::from)
        .unwrap_or_else(|| {
            root.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project")
                .to_string()
        });

    let platforms: Vec<scaffold::Platform> = platforms_str
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .filter_map(|s| scaffold::Platform::from_str(s))
        .collect();

    let themes: Vec<String> = themes_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let extras: Vec<scaffold::Extra> = extras_str
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .filter_map(|s| scaffold::Extra::from_str(s))
        .collect();

    let opts = scaffold::ScaffoldOptions {
        name: project_name.clone(),
        kind,
        platforms,
        themes,
        mcp,
        extras,
        preview,
    };

    let result = scaffold::scaffold_with_options(&root, &opts)?;

    // git init so pre-commit hook can be installed
    if !preview && !root.join(".git").exists() {
        let _ = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&root)
            .output();
    }
    let setup_result = if !preview {
        setup_internal(&root).unwrap_or_default()
    } else {
        String::new()
    };

    if format == "json" {
        let tree = scaffold::render_tree(&project_name, &result.created);
        let json = serde_json::json!({
            "name": project_name,
            "kind": kind_str,
            "preview": preview,
            "created": result.created,
            "skipped": result.skipped,
            "tree": tree,
        });
        Ok(serde_json::to_string_pretty(&json).unwrap_or_default())
    } else {
        let mut out = format!("{}\n", result.summary);
        for p in &result.created {
            out.push_str(&format!("  {p}\n"));
        }
        if !result.skipped.is_empty() {
            out.push_str("\nSkipped:\n");
            for s in &result.skipped {
                out.push_str(&format!("  {s}\n"));
            }
        }
        if !setup_result.is_empty() {
            out.push_str(&format!("\n{setup_result}\n"));
        }
        Ok(out)
    }
}

pub fn cmd_update(
    path: &PathBuf,
    platforms_str: &str,
    themes_str: &str,
    crate_name: Option<&str>,
    folders_str: &str,
    preview: bool,
    format: &str,
) {
    match update_internal(path, platforms_str, themes_str, crate_name, folders_str, preview, format) {
        Ok(output) => print!("{output}"),
        Err(e) => {
            eprintln!("Update failed: {e}");
            std::process::exit(1);
        }
    }
}

pub fn update_internal(
    path: &std::path::Path,
    platforms_str: &str,
    themes_str: &str,
    crate_name: Option<&str>,
    folders_str: &str,
    preview: bool,
    format: &str,
) -> Result<String, String> {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    let platforms: Vec<scaffold::Platform> = platforms_str
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .filter_map(|s| scaffold::Platform::from_str(s))
        .collect();

    let themes: Vec<String> = themes_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let folders: Vec<scaffold::Extra> = folders_str
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .filter_map(|s| scaffold::Extra::from_str(s))
        .collect();

    let opts = scaffold::UpdateOptions {
        platforms,
        themes,
        crate_name: crate_name.map(String::from),
        folders,
        preview,
    };

    let result = scaffold::update_project(&root, &opts)?;

    if !preview {
        let _ = setup_internal(&root);
    }

    if format == "json" {
        let json = serde_json::json!({
            "preview": preview,
            "created": result.created,
            "skipped": result.skipped,
        });
        Ok(serde_json::to_string_pretty(&json).unwrap_or_default())
    } else {
        let mut out = format!("{}\n", result.summary);
        for p in &result.created {
            out.push_str(&format!("  {p}\n"));
        }
        if !result.skipped.is_empty() {
            out.push_str("\nSkipped:\n");
            for s in &result.skipped {
                out.push_str(&format!("  {s}\n"));
            }
        }
        Ok(out)
    }
}

pub fn cmd_upgrade(path: &PathBuf, to_str: &str, preview: bool, format: &str) {
    match upgrade_internal(path, to_str, preview, format) {
        Ok(output) => print!("{output}"),
        Err(e) => {
            eprintln!("Upgrade failed: {e}");
            std::process::exit(1);
        }
    }
}

pub fn upgrade_internal(
    path: &std::path::Path,
    to_str: &str,
    preview: bool,
    format: &str,
) -> Result<String, String> {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    let to_kind = rulestools_scanner::project::ProjectKind::from_str(to_str)
        .ok_or_else(|| format!("Unknown kind: {to_str}\nValid kinds: tool, cli, library, website, slint-app, workspace"))?;

    let result = scaffold::upgrade_project(&root, to_kind, preview)?;

    if !preview {
        let _ = setup_internal(&root);
    }

    if format == "json" {
        let guidance: Vec<serde_json::Value> = result
            .move_guidance
            .iter()
            .map(|g| {
                serde_json::json!({
                    "from": g.from,
                    "to": g.to,
                    "reason": g.reason,
                })
            })
            .collect();
        let json = serde_json::json!({
            "from": result.from_kind.as_str(),
            "to": result.to_kind.as_str(),
            "preview": preview,
            "created": result.created,
            "move_guidance": guidance,
            "manual_steps": result.manual_steps,
        });
        Ok(serde_json::to_string_pretty(&json).unwrap_or_default())
    } else {
        let label = if preview { "Preview" } else { "Upgraded" };
        let mut out = format!(
            "{}: {:?} -> {:?}\n",
            label, result.from_kind, result.to_kind
        );

        if !result.created.is_empty() {
            out.push_str("\nCreated:\n");
            for p in &result.created {
                out.push_str(&format!("  {p}\n"));
            }
        }

        if !result.move_guidance.is_empty() {
            out.push_str("\nMove guidance:\n");
            for g in &result.move_guidance {
                out.push_str(&format!("  {} -> {}\n", g.from, g.to));
                out.push_str(&format!("    {}\n", g.reason));
            }
        }

        if !result.manual_steps.is_empty() {
            out.push_str("\nManual steps:\n");
            for step in &result.manual_steps {
                out.push_str(&format!("  - {step}\n"));
            }
        }

        Ok(out)
    }
}

pub fn cmd_detect(path: &PathBuf) {
    println!("{}", detect_internal(path));
}

pub fn detect_internal(path: &std::path::Path) -> String {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let identity = rulestools_scanner::project::ProjectIdentity::detect(&root);
    let cfg = rulestools_scanner::config::Config::load(&root);

    let mut out = String::new();
    out.push_str(&format!("Project:  {}\n", root.display()));
    out.push_str(&format!("Kind:     {:?}\n", identity.kind));
    out.push_str(&format!("Layout:   {:?}\n", identity.layout));
    out.push_str(&format!("Languages: {:?}\n", cfg.languages));
    out.push_str(&format!("Deny:     {}\n", cfg.deny));
    out.push_str("\nSkipped check categories:\n");
    for cat in identity.kind.skipped_categories() {
        out.push_str(&format!("  - {cat}\n"));
    }
    out
}

pub fn cmd_list(path: &PathBuf) {
    print!("{}", list_internal(path));
}

pub fn list_internal(path: &std::path::Path) -> String {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let cfg = rulestools_scanner::config::Config::load(&root);
    let identity = rulestools_scanner::project::ProjectIdentity::detect(&root);
    let registry = rulestools_scanner::checks::registry();

    let mut out = format!("rulestools: {:?} / {:?}\n", identity.kind, identity.layout);

    if registry.is_empty() {
        out.push_str("0 checks registered (skeleton — checks added in next phase)\n");
        return out;
    }

    out.push_str(&format!("{:<40} {:<20} {}\n", "CHECK", "LANGUAGES", "STATUS"));
    out.push_str(&format!("{}\n", "-".repeat(70)));

    for entry in &registry {
        let langs: Vec<&str> = entry.languages.iter().map(|l| l.name()).collect();
        let lang_str = if langs.is_empty() {
            "all".to_string()
        } else {
            langs.join(", ")
        };
        let active = cfg.is_enabled(&entry.id) && identity.kind.allows_check(&entry.id);
        let status = if active { "enabled" } else { "disabled" };
        out.push_str(&format!("{:<40} {:<20} {}\n", entry.id, lang_str, status));
    }
    out
}

pub fn setup_internal(path: &std::path::Path) -> Result<String, String> {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let identity = rulestools_scanner::project::ProjectIdentity::detect(&root);
    let mut actions = Vec::new();

    // 1. proj/ dir
    let proj_dir = root.join("proj");
    if !proj_dir.exists() {
        std::fs::create_dir_all(&proj_dir).map_err(|e| format!("Cannot create proj/: {e}"))?;
        actions.push("Created proj/".into());
    }

    // 2. proj/rulestools.toml
    let toml_path = proj_dir.join("rulestools.toml");
    if !toml_path.exists() {
        let kind_lower = identity.kind.as_str().to_lowercase();
        let content = format!("[project]\nkind = \"{kind_lower}\"\n");
        std::fs::write(&toml_path, content).map_err(|e| format!("Cannot write rulestools.toml: {e}"))?;
        actions.push("Created proj/rulestools.toml".into());
    }

    // 3. Cargo.toml build-deps
    if let Some(msg) = ensure_build_deps(&root, identity.kind)? {
        actions.push(msg);
    }

    // 4. build.rs
    if let Some(msg) = ensure_build_rs(&root, identity.kind)? {
        actions.push(msg);
    }

    // 5. .git/hooks/pre-commit
    let git_hooks = root.join(".git").join("hooks");
    if git_hooks.exists() {
        let hook_path = git_hooks.join("pre-commit");
        let hook_content = "#!/bin/sh\nrulestools check \"$(git rev-parse --show-toplevel)\"\n";
        std::fs::write(&hook_path, hook_content).map_err(|e| format!("Cannot write pre-commit hook: {e}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o755));
        }
        actions.push("Installed .git/hooks/pre-commit".into());
    }

    // 6. .claude/settings.json PostToolUse hook
    if let Some(msg) = ensure_claude_settings(&root)? {
        actions.push(msg);
    }

    if actions.is_empty() {
        Ok("Setup already complete — nothing to do".into())
    } else {
        Ok(format!("Setup complete:\n- {}", actions.join("\n- ")))
    }
}

/// Ensure Cargo.toml has rulestools-scanner in [build-dependencies].
fn ensure_build_deps(root: &std::path::Path, kind: rulestools_scanner::project::ProjectKind) -> Result<Option<String>, String> {
    if kind == rulestools_scanner::project::ProjectKind::Website {
        return Ok(None);
    }

    let cargo_path = root.join("Cargo.toml");
    if !cargo_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&cargo_path)
        .map_err(|e| format!("Cannot read Cargo.toml: {e}"))?;

    if content.contains("rulestools-scanner") {
        return Ok(None);
    }

    // Don't modify workspace root Cargo.toml (it has [workspace] section)
    if content.contains("[workspace]") {
        return Ok(None);
    }

    let scanner_dep = "rulestools-scanner = { git = \"https://github.com/lpmwfx/RulesTools\" }\n";

    let new_content = if content.contains("[build-dependencies]") {
        content.replacen("[build-dependencies]\n", &format!("[build-dependencies]\n{scanner_dep}"), 1)
    } else {
        format!("{content}\n[build-dependencies]\n{scanner_dep}")
    };

    std::fs::write(&cargo_path, new_content)
        .map_err(|e| format!("Cannot write Cargo.toml: {e}"))?;

    Ok(Some("Added rulestools-scanner to Cargo.toml [build-dependencies]".into()))
}

/// Ensure build.rs exists and calls rulestools_scanner::scan_project().
fn ensure_build_rs(root: &std::path::Path, kind: rulestools_scanner::project::ProjectKind) -> Result<Option<String>, String> {
    if kind == rulestools_scanner::project::ProjectKind::Website {
        return Ok(None);
    }

    let build_rs = root.join("build.rs");
    let is_slint = kind == rulestools_scanner::project::ProjectKind::SlintApp
        || kind == rulestools_scanner::project::ProjectKind::Super;

    if !build_rs.exists() {
        let content = if is_slint {
            "fn main() {\n    rulestools_scanner::scan_project();\n    slint_build::compile(\"ui/main.slint\").expect(\"Slint build failed\");\n}\n"
        } else {
            "fn main() {\n    rulestools_scanner::scan_project();\n}\n"
        };
        std::fs::write(&build_rs, content)
            .map_err(|e| format!("Cannot write build.rs: {e}"))?;
        return Ok(Some("Created build.rs with scan_project()".into()));
    }

    let content = std::fs::read_to_string(&build_rs)
        .map_err(|e| format!("Cannot read build.rs: {e}"))?;

    if content.contains("rulestools_scanner") {
        return Ok(None);
    }

    // Insert scan_project() at start of fn main()
    let new_content = content.replacen(
        "fn main() {\n",
        "fn main() {\n    rulestools_scanner::scan_project();\n",
        1,
    );

    if new_content == content {
        // Couldn't find fn main() { pattern — skip
        return Ok(None);
    }

    std::fs::write(&build_rs, new_content)
        .map_err(|e| format!("Cannot write build.rs: {e}"))?;

    Ok(Some("Added rulestools_scanner::scan_project() to build.rs".into()))
}

/// Ensure .claude/settings.json has PostToolUse hook for rulestools.
fn ensure_claude_settings(root: &std::path::Path) -> Result<Option<String>, String> {
    let claude_dir = root.join(".claude");
    let settings_path = claude_dir.join("settings.json");

    let hook_entry = serde_json::json!({
        "matcher": "Edit|Write|MultiEdit",
        "hooks": [{ "type": "command", "command": "rulestools hook" }]
    });

    if !claude_dir.exists() {
        std::fs::create_dir_all(&claude_dir)
            .map_err(|e| format!("Cannot create .claude/: {e}"))?;
    }

    if !settings_path.exists() {
        let settings = serde_json::json!({
            "hooks": {
                "PostToolUse": [hook_entry]
            }
        });
        let content = serde_json::to_string_pretty(&settings).unwrap_or_default();
        std::fs::write(&settings_path, format!("{content}\n"))
            .map_err(|e| format!("Cannot write .claude/settings.json: {e}"))?;
        return Ok(Some("Created .claude/settings.json with PostToolUse hook".into()));
    }

    // Parse existing settings
    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Cannot read .claude/settings.json: {e}"))?;

    let mut settings: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Cannot parse .claude/settings.json: {e}"))?;

    // Check if hook already exists
    if let Some(hooks) = settings.get("hooks").and_then(|h| h.get("PostToolUse")) {
        if let Some(arr) = hooks.as_array() {
            for entry in arr {
                if let Some(inner_hooks) = entry.get("hooks").and_then(|h| h.as_array()) {
                    for h in inner_hooks {
                        if h.get("command").and_then(|c| c.as_str()) == Some("rulestools hook") {
                            return Ok(None); // Already installed
                        }
                    }
                }
            }
        }
    }

    // Merge hook into existing settings
    let hooks = settings
        .as_object_mut()
        .ok_or("settings.json is not an object")?
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));

    let post_tool = hooks
        .as_object_mut()
        .ok_or("hooks is not an object")?
        .entry("PostToolUse")
        .or_insert_with(|| serde_json::json!([]));

    if let Some(arr) = post_tool.as_array_mut() {
        arr.push(hook_entry);
    }

    let new_content = serde_json::to_string_pretty(&settings).unwrap_or_default();
    std::fs::write(&settings_path, format!("{new_content}\n"))
        .map_err(|e| format!("Cannot write .claude/settings.json: {e}"))?;

    Ok(Some("Added PostToolUse hook to .claude/settings.json".into()))
}

pub fn cmd_setup(path: &PathBuf) {
    match setup_internal(path) {
        Ok(output) => println!("{output}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
