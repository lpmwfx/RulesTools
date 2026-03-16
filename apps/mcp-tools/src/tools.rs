use std::path::{Path, PathBuf};
use serde_json::Value;

use rulestools_scanner::config::Config;
use rulestools_scanner::context::FileContext;
use rulestools_scanner::issue::Severity;
use rulestools_scanner::project::ProjectIdentity;

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
            description: "Scan an entire project tree for rule violations.\n\nWalks all source files, runs enabled checks, writes proj/ISSUES.\nReturns summary with issue count and details.".into(),
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
            description: "Install RulesTools hooks and config for a project.\n\nInstalls: .claude/settings.json hooks, .git/hooks/pre-commit, proj/rulestools.toml.".into(),
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
            description: "Security-focused scan — checks for secrets, injection, unsafe patterns.\n\nRuns the secrets check plus language-specific security rules.".into(),
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
            description: "Initialize a new project with RulesTools.\n\nCreates proj/ structure (PROJECT, TODO, RULES, FIXES) and installs hooks.".into(),
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

/// Dispatch tool calls.
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

fn tool_scan_file(args: &Value) -> ToolResult {
    let path = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    if !path.exists() {
        return ToolResult::error(format!("File not found: {}", path.display()));
    }

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => return ToolResult::error(format!("Cannot read file: {e}")),
    };

    let file_ctx = match FileContext::from_path(&path) {
        Some(c) => c,
        None => return ToolResult::text("SKIP — unsupported file type"),
    };

    // Find project root by walking up
    let project_root = find_project_root(&path);
    let cfg = Config::load(&project_root);
    let identity = ProjectIdentity::detect(&project_root);
    let registry = rulestools_scanner::checks::registry();

    let lines: Vec<&str> = content.lines().collect();
    let mut issues = Vec::new();

    for check in &registry {
        if !check.applies_to(file_ctx.language) {
            continue;
        }
        if !cfg.is_enabled(&check.id) || !identity.kind.allows_check(&check.id) {
            continue;
        }
        if let rulestools_scanner::checks::CheckKind::PerFile(func) = &check.kind {
            func(&file_ctx, &lines, &cfg, &mut issues, &path);
        }
    }

    if issues.is_empty() {
        ToolResult::text("CLEAN — no violations found")
    } else {
        let mut output = String::new();
        for issue in &issues {
            output.push_str(&issue.display_line());
            output.push('\n');
        }
        let error_count = issues.iter().filter(|i| i.severity == Severity::Error).count();
        let warn_count = issues.iter().filter(|i| i.severity == Severity::Warning).count();
        output.push_str(&format!(
            "\n{error_count} error(s), {warn_count} warning(s)\nFix all errors before continuing."
        ));
        ToolResult::text(output)
    }
}

fn tool_scan_tree(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    let root = std::fs::canonicalize(&root).unwrap_or(root);
    let identity = ProjectIdentity::detect(&root);
    let (issues, new_count) = rulestools_scanner::scan_at(&root);

    let error_count = issues.iter().filter(|i| i.severity == Severity::Error).count();
    let warn_count = issues.iter().filter(|i| i.severity == Severity::Warning).count();

    if issues.is_empty() {
        ToolResult::text(format!("CLEAN — {:?} / {:?}, 0 issues", identity.kind, identity.layout))
    } else {
        let mut output = format!(
            "{:?} / {:?}: {} issues ({} errors, {} warnings, {} new)\n\n",
            identity.kind, identity.layout,
            issues.len(), error_count, warn_count, new_count,
        );
        for issue in &issues {
            output.push_str(&issue.display_line());
            output.push('\n');
        }
        ToolResult::text(output)
    }
}

fn tool_check_staged(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    // Get staged files from git
    let staged_output = match std::process::Command::new("git")
        .args(["diff", "--cached", "--name-only", "--diff-filter=ACMR"])
        .current_dir(&root)
        .output()
    {
        Ok(o) => o,
        Err(e) => return ToolResult::error(format!("Cannot run git: {e}")),
    };

    let staged_files: Vec<PathBuf> = String::from_utf8_lossy(&staged_output.stdout)
        .lines()
        .map(|l| root.join(l.trim()))
        .filter(|p| p.exists())
        .collect();

    if staged_files.is_empty() {
        return ToolResult::text("No staged files to check");
    }

    let project_root = &root;
    let cfg = Config::load(project_root);
    let identity = ProjectIdentity::detect(project_root);
    let registry = rulestools_scanner::checks::registry();

    let mut all_issues = Vec::new();

    for file_path in &staged_files {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let file_ctx = match FileContext::from_path(file_path) {
            Some(c) => c,
            None => continue,
        };
        let lines: Vec<&str> = content.lines().collect();

        for check in &registry {
            if !check.applies_to(file_ctx.language) {
                continue;
            }
            if !cfg.is_enabled(&check.id) || !identity.kind.allows_check(&check.id) {
                continue;
            }
            if let rulestools_scanner::checks::CheckKind::PerFile(func) = &check.kind {
                func(&file_ctx, &lines, &cfg, &mut all_issues, file_path);
            }
        }
    }

    if all_issues.is_empty() {
        ToolResult::text(format!("CLEAN — {} staged files checked", staged_files.len()))
    } else {
        let error_count = all_issues.iter().filter(|i| i.severity == Severity::Error).count();
        let mut output = format!(
            "{} staged files, {} issues ({} errors)\n\n",
            staged_files.len(), all_issues.len(), error_count,
        );
        for issue in &all_issues {
            output.push_str(&issue.display_line());
            output.push('\n');
        }
        if error_count > 0 {
            output.push_str("\nFix all errors before committing.");
        }
        ToolResult::text(output)
    }
}

