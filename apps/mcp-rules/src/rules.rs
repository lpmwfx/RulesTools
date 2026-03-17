use std::path::{Path, PathBuf};
use serde_json::Value;
use walkdir::WalkDir;

use crate::protocol::{ToolDef, ToolResult};

/// Find Rules repo — check common locations.
fn find_rules_repo() -> Option<PathBuf> {
    // 1. Environment variable
    if let Ok(path) = std::env::var("RULES_REPO") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    // 2. Sibling to current exe's grandparent (workspace layout)
    if let Ok(exe) = std::env::current_exe() {
        // exe is in target/debug/ — go up to workspace root
        let mut dir = exe.clone();
        for _ in 0..4 {
            dir.pop();
        }
        let sibling = dir.join("Rules");
        if sibling.join("global").exists() {
            return Some(sibling);
        }
    }

    // 3. Common dev locations
    for candidate in &[
        "D:/REPO/Rules-dev/Rules",
        "C:/Users/mathi/.cache/rules-mcp/Rules",
    ] {
        let p = PathBuf::from(candidate);
        if p.join("global").exists() {
            return Some(p);
        }
    }

    None
}

/// Register all tool definitions.
pub fn definitions() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "help".into(),
            description: "AI coding rules lookup — overview of available tools and categories.".into(),
            input_schema: serde_json::json!({ "type": "object", "properties": {} }),
        },
        ToolDef {
            name: "get_rule".into(),
            description: "Get full markdown content of a specific rule file.\n\nArgs:\n    file: Path relative to repo root (e.g. \"python/types.md\")".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file": { "type": "string" }
                },
                "required": ["file"]
            }),
        },
        ToolDef {
            name: "search_rules".into(),
            description: "Search rules by keyword. Returns matching rule files with titles.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search terms" },
                    "category": { "type": "string", "description": "Optional category filter (rust, python, global, etc.)" }
                },
                "required": ["query"]
            }),
        },
        ToolDef {
            name: "list_rules".into(),
            description: "List all available rule files, optionally filtered by category.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "category": { "type": "string", "description": "Optional category (rust, python, js, css, global, uiux, etc.)" }
                }
            }),
        },
        ToolDef {
            name: "get_context".into(),
            description: "Get all rules for given languages. Returns grouped rule content.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "languages": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Languages to get rules for"
                    }
                },
                "required": ["languages"]
            }),
        },
    ]
}

/// Dispatch tool calls.
pub fn handle(name: &str, args: &Value) -> ToolResult {
    let repo = match find_rules_repo() {
        Some(r) => r,
        None => return ToolResult::error("Rules repo not found. Set RULES_REPO env var."),
    };

    match name {
        "help" => tool_help(),
        "get_rule" => tool_get_rule(&repo, args),
        "search_rules" => tool_search_rules(&repo, args),
        "list_rules" => tool_list_rules(&repo, args),
        "get_context" => tool_get_context(&repo, args),
        _ => ToolResult::error(format!("Unknown tool: {name}")),
    }
}

fn tool_help() -> ToolResult {
    ToolResult::text(
        "AI coding rules lookup — Python, JS, CSS, C++, Rust, Kotlin standards.\n\n\
        Tools:\n\
        - get_rule(file)         — full markdown of one rule\n\
        - search_rules(query)    — keyword search across all rules\n\
        - list_rules(category?)  — browse available rules\n\
        - get_context(languages) — all rules for given languages\n\n\
        Categories: global, rust, slint, python, js, css, kotlin, csharp, uiux, project-files, catalog\n\n\
        Libraries:\n\
        - slint-ui-templates: UI + adapter foundation for Slint apps (crates.io/crates/slint-ui-templates)\n\
          get_rule(\"catalog/slint-ui-templates.md\") for full docs\n\n\
        Example: get_rule(\"global/startup.md\")"
    )
}

fn tool_get_rule(repo: &Path, args: &Value) -> ToolResult {
    let file = match args.get("file").and_then(|v| v.as_str()) {
        Some(f) => f,
        None => return ToolResult::error("Missing required parameter: file"),
    };

    let path = repo.join(file);
    match std::fs::read_to_string(&path) {
        Ok(content) => ToolResult::text(content),
        Err(_) => ToolResult::error(format!("Rule not found: {file}")),
    }
}

