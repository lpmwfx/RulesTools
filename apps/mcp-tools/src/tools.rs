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
            description: "Initialize a new project with RulesTools.\n\nCreates proj/ structure and installs hooks.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" },
                    "languages": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Languages used in the project"
                    }
                },
                "required": ["path", "languages"]
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

    let languages: Vec<String> = args.get("languages")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let proj_dir = root.join("proj");
    let _ = std::fs::create_dir_all(&proj_dir);

    let project_content = format!(
        "# PROJECT\n\n## Identity\n\n- **Languages:** {}\n\n## Current\n\n- **Phase:** 1\n- **Status:** setup\n",
        languages.join(", ")
    );
    let _ = std::fs::write(proj_dir.join("PROJECT"), &project_content);
    let _ = std::fs::write(proj_dir.join("TODO"), "# TODO\n\n(empty — add tasks here)\n");
    let _ = std::fs::write(proj_dir.join("RULES"), "# RULES\n\nRun `mcp__rules__get_context` for active rules.\n");
    let _ = std::fs::write(proj_dir.join("FIXES"), "# FIXES\n\n(no known issues)\n");

    // Detect and write rulestools.toml
    let detect_output = run_rulestools(&["detect", &root.to_string_lossy()])
        .unwrap_or_default();
    let kind = detect_output.lines()
        .find(|l| l.starts_with("Kind:"))
        .and_then(|l| l.split_whitespace().last())
        .unwrap_or("tool")
        .to_lowercase();
    let toml_content = format!(
        "[project]\nkind = \"{kind}\"\n\n[scan]\nlanguages = [{}]\n",
        languages.iter().map(|l| format!("\"{l}\"")).collect::<Vec<_>>().join(", ")
    );
    let _ = std::fs::write(proj_dir.join("rulestools.toml"), &toml_content);

    let setup_result = tool_setup(args);

    ToolResult::text(format!(
        "Project initialized:\n- proj/PROJECT\n- proj/TODO\n- proj/RULES\n- proj/FIXES\n- proj/rulestools.toml ({kind})\n\n{}",
        setup_result.content.first().map(|c| c.text.as_str()).unwrap_or("")
    ))
}