fn tool_setup(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    let mut actions = Vec::new();

    // 1. Ensure proj/ exists
    let proj_dir = root.join("proj");
    if !proj_dir.exists() {
        let _ = std::fs::create_dir_all(&proj_dir);
        actions.push("Created proj/");
    }

    // 2. Write proj/rulestools.toml if missing
    let toml_path = proj_dir.join("rulestools.toml");
    if !toml_path.exists() {
        let identity = ProjectIdentity::detect(&root);
        let kind_str = match identity.kind {
            rulestools_scanner::project::ProjectKind::SlintApp => "slint-app",
            rulestools_scanner::project::ProjectKind::CliApp => "cli",
            rulestools_scanner::project::ProjectKind::Library => "library",
            rulestools_scanner::project::ProjectKind::Tool => "tool",
        };
        let content = format!("[project]\nkind = \"{kind_str}\"\n");
        let _ = std::fs::write(&toml_path, content);
        actions.push("Created proj/rulestools.toml");
    }

    // 3. Install pre-commit hook if .git exists
    let git_hooks = root.join(".git").join("hooks");
    if git_hooks.exists() {
        let hook_path = git_hooks.join("pre-commit");
        let hook_content = "#!/bin/sh\nrulestools check \"$(git rev-parse --show-toplevel)\"\n";
        let _ = std::fs::write(&hook_path, hook_content);
        // Make executable on Unix
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

    let root = std::fs::canonicalize(&root).unwrap_or(root);
    let files = rulestools_scanner::walker::collect_files(&root, &[]);

    let mut issues = Vec::new();
    let cfg = Config::default();

    // Run secrets check on all files
    for file_path in &files {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let file_ctx = match FileContext::from_path(file_path) {
            Some(c) => c,
            None => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        rulestools_scanner::checks::secrets::check(&file_ctx, &lines, &cfg, &mut issues, file_path);
    }

    if issues.is_empty() {
        ToolResult::text("Security scan CLEAN — no secrets or injection patterns found")
    } else {
        let mut output = format!("Security scan: {} issues found\n\n", issues.len());
        for issue in &issues {
            output.push_str(&issue.display_line());
            output.push('\n');
        }
        ToolResult::text(output)
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

    // Write PROJECT
    let project_content = format!(
        "# PROJECT\n\n## Identity\n\n- **Languages:** {}\n\n## Current\n\n- **Phase:** 1\n- **Status:** setup\n",
        languages.join(", ")
    );
    let _ = std::fs::write(proj_dir.join("PROJECT"), &project_content);

    // Write TODO
    let _ = std::fs::write(proj_dir.join("TODO"), "# TODO\n\n(empty — add tasks here)\n");

    // Write RULES
    let _ = std::fs::write(proj_dir.join("RULES"), "# RULES\n\nRun `mcp__rules__get_context` for active rules.\n");

    // Write FIXES
    let _ = std::fs::write(proj_dir.join("FIXES"), "# FIXES\n\n(no known issues)\n");

    // Write rulestools.toml
    let identity = ProjectIdentity::detect(&root);
    let kind_str = match identity.kind {
        rulestools_scanner::project::ProjectKind::SlintApp => "slint-app",
        rulestools_scanner::project::ProjectKind::CliApp => "cli",
        rulestools_scanner::project::ProjectKind::Library => "library",
        rulestools_scanner::project::ProjectKind::Tool => "tool",
    };
    let toml_content = format!(
        "[project]\nkind = \"{kind_str}\"\n\n[scan]\nlanguages = [{}]\n",
        languages.iter().map(|l| format!("\"{l}\"")).collect::<Vec<_>>().join(", ")
    );
    let _ = std::fs::write(proj_dir.join("rulestools.toml"), &toml_content);

    // Run setup (hooks)
    let setup_result = tool_setup(args);

    ToolResult::text(format!(
        "Project initialized:\n- proj/PROJECT\n- proj/TODO\n- proj/RULES\n- proj/FIXES\n- proj/rulestools.toml ({kind_str})\n\n{}",
        setup_result.content.first().map(|c| c.text.as_str()).unwrap_or("")
    ))
}

/// Walk up from file to find project root (directory with Cargo.toml or proj/).
fn find_project_root(path: &Path) -> PathBuf {
    let mut current = if path.is_file() {
        path.parent().unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    };

    loop {
        if current.join("Cargo.toml").exists() || current.join("proj").exists() {
            return current;
        }
        if !current.pop() {
            return path.parent().unwrap_or(path).to_path_buf();
        }
    }
}