fn tool_search_rules(repo: &Path, args: &Value) -> ToolResult {
    let query = match args.get("query").and_then(|v| v.as_str()) {
        Some(q) => q.to_lowercase(),
        None => return ToolResult::error("Missing required parameter: query"),
    };
    let category = args.get("category").and_then(|v| v.as_str());

    let tokens: Vec<&str> = query.split_whitespace().collect();
    let mut matches: Vec<(String, usize)> = Vec::new();

    for entry in WalkDir::new(repo).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let rel = match path.strip_prefix(repo) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };

        // Category filter
        if let Some(cat) = category {
            if !rel.starts_with(cat) {
                continue;
            }
        }

        // Score by matching tokens against filename and content
        let content = std::fs::read_to_string(path).unwrap_or_default().to_lowercase();
        let mut score = 0usize;

        for token in &tokens {
            if rel.to_lowercase().contains(token) {
                score += 3;
            }
            if content.contains(token) {
                score += 1;
            }
        }

        if score > 0 {
            matches.push((rel, score));
        }
    }

    matches.sort_by(|a, b| b.1.cmp(&a.1));
    matches.truncate(10);

    if matches.is_empty() {
        ToolResult::text(format!("No rules found matching \"{query}\""))
    } else {
        let mut output = format!("Search results for \"{query}\":\n\n");
        for (file, score) in &matches {
            output.push_str(&format!("  {file} (score: {score})\n"));
        }
        ToolResult::text(output)
    }
}

fn tool_list_rules(repo: &Path, args: &Value) -> ToolResult {
    let category = args.get("category").and_then(|v| v.as_str());

    let mut rules: Vec<String> = Vec::new();

    for entry in WalkDir::new(repo).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let rel = match path.strip_prefix(repo) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };

        if let Some(cat) = category {
            if !rel.starts_with(cat) {
                continue;
            }
        }

        rules.push(rel);
    }

    rules.sort();

    if rules.is_empty() {
        ToolResult::text("No rules found")
    } else {
        let mut output = format!("{} rules:\n\n", rules.len());
        let mut current_category = String::new();
        for rule in &rules {
            let cat = rule.split('/').next().unwrap_or("");
            if cat != current_category {
                current_category = cat.to_string();
                output.push_str(&format!("\n## {cat}\n"));
            }
            output.push_str(&format!("  {rule}\n"));
        }
        ToolResult::text(output)
    }
}

fn tool_get_context(repo: &Path, args: &Value) -> ToolResult {
    let languages: Vec<String> = args.get("languages")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    if languages.is_empty() {
        return ToolResult::error("Missing required parameter: languages");
    }

    // Always include global + uiux
    let mut categories = vec!["global".to_string(), "uiux".to_string()];
    categories.extend(languages);

    let mut output = String::new();

    for cat in &categories {
        let cat_dir = repo.join(cat);
        if !cat_dir.is_dir() {
            continue;
        }

        output.push_str(&format!("\n# {cat}\n\n"));

        let mut files: Vec<PathBuf> = Vec::new();
        for entry in WalkDir::new(&cat_dir).max_depth(1).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && entry.path().extension().and_then(|e| e.to_str()) == Some("md") {
                files.push(entry.path().to_path_buf());
            }
        }
        files.sort();

        for file_path in &files {
            let filename = file_path.file_name().and_then(|f| f.to_str()).unwrap_or("?");
            if let Ok(content) = std::fs::read_to_string(file_path) {
                // Extract RULE: and BANNED: lines
                let rules_banned: Vec<&str> = content
                    .lines()
                    .filter(|l| l.starts_with("RULE:") || l.starts_with("BANNED:"))
                    .collect();

                if !rules_banned.is_empty() {
                    output.push_str(&format!("## {cat}/{filename}\n"));
                    for line in &rules_banned {
                        output.push_str(&format!("  {line}\n"));
                    }
                    output.push('\n');
                }
            }
        }
    }

    if output.is_empty() {
        ToolResult::text("No rules found for given languages")
    } else {
        ToolResult::text(output)
    }
}
