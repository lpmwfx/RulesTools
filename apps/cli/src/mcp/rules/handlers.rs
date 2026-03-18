use std::path::Path;
use serde_json::Value;
use crate::mcp::ToolResult;
use super::registry::{get_registry, RuleEntry};

/// fn `help`.
pub fn help(repo: &Path) -> ToolResult {
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

/// fn `get_rule`.
pub fn get_rule(repo: &Path, args: &Value) -> ToolResult {
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

/// fn `search_rules`.
pub fn search_rules(repo: &Path, args: &Value) -> ToolResult {
    let query = match args.get("query").and_then(|v| v.as_str()) {
        Some(q) => q,
        None => return ToolResult::error("Missing required parameter: query"),
    };
    let category = args.get("category").and_then(|v| v.as_str());
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

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

/// fn `list_rules`.
pub fn list_rules(repo: &Path, args: &Value) -> ToolResult {
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

/// fn `get_context`.
pub fn get_context(repo: &Path, args: &Value) -> ToolResult {
    let languages: Vec<String> = args
        .get("languages")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    if languages.is_empty() {
        return ToolResult::error("Missing required parameter: languages");
    }

    let topics: Vec<String> = args
        .get("topics")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_lowercase())).collect())
        .unwrap_or_default();

    let quick_ref = args
        .get("quick_ref")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let reg = match get_registry(repo) {
        Ok(r) => r,
        Err(e) => return ToolResult::error(e),
    };

    let lang_set: std::collections::HashSet<String> =
        languages.iter().map(|l| l.to_lowercase()).collect();
    let topic_set: std::collections::HashSet<String> = topics.into_iter().collect();

    let mut matched: Vec<&RuleEntry> = Vec::new();
    for entry in reg.list(None) {
        let cat = entry.category.to_lowercase();
        let is_quick_ref = entry.file.ends_with("quick-ref.md");

        if quick_ref {
            // Quick-ref mode: only quick-ref files from requested languages + global
            if is_quick_ref && (lang_set.contains(&cat) || cat == "global") {
                matched.push(entry);
            }
        } else if topic_set.is_empty() {
            // No topics: include all files from requested languages
            if lang_set.contains(&cat) {
                matched.push(entry);
            }
        } else {
            // Topics specified: filter by topic match within scope
            let concepts: std::collections::HashSet<String> =
                entry.concepts.iter().map(|c| c.to_lowercase()).collect();
            let tags: std::collections::HashSet<String> =
                entry.tags.iter().map(|t| t.to_lowercase()).collect();
            let has_topic =
                !concepts.is_disjoint(&topic_set) || !tags.is_disjoint(&topic_set);

            if lang_set.contains(&cat) && has_topic {
                // Language file matching topic
                matched.push(entry);
            } else if cat == "global" && has_topic {
                // Global foundation matching topic
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

/// fn `learning_path`.
pub fn learning_path(repo: &Path, args: &Value) -> ToolResult {
    let languages: Vec<String> = args
        .get("languages")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
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

/// fn `get_related`.
pub fn get_related(repo: &Path, args: &Value) -> ToolResult {
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
