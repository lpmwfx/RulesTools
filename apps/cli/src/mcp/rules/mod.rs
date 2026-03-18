mod registry;
mod handlers;

use serde_json::Value;
use crate::mcp::{self, ToolResult, ToolDef};
use std::path::PathBuf;

/// Run the MCP rules server (stdio loop).
pub fn run() {
    let tool_defs = definitions();
    mcp::run_server("rules", tool_defs, handle);
}

/// Find Rules repo — check common locations (unified).
pub fn find_rules_repo() -> Option<PathBuf> {
    // 1. Environment variable
    if let Ok(path) = std::env::var("RULES_REPO") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    // 2. Sibling to current exe's grandparent (workspace layout)
    if let Ok(exe) = std::env::current_exe() {
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

fn definitions() -> Vec<ToolDef> {
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
            description: "Search rules by keyword. Matches tags, concepts, keywords, title with weighted scoring.\n\nArgs:\n    query: Search terms (e.g. \"ownership threading types\")\n    category: Optional category filter\n    limit: Max results (default 10)".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search terms" },
                    "category": { "type": "string", "description": "Optional category filter (rust, python, global, etc.)" },
                    "limit": { "type": "integer", "description": "Max results (default 10)" }
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
            description: "Get combined rules context for given languages and optional topics.\n\nArgs:\n    languages: Language categories (e.g. [\"rust\", \"slint\"])\n    topics: Optional concept/tag filter — returns only files matching these topics\n    quick_ref: If true, return only quick-ref files (compact onboarding). Default: false\n\nExamples:\n    get_context([\"rust\", \"slint\"])                    → all rust + slint rules\n    get_context([\"rust\"], topics: [\"workspace\"])       → rust files about workspace + matching global\n    get_context([\"rust\", \"slint\"], quick_ref: true)    → 3 files: global + rust + slint quick-ref".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "languages": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Languages to get rules for"
                    },
                    "topics": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional concept/tag filter"
                    },
                    "quick_ref": {
                        "type": "boolean",
                        "description": "If true, return only quick-ref summary files (compact onboarding)"
                    }
                },
                "required": ["languages"]
            }),
        },
        ToolDef {
            name: "get_learning_path".into(),
            description: "Get rules in implementation order — foundational first, dependent later.\nReturns rules grouped in phases (layers). Phase 1 = read first, etc.\n\nArgs:\n    languages: Language categories (e.g. [\"python\", \"js\"])\n    phase: Optional layer number. Omit for full path overview.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "languages": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Language categories"
                    },
                    "phase": {
                        "type": "integer",
                        "description": "Optional layer number (1-6). Omit for full path."
                    }
                },
                "required": ["languages"]
            }),
        },
        ToolDef {
            name: "get_related".into(),
            description: "Get related rules by following graph edges from a specific rule file.\nShows requires, required_by, feeds, fed_by, and related edges.\n\nArgs:\n    file: Path relative to repo root (e.g. \"python/types.md\")".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file": { "type": "string", "description": "Rule file path" }
                },
                "required": ["file"]
            }),
        },
    ]
}

fn handle(name: &str, args: &Value) -> ToolResult {
    let repo = match find_rules_repo() {
        Some(r) => r,
        None => return ToolResult::error("Rules repo not found. Set RULES_REPO env var."),
    };

    match name {
        "help" => handlers::help(&repo),
        "get_rule" => handlers::get_rule(&repo, args),
        "search_rules" => handlers::search_rules(&repo, args),
        "list_rules" => handlers::list_rules(&repo, args),
        "get_context" => handlers::get_context(&repo, args),
        "get_learning_path" => handlers::learning_path(&repo, args),
        "get_related" => handlers::get_related(&repo, args),
        _ => ToolResult::error(format!("Unknown tool: {name}")),
    }
}
