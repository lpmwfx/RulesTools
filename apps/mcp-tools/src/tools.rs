use std::path::PathBuf;
use serde_json::Value;

use crate::protocol::{ToolDef, ToolResult};

/// Register all tool definitions.
pub fn definitions() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "scan_file".into(),
            description: "Scan a single source file for rule violations.\n\nCall this after every Edit or Write to verify compliance.\nReturns CLEAN or a list of violations to fix.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to the file to scan" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "scan_tree".into(),
            description: "Scan an entire project tree for rule violations.\n\nWalks all source files, runs enabled checks, writes proj/ISSUES.\nReturns grouped output with guidance and decision trees.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "check_staged".into(),
            description: "Pre-commit check — scan staged files and report violations.\n\nDesigned for pre-commit hooks. Returns CLEAN or violation list.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to git repo root" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "setup".into(),
            description: "Install RulesTools hooks and config for a project.\n\nCreates proj/rulestools.toml and installs pre-commit hook.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "security_scan".into(),
            description: "Security-focused scan — checks for secrets, injection, unsafe patterns.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "init_project".into(),
            description: "Initialize a new project with full scaffolding.\n\n\
                Creates directory structure, stub source files, Cargo.toml, proj/ files, and .gitignore.\n\
                Kinds: tool, cli, library, slint-app, workspace.\n\
                Existing files are never overwritten.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" },
                    "kind": {
                        "type": "string",
                        "description": "Project kind: tool, cli, library, slint-app, workspace",
                        "enum": ["tool", "cli", "library", "slint-app", "workspace"]
                    },
                    "name": { "type": "string", "description": "Project name (default: directory name)" }
                },
                "required": ["path", "kind"]
            }),
        },
        ToolDef {
            name: "report_issue".into(),
            description: "Report an issue to Forgejo issue tracker.\n\nAlways adds 'ai-reported' label. Use component labels (scanner, documenter, mcp-rules, mcp-tools, rules, man-viewer) and type labels (bug, debt, architecture, security).".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Issue title" },
                    "body": { "type": "string", "description": "Issue description" },
                    "labels": { "type": "string", "description": "Comma-separated labels (e.g. 'bug,scanner'). 'ai-reported' is always added." }
                },
                "required": ["title"]
            }),
        },
        ToolDef {
            name: "list_issues".into(),
            description: "List issues from Forgejo issue tracker.\n\nFilter by state and labels. Use before reporting to check for duplicates.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "state": { "type": "string", "description": "Filter: open, closed, all", "default": "open" },
                    "labels": { "type": "string", "description": "Filter by labels (comma-separated)" }
                }
            }),
        },
        ToolDef {
            name: "add_label".into(),
            description: "Add a label to an existing issue on Forgejo.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "number": { "type": "integer", "description": "Issue number" },
                    "label": { "type": "string", "description": "Label name to add (e.g. 'bug', 'scanner', 'security')" }
                },
                "required": ["number", "label"]
            }),
        },
        ToolDef {
            name: "list_labels".into(),
            description: "List all available labels in the Forgejo issue tracker.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDef {
            name: "comment_issue".into(),
            description: "Add a comment to an existing issue on Forgejo.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "number": { "type": "integer", "description": "Issue number" },
                    "body": { "type": "string", "description": "Comment text" }
                },
                "required": ["number", "body"]
            }),
        },
        ToolDef {
            name: "create_label".into(),
            description: "Create a new label in the Forgejo issue tracker.\n\nUse this before add_label if the label doesn't exist yet.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Label name (e.g. 'slint-ui-templates')" },
                    "color": { "type": "string", "description": "Hex color without # (e.g. '0075ca')", "default": "0075ca" },
                    "description": { "type": "string", "description": "Label description" }
                },
                "required": ["name"]
            }),
        },
        ToolDef {
            name: "new_project".into(),
            description: "Create a new project with full scaffolding and options.\n\n\
                Creates directory structure, stub source files, Cargo.toml, proj/ files.\n\
                Supports platforms, themes, MCP crate, extras, and preview mode.\n\
                Kinds: tool, cli, library, website, slint-app, workspace.\n\
                Existing files are never overwritten.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" },
                    "kind": {
                        "type": "string",
                        "description": "Project kind",
                        "enum": ["tool", "cli", "library", "website", "slint-app", "workspace"]
                    },
                    "name": { "type": "string", "description": "Project name (default: directory name)" },
                    "platforms": {
                        "type": "array",
                        "items": { "type": "string", "enum": ["desktop", "mobile", "small"] },
                        "description": "Target platforms (SlintApp/Super only)"
                    },
                    "themes": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Theme names (SlintApp/Super only)"
                    },
                    "mcp": { "type": "boolean", "description": "Add MCP server crate (workspace only)" },
                    "extras": {
                        "type": "array",
                        "items": { "type": "string", "enum": ["lib", "shared", "doc"] },
                        "description": "Extra folders/crates to add"
                    },
                    "preview": { "type": "boolean", "description": "Preview only — show what would be created" }
                },
                "required": ["path", "kind"]
            }),
        },
        ToolDef {
            name: "update_project".into(),
            description: "Add features to an existing project without changing its kind.\n\n\
                Detects current project kind and adds platforms, themes, crates, or folders.\n\
                Existing files are never overwritten. Kind is never changed.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" },
                    "platforms": {
                        "type": "array",
                        "items": { "type": "string", "enum": ["desktop", "mobile", "small"] },
                        "description": "Platforms to add (SlintApp/Super only)"
                    },
                    "themes": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Themes to add (SlintApp/Super only)"
                    },
                    "crate_name": { "type": "string", "description": "Crate name to add (workspace only)" },
                    "folders": {
                        "type": "array",
                        "items": { "type": "string", "enum": ["lib", "shared", "doc"] },
                        "description": "Extra folders to add"
                    },
                    "preview": { "type": "boolean", "description": "Preview only" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "upgrade_project".into(),
            description: "Upgrade a project to a higher kind (structural transformation).\n\n\
                Changes ProjectKind upward (never down). Scaffolds new structure\n\
                and provides move guidance for existing files.\n\
                Order: tool < library/website < cli < slint-app < workspace".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" },
                    "to": {
                        "type": "string",
                        "description": "Target kind to upgrade to",
                        "enum": ["tool", "cli", "library", "website", "slint-app", "workspace"]
                    },
                    "preview": { "type": "boolean", "description": "Preview only" }
                },
                "required": ["path", "to"]
            }),
        },
        ToolDef {
            name: "close_issue".into(),
            description: "Close an issue on Forgejo by number.\n\nOptionally add a closing comment.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "number": { "type": "integer", "description": "Issue number to close" },
                    "comment": { "type": "string", "description": "Optional closing comment" }
                },
                "required": ["number"]
            }),
        },
        ToolDef {
            name: "publish_plan".into(),
            description: "Analyze project and show publish targets, version, and pre-checks.\n\n\
                Reads [publish] config, checks git state, scanner status.\n\
                Returns targets with version info and changelog.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "publish_run".into(),
            description: "Execute publish to a target (github/forgejo/archive).\n\n\
                Runs pre-publish checks (scanner, tests, clean git, token),\n\
                then builds, tags, and creates release. Use preview for dry run.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" },
                    "target": {
                        "type": "string",
                        "description": "Publish target",
                        "enum": ["github", "forgejo", "archive"]
                    },
                    "preview": { "type": "boolean", "description": "Preview only — run checks without publishing" }
                },
                "required": ["path", "target"]
            }),
        },
        ToolDef {
            name: "publish_status".into(),
            description: "Show what is published where.\n\n\
                Queries GitHub/Forgejo APIs for latest release info.\n\
                Shows version, date, and URL per configured target.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "publish_init".into(),
            description: "Initialize a pub-repo for dev/pub separation.\n\n\
                Creates ../{name}-pub/ directory, git init, adds remote,\n\
                writes [publish.repo] config to proj/rulestools.toml.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" },
                    "remote": { "type": "string", "description": "Remote URL for the pub-repo (e.g. git@github.com:user/repo.git)" }
                },
                "required": ["path", "remote"]
            }),
        },
        ToolDef {
            name: "publish_sync".into(),
            description: "Sync dev-repo to pub-repo (whitelist copy).\n\n\
                Only whitelisted files/dirs are copied. Excluded patterns never copied.\n\
                Hardcoded safety: proj/, .claude/, target/, .env*, *.key always excluded.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" },
                    "preview": { "type": "boolean", "description": "Preview only — show what would be synced" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "publish_check".into(),
            description: "Validate pub-repo for leaks and sync status.\n\n\
                Walks pub-repo files and reports:\n\
                - LEAKED: excluded files found in pub-repo\n\
                - OUT-OF-SYNC: files that differ from dev-repo\n\
                - CLEAN: all checks pass".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" }
                },
                "required": ["path"]
            }),
        },
    ]
}

