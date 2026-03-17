use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde_json::Value;

use crate::protocol::{ToolDef, ToolResult};
use crate::registry::Registry;

/// Lazy-loaded global registry.
static REGISTRY: OnceLock<Registry> = OnceLock::new();

fn get_registry(repo: &Path) -> Result<&'static Registry, String> {
    if let Some(reg) = REGISTRY.get() {
        return Ok(reg);
    }
    let reg = Registry::load(repo)?;
    // Race is fine — first writer wins, others get that value.
    let _ = REGISTRY.set(reg);
    Ok(REGISTRY.get().unwrap())
}

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
            description: "AI coding rules lookup — overview of available tools and categories."
                .into(),
            input_schema: serde_json::json!({ "type": "object", "properties": {} }),
        },
        ToolDef {
            name: "get_rule".into(),
            description:
                "Get full markdown content of a specific rule file.\n\nArgs:\n    file: Path relative to repo root (e.g. \"python/types.md\")"
                    .into(),
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
            description:
                "Search rules by keyword. Matches tags, concepts, keywords, title with weighted scoring.\n\nArgs:\n    query: Search terms (e.g. \"ownership threading types\")\n    category: Optional category filter\n    limit: Max results (default 10)"
                    .into(),
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
            description:
                "Get combined rules context for given languages and optional topics.\n\nArgs:\n    languages: Language categories (e.g. [\"python\", \"js\"])\n    topics: Optional concept/tag filter (e.g. [\"types\", \"testing\"])"
                    .into(),
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
                    }
                },
                "required": ["languages"]
            }),
        },
        ToolDef {
            name: "get_learning_path".into(),
            description:
                "Get rules in implementation order — foundational first, dependent later.\nReturns rules grouped in phases (layers). Phase 1 = read first, etc.\n\nArgs:\n    languages: Language categories (e.g. [\"python\", \"js\"])\n    phase: Optional layer number. Omit for full path overview."
                    .into(),
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
            description:
                "Get related rules by following graph edges from a specific rule file.\nShows requires, required_by, feeds, fed_by, and related edges.\n\nArgs:\n    file: Path relative to repo root (e.g. \"python/types.md\")"
                    .into(),
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

/// Dispatch tool calls.
pub fn handle(name: &str, args: &Value) -> ToolResult {
    let repo = match find_rules_repo() {
        Some(r) => r,
        None => return ToolResult::error("Rules repo not found. Set RULES_REPO env var."),
    };

    match name {
        "help" => tool_help(&repo),
        "get_rule" => tool_get_rule(&repo, args),
        "search_rules" => tool_search_rules(&repo, args),
        "list_rules" => tool_list_rules(&repo, args),
        "get_context" => tool_get_context(&repo, args),
        "get_learning_path" => tool_learning_path(&repo, args),
        "get_related" => tool_get_related(&repo, args),
        _ => ToolResult::error(format!("Unknown tool: {name}")),
    }
}

fn tool_help(repo: &Path) -> ToolResult {
    let (total, cat_count, cat_list, rule_count, banned_count) = match get_registry(repo) {
        Ok(reg) => {
            let cats = reg.categories();
            (
                reg.len(),
                cats.len(),
                cats.join(", "),
                reg.rule_count(),
                reg.banned_count(),
            )
        }
        Err(_) => (0, 0, String::from("(registry unavailable)"), 0, 0),
    };

    ToolResult::text(format!(
        "# Rules MCP — AI coding standards lookup\n\n\
        **{total} rules** across **{cat_count} categories** ({rule_count} RULE markers, {banned_count} BANNED markers)\n\n\
        ## Tools\n\n\
        | Tool | Purpose | Example |\n\
        |------|---------|--------|\n\
        | `help()` | This overview | — |\n\
        | `search_rules(query)` | Find rules by keyword | `search_rules(\"testing\")` |\n\
        | `get_rule(file)` | Read full rule content | `get_rule(\"python/types.md\")` |\n\
        | `get_context(languages)` | All rules for languages | `get_context([\"python\", \"js\"])` |\n\
        | `get_learning_path(languages)` | Phased reading order | `get_learning_path([\"cpp\"], phase=1)` |\n\
        | `list_rules(category)` | Browse available rules | `list_rules(\"rust\")` |\n\
        | `get_related(file)` | Follow edges to related rules | `get_related(\"python/types.md\")` |\n\n\
        ## Quick start\n\n\
        - **App architecture / folder layout** → `get_context([\"global\"])`\n\
        - **New project setup** → `get_context([\"global\", \"project-files\"])`\n\
        - **UI/UX rules** → `get_context([\"uiux\"])`\n\
        - **Learn a language's rules** → `get_learning_path([\"python\"], phase=1)`\n\
        - **Search a topic** → `search_rules(\"error handling\")`\n\
        - **File size limits** → `get_rule(\"global/file-limits.md\")`\n\
        - **Browse everything** → `list_rules()`\n\n\
        ## Before writing code\n\n\
        1. Check file sizes: `search_rules(\"file limits\")` → split any file at its limit before adding\n\
        2. Read project rules: `get_context([\"global\"])` → architecture + file-size + layer rules\n\
        3. For UI/CSS work: `get_context([\"uiux\"])` → component structure + platform behaviour\n\n\
        ## Categories\n\n\
        {cat_list}"
    ))
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
        Some(q) => q,
        None => return ToolResult::error("Missing required parameter: query"),
    };
    let category = args.get("category").and_then(|v| v.as_str());
    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;

    let reg = match get_registry(repo) {
        Ok(r) => r,
        Err(e) => return ToolResult::error(e),
    };

    let results = reg.search(query, category, limit);
    if results.is_empty() {
        return ToolResult::text(format!("No matching rules found for \"{query}\"."));
    }

    let mut lines = Vec::new();
    for (entry, _score) in &results {
        let tags: String = entry.tags.iter().take(5).cloned().collect::<Vec<_>>().join(", ");
        lines.push(format!("- **{}**: {}", entry.file, entry.title));
        if !tags.is_empty() {
            lines.push(format!("  tags: {tags}"));
        }
    }
    ToolResult::text(lines.join("\n"))
}

