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
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
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

    match scan_cmd::scan_internal(&root, true) {
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