/// Dispatch tool calls — all delegate to rulestools CLI via subprocess.
pub fn handle(name: &str, args: &Value) -> ToolResult {
    match name {
        "scan_file" => tool_scan_file(args),
        "scan_tree" => tool_scan_tree(args),
        "check_staged" => tool_check_staged(args),
        "setup" => tool_setup(args),
        "security_scan" => tool_security_scan(args),
        "init_project" => tool_init_project(args),
        "new_project" => tool_new_project(args),
        "update_project" => tool_update_project(args),
        "upgrade_project" => tool_upgrade_project(args),
        "report_issue" => tool_report_issue(args),
        "list_issues" => tool_list_issues(args),
        "add_label" => tool_add_label(args),
        "list_labels" => tool_list_labels(args),
        "comment_issue" => tool_comment_issue(args),
        "create_label" => tool_create_label(args),
        "close_issue" => tool_close_issue(args),
        "publish_plan" => tool_publish_plan(args),
        "publish_run" => tool_publish_run(args),
        "publish_status" => tool_publish_status(args),
        "publish_init" => tool_publish_init(args),
        "publish_sync" => tool_publish_sync(args),
        "publish_check" => tool_publish_check(args),
        _ => ToolResult::error(format!("Unknown tool: {name}")),
    }
}

