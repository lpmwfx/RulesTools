use serde_json::Value;
use crate::mcp::ToolResult;
use crate::commands::scan as scan_cmd;
use super::get_path;

/// fn `scan_file`.
pub fn scan_file(args: &Value) -> ToolResult {
    let path = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    if !path.exists() {
        return ToolResult::error(format!("File not found: {}", path.display()));
    }

    match scan_cmd::scan_file_internal(&path, "text") {
        Ok(mut output) => {
            // If not CLEAN, append rule hints
            if !output.contains("CLEAN") {
                let hints = collect_rule_hints(&output);
                if !hints.is_empty() {
                    output.push_str("\n\n### Relevant rules — call get_rule() for details\n");
                    for hint in hints {
                        output.push_str(&format!("- get_rule(\"{}\")\n", hint));
                    }
                }
            }
            ToolResult::text(output)
        }
        Err(e) => ToolResult::error(e),
    }
}

/// Collect relevant rule files based on violation keywords.
fn collect_rule_hints(output: &str) -> Vec<String> {
    let mut hints = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let lower = output.to_lowercase();

    // Map violation keywords to rule files
    let patterns = vec![
        ("mother", "global/mother-tree.md"),
        ("delegate", "global/mother-tree.md"),
        ("fn definitions", "global/mother-tree.md"),
        ("magic number", "global/config-driven.md"),
        ("literal", "global/config-driven.md"),
        ("zero", "global/config-driven.md"),
        ("unwrap", "rust/errors.md"),
        ("expect", "rust/errors.md"),
        ("panic", "rust/errors.md"),
        ("too long", "global/file-limits.md"),
        ("oversized", "global/file-limits.md"),
        ("lines", "global/file-limits.md"),
        ("stringly", "rust/types.md"),
        ("string match", "rust/types.md"),
        ("clone", "rust/ownership.md"),
        ("layer", "global/topology.md"),
        ("topology", "global/topology.md"),
        ("placement", "global/topology.md"),
        ("unsafe", "rust/threading.md"),
        ("arc", "rust/threading.md"),
        ("rc", "rust/threading.md"),
    ];

    for (keyword, rule) in patterns {
        if lower.contains(keyword) && !seen.contains(rule) {
            hints.push(rule.to_string());
            seen.insert(rule);
        }
    }

    hints
}

/// fn `scan_tree`.
pub fn scan_tree(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    match scan_cmd::scan_internal(&root, false) {
        Ok(output) => ToolResult::text(output),
        Err(output) => ToolResult::text(output), // Still return output on deny failure
    }
}

/// fn `check_staged`.
pub fn check_staged(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    match scan_cmd::check_internal(&root) {
        Ok(output) => ToolResult::text(output),
        Err(output) => ToolResult::error(output),
    }
}

/// fn `security_scan`.
pub fn security_scan(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    // Run full scan, then filter for security-relevant lines
    let output = match scan_cmd::scan_internal(&root, false) {
        Ok(o) => o,
        Err(o) => o,
    };

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