fn tool_list_rules(repo: &Path, args: &Value) -> ToolResult {
    let category = args.get("category").and_then(|v| v.as_str());

    let reg = match get_registry(repo) {
        Ok(r) => r,
        Err(e) => return ToolResult::error(e),
    };

    let entries = reg.list(category);
    if entries.is_empty() {
        let available = reg.categories().join(", ");
        return ToolResult::text(format!(
            "No rules found. Available categories: {available}"
        ));
    }

    let mut lines = Vec::new();
    let mut current_cat = String::new();
    for entry in &entries {
        if entry.category != current_cat {
            current_cat = entry.category.clone();
            lines.push(format!("\n### {}", current_cat));
        }
        lines.push(format!("- {}: {}", entry.file, entry.title));
    }

    ToolResult::text(lines.join("\n"))
}

fn tool_get_context(repo: &Path, args: &Value) -> ToolResult {
    let languages: Vec<String> = args
        .get("languages")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    if languages.is_empty() {
        return ToolResult::error("Missing required parameter: languages");
    }

    let topics: Vec<String> = args
        .get("topics")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_lowercase()))
                .collect()
        })
        .unwrap_or_default();

    let reg = match get_registry(repo) {
        Ok(r) => r,
        Err(e) => return ToolResult::error(e),
    };

    let lang_set: std::collections::HashSet<String> =
        languages.iter().map(|l| l.to_lowercase()).collect();
    let topic_set: std::collections::HashSet<String> = topics.into_iter().collect();

    let mut matched: Vec<&crate::registry::RuleEntry> = Vec::new();
    for entry in reg.list(None) {
        let cat = entry.category.to_lowercase();
        if lang_set.contains(&cat) {
            matched.push(entry);
        } else if !topic_set.is_empty() {
            let concepts: std::collections::HashSet<String> =
                entry.concepts.iter().map(|c| c.to_lowercase()).collect();
            let tags: std::collections::HashSet<String> =
                entry.tags.iter().map(|t| t.to_lowercase()).collect();
            if !concepts.is_disjoint(&topic_set) || !tags.is_disjoint(&topic_set) {
                matched.push(entry);
            }
        }
    }

    if matched.is_empty() {
        return ToolResult::text("No rules found for the given languages/topics.");
    }

    let mut sections = Vec::new();
    for entry in &matched {
        let file_path = repo.join(&entry.file);
        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        sections.push(format!("## {}", entry.file));
        if !entry.rules.is_empty() {
            sections.push(format!("**RULES:** {}", entry.rules.join(" | ")));
        }
        if !entry.banned.is_empty() {
            sections.push(format!("**BANNED:** {}", entry.banned.join(" | ")));
        }
        sections.push(content);
        sections.push("---".into());
    }

    if sections.is_empty() {
        ToolResult::text("No rules found for the given languages/topics.")
    } else {
        ToolResult::text(sections.join("\n\n"))
    }
}