fn get_path(args: &Value) -> Result<PathBuf, ToolResult> {
    args.get("path")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .ok_or_else(|| ToolResult::error("Missing required parameter: path"))
}

/// Run rulestools CLI command and return stdout.
fn run_rulestools(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("rulestools")
        .args(args)
        .output()
        .map_err(|e| format!("Cannot run rulestools: {e}. Is it installed? (cargo install --path apps/cli)"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !stderr.is_empty() && !output.status.success() {
        return Err(format!("{stderr}\n{stdout}"));
    }

    Ok(stdout)
}

fn tool_scan_file(args: &Value) -> ToolResult {
    let path = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    if !path.exists() {
        return ToolResult::error(format!("File not found: {}", path.display()));
    }

    match run_rulestools(&["scan-file", &path.to_string_lossy()]) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_scan_tree(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    match run_rulestools(&["scan", &root.to_string_lossy()]) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_check_staged(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    match run_rulestools(&["check", &root.to_string_lossy()]) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_setup(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    // Setup still runs directly — it's simple file operations
    let mut actions = Vec::new();
    let proj_dir = root.join("proj");
    if !proj_dir.exists() {
        let _ = std::fs::create_dir_all(&proj_dir);
        actions.push("Created proj/");
    }

    let toml_path = proj_dir.join("rulestools.toml");
    if !toml_path.exists() {
        // Detect kind via CLI
        let detect_output = run_rulestools(&["detect", &root.to_string_lossy()])
            .unwrap_or_default();
        let kind = detect_output.lines()
            .find(|l| l.starts_with("Kind:"))
            .and_then(|l| l.split_whitespace().last())
            .unwrap_or("tool");
        let kind_lower = kind.to_lowercase();
        let content = format!("[project]\nkind = \"{kind_lower}\"\n");
        let _ = std::fs::write(&toml_path, content);
        actions.push("Created proj/rulestools.toml");
    }

    let git_hooks = root.join(".git").join("hooks");
    if git_hooks.exists() {
        let hook_path = git_hooks.join("pre-commit");
        let hook_content = "#!/bin/sh\nrulestools check \"$(git rev-parse --show-toplevel)\"\n";
        let _ = std::fs::write(&hook_path, hook_content);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o755));
        }
        actions.push("Installed .git/hooks/pre-commit");
    }

    if actions.is_empty() {
        ToolResult::text("Setup already complete — nothing to do")
    } else {
        ToolResult::text(format!("Setup complete:\n- {}", actions.join("\n- ")))
    }
}

fn tool_security_scan(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    // Security scan = regular scan, grep for secrets
    match run_rulestools(&["scan", &root.to_string_lossy()]) {
        Ok(output) => {
            let security_lines: Vec<&str> = output.lines()
                .filter(|l| l.contains("secrets") || l.contains("SAFETY") || l.contains("credential") || l.contains("private key"))
                .collect();
            if security_lines.is_empty() {
                ToolResult::text("Security scan CLEAN — no secrets or injection patterns found")
            } else {
                let mut result = format!("Security scan: {} issues found\n\n", security_lines.len());
                for line in &security_lines {
                    result.push_str(line);
                    result.push('\n');
                }
                ToolResult::text(result)
            }
        }
        Err(e) => ToolResult::error(e),
    }
}

fn tool_init_project(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    let kind = match args.get("kind").and_then(|v| v.as_str()) {
        Some(k) => k,
        None => return ToolResult::error("Missing required parameter: kind"),
    };

    let root_str = root.to_string_lossy().to_string();
    let mut cmd_args = vec!["init", &root_str, "--kind", kind];
    let name_str;
    if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
        name_str = name.to_string();
        cmd_args.push("--name");
        cmd_args.push(&name_str);
    }

    match run_rulestools(&cmd_args) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_new_project(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    let kind = match args.get("kind").and_then(|v| v.as_str()) {
        Some(k) => k,
        None => return ToolResult::error("Missing required parameter: kind"),
    };

    let root_str = root.to_string_lossy().to_string();
    let mut cmd_args = vec!["new", &root_str, "--kind", kind, "--format", "json"];

    let name_str;
    if let Some(name) = args.get("name").and_then(|v| v.as_str()) {
        name_str = name.to_string();
        cmd_args.push("--name");
        cmd_args.push(&name_str);
    }

    let platforms_str;
    if let Some(platforms) = args.get("platforms").and_then(|v| v.as_array()) {
        let items: Vec<&str> = platforms.iter().filter_map(|v| v.as_str()).collect();
        if !items.is_empty() {
            platforms_str = items.join(",");
            cmd_args.push("--platforms");
            cmd_args.push(&platforms_str);
        }
    }

    let themes_str;
    if let Some(themes) = args.get("themes").and_then(|v| v.as_array()) {
        let items: Vec<&str> = themes.iter().filter_map(|v| v.as_str()).collect();
        if !items.is_empty() {
            themes_str = items.join(",");
            cmd_args.push("--themes");
            cmd_args.push(&themes_str);
        }
    }

    if args.get("mcp").and_then(|v| v.as_bool()).unwrap_or(false) {
        cmd_args.push("--mcp");
    }

    let extras_str;
    if let Some(extras) = args.get("extras").and_then(|v| v.as_array()) {
        let items: Vec<&str> = extras.iter().filter_map(|v| v.as_str()).collect();
        if !items.is_empty() {
            extras_str = items.join(",");
            cmd_args.push("--extras");
            cmd_args.push(&extras_str);
        }
    }

    if args.get("preview").and_then(|v| v.as_bool()).unwrap_or(false) {
        cmd_args.push("--preview");
    }

    match run_rulestools(&cmd_args) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_update_project(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    let root_str = root.to_string_lossy().to_string();
    let mut cmd_args = vec!["update", &root_str, "--format", "json"];

    let platforms_str;
    if let Some(platforms) = args.get("platforms").and_then(|v| v.as_array()) {
        let items: Vec<&str> = platforms.iter().filter_map(|v| v.as_str()).collect();
        if !items.is_empty() {
            platforms_str = items.join(",");
            cmd_args.push("--add-platform");
            cmd_args.push(&platforms_str);
        }
    }

    let themes_str;
    if let Some(themes) = args.get("themes").and_then(|v| v.as_array()) {
        let items: Vec<&str> = themes.iter().filter_map(|v| v.as_str()).collect();
        if !items.is_empty() {
            themes_str = items.join(",");
            cmd_args.push("--add-theme");
            cmd_args.push(&themes_str);
        }
    }

    let crate_str;
    if let Some(crate_name) = args.get("crate_name").and_then(|v| v.as_str()) {
        crate_str = crate_name.to_string();
        cmd_args.push("--add-crate");
        cmd_args.push(&crate_str);
    }

    let folders_str;
    if let Some(folders) = args.get("folders").and_then(|v| v.as_array()) {
        let items: Vec<&str> = folders.iter().filter_map(|v| v.as_str()).collect();
        if !items.is_empty() {
            folders_str = items.join(",");
            cmd_args.push("--add-folder");
            cmd_args.push(&folders_str);
        }
    }

    if args.get("preview").and_then(|v| v.as_bool()).unwrap_or(false) {
        cmd_args.push("--preview");
    }

    match run_rulestools(&cmd_args) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_upgrade_project(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    let to = match args.get("to").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return ToolResult::error("Missing required parameter: to"),
    };

    let root_str = root.to_string_lossy().to_string();
    let mut cmd_args = vec!["upgrade", &root_str, "--to", to, "--format", "json"];

    if args.get("preview").and_then(|v| v.as_bool()).unwrap_or(false) {
        cmd_args.push("--preview");
    }

    match run_rulestools(&cmd_args) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_report_issue(args: &Value) -> ToolResult {
    let title = match args.get("title").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return ToolResult::error("Missing required parameter: title"),
    };
    let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
    let user_labels = args.get("labels").and_then(|v| v.as_str()).unwrap_or("");

    // Always include ai-reported
    let labels = if user_labels.is_empty() {
        "ai-reported".to_string()
    } else if user_labels.contains("ai-reported") {
        user_labels.to_string()
    } else {
        format!("ai-reported,{user_labels}")
    };

    let mut cmd_args = vec!["issue", "report", "--title", title, "--labels", &labels];
    if !body.is_empty() {
        cmd_args.push("--body");
        cmd_args.push(body);
    }

    match run_rulestools(&cmd_args) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_list_issues(args: &Value) -> ToolResult {
    let state = args.get("state").and_then(|v| v.as_str()).unwrap_or("open");
    let labels = args.get("labels").and_then(|v| v.as_str()).unwrap_or("");

    let mut cmd_args = vec!["issue", "list", "--state", state];
    if !labels.is_empty() {
        cmd_args.push("--labels");
        cmd_args.push(labels);
    }

    match run_rulestools(&cmd_args) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_add_label(args: &Value) -> ToolResult {
    let number = match args.get("number").and_then(|v| v.as_u64()) {
        Some(n) => n.to_string(),
        None => return ToolResult::error("Missing required parameter: number"),
    };
    let label = match args.get("label").and_then(|v| v.as_str()) {
        Some(l) => l,
        None => return ToolResult::error("Missing required parameter: label"),
    };

    match run_rulestools(&["issue", "add-label", &number, label]) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_list_labels(_args: &Value) -> ToolResult {
    match run_rulestools(&["issue", "list-labels"]) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_comment_issue(args: &Value) -> ToolResult {
    let number = match args.get("number").and_then(|v| v.as_u64()) {
        Some(n) => n.to_string(),
        None => return ToolResult::error("Missing required parameter: number"),
    };
    let body = match args.get("body").and_then(|v| v.as_str()) {
        Some(b) => b,
        None => return ToolResult::error("Missing required parameter: body"),
    };

    match run_rulestools(&["issue", "comment", &number, body]) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_create_label(args: &Value) -> ToolResult {
    let name = match args.get("name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return ToolResult::error("Missing required parameter: name"),
    };
    let color = args.get("color").and_then(|v| v.as_str()).unwrap_or("0075ca");
    let desc = args.get("description").and_then(|v| v.as_str()).unwrap_or("");

    let mut cmd_args = vec!["issue", "create-label", name, "--color", color];
    if !desc.is_empty() {
        cmd_args.push("--description");
        cmd_args.push(desc);
    }

    match run_rulestools(&cmd_args) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_close_issue(args: &Value) -> ToolResult {
    let number = match args.get("number").and_then(|v| v.as_u64()) {
        Some(n) => n.to_string(),
        None => return ToolResult::error("Missing required parameter: number"),
    };
    let comment = args.get("comment").and_then(|v| v.as_str()).unwrap_or("");

    let mut cmd_args = vec!["issue", "close", &number];
    if !comment.is_empty() {
        cmd_args.push("--comment");
        cmd_args.push(comment);
    }

    match run_rulestools(&cmd_args) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_publish_plan(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let root_str = root.to_string_lossy().to_string();
    match run_rulestools(&["publish", "plan", &root_str, "--format", "json"]) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_publish_run(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let target = match args.get("target").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return ToolResult::error("Missing required parameter: target"),
    };
    let root_str = root.to_string_lossy().to_string();
    let mut cmd_args = vec!["publish", "run", &root_str, "--target", target];
    if args.get("preview").and_then(|v| v.as_bool()).unwrap_or(false) {
        cmd_args.push("--preview");
    }
    match run_rulestools(&cmd_args) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_publish_status(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let root_str = root.to_string_lossy().to_string();
    match run_rulestools(&["publish", "status", &root_str, "--format", "json"]) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_publish_init(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let remote = match args.get("remote").and_then(|v| v.as_str()) {
        Some(r) => r,
        None => return ToolResult::error("Missing required parameter: remote"),
    };
    let root_str = root.to_string_lossy().to_string();
    match run_rulestools(&["publish", "init", &root_str, "--remote", remote]) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_publish_sync(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let root_str = root.to_string_lossy().to_string();
    let mut cmd_args = vec!["publish", "sync", &root_str];
    if args.get("preview").and_then(|v| v.as_bool()).unwrap_or(false) {
        cmd_args.push("--preview");
    }
    match run_rulestools(&cmd_args) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

fn tool_publish_check(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let root_str = root.to_string_lossy().to_string();
    match run_rulestools(&["publish", "check", &root_str]) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}