fn tool_learning_path(repo: &Path, args: &Value) -> ToolResult {
    let languages: Vec<String> = args
        .get("languages")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    if languages.is_empty() {
        return ToolResult::error("Missing required parameter: languages");
    }

    let phase = args.get("phase").and_then(|v| v.as_u64()).map(|p| p as u8);

    let reg = match get_registry(repo) {
        Ok(r) => r,
        Err(e) => return ToolResult::error(e),
    };

    let layers = reg.learning_path(&languages, phase);
    if layers.is_empty() {
        return ToolResult::text("No rules found for the given languages.");
    }

    let total: usize = layers.iter().map(|(_, entries)| entries.len()).sum();
    let total_phases = layers.len();

    let mut sections = vec![format!(
        "# Learning Path: {} — {total} rules in {total_phases} phases\n",
        languages.join(", ")
    )];

    for (layer_num, entries) in &layers {
        sections.push(format!("## Phase {layer_num}: {} rules", entries.len()));
        for entry in entries {
            let mut markers = Vec::new();
            if !entry.rules.is_empty() {
                markers.push(format!("RULES: {}", entry.rules.len()));
            }
            if !entry.banned.is_empty() {
                markers.push(format!("BANNED: {}", entry.banned.len()));
            }
            let marker_str = if markers.is_empty() {
                String::new()
            } else {
                format!(" [{}]", markers.join(", "))
            };
            sections.push(format!("- {}: {}{marker_str}", entry.file, entry.title));
        }
        sections.push(String::new());
    }

    ToolResult::text(sections.join("\n"))
}

fn tool_get_related(repo: &Path, args: &Value) -> ToolResult {
    let file = match args.get("file").and_then(|v| v.as_str()) {
        Some(f) => f,
        None => return ToolResult::error("Missing required parameter: file"),
    };

    let reg = match get_registry(repo) {
        Ok(r) => r,
        Err(e) => return ToolResult::error(e),
    };

    let entry = match reg.find_by_file(file) {
        Some(e) => e,
        None => return ToolResult::error(format!("File not found in registry: {file}")),
    };

    let edges = &entry.edges;
    let edge_types: &[(&str, &str, &Vec<String>)] = &[
        ("requires", "Depends on (must read first)", &edges.requires),
        ("required_by", "Depended on by", &edges.required_by),
        ("feeds", "Feeds into", &edges.feeds),
        ("fed_by", "Fed by", &edges.fed_by),
        ("related", "Related", &edges.related),
    ];

    let has_any = edge_types.iter().any(|(_, _, targets)| !targets.is_empty());
    if !has_any {
        return ToolResult::text(format!("No edges found for {file}"));
    }

    let mut lines = vec![format!("# Edges for {file}\n")];

    for &(_, label, targets) in edge_types {
        if targets.is_empty() {
            continue;
        }
        lines.push(format!("## {label}"));
        for target in targets {
            let title = reg
                .find_by_file(target)
                .map(|e| e.title.as_str())
                .unwrap_or("(not found)");
            lines.push(format!("- {target}: {title}"));
        }
        lines.push(String::new());
    }

    ToolResult::text(lines.join("\n"))
}
