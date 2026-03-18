use serde_json::Value;
use crate::mcp::ToolResult;
use crate::publish as publish_mod;
use super::get_path;

/// fn `plan`.
pub fn plan(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let root = std::fs::canonicalize(&root).unwrap_or(root);
    match publish_mod::publish_plan(&root, "json") {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

/// fn `run`.
pub fn run(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let target = match args.get("target").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return ToolResult::error("Missing required parameter: target"),
    };
    let preview = args.get("preview").and_then(|v| v.as_bool()).unwrap_or(false);
    let root = std::fs::canonicalize(&root).unwrap_or(root);
    match publish_mod::publish_run(&root, target, preview) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

/// fn `status`.
pub fn status(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let root = std::fs::canonicalize(&root).unwrap_or(root);
    match publish_mod::publish_status(&root, "json") {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

/// fn `init`.
pub fn init(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let remote = match args.get("remote").and_then(|v| v.as_str()) {
        Some(r) => r,
        None => return ToolResult::error("Missing required parameter: remote"),
    };
    let root = std::fs::canonicalize(&root).unwrap_or(root);
    match publish_mod::publish_init(&root, remote) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

/// fn `sync`.
pub fn sync(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let preview = args.get("preview").and_then(|v| v.as_bool()).unwrap_or(false);
    let root = std::fs::canonicalize(&root).unwrap_or(root);
    match publish_mod::publish_sync(&root, preview) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

/// fn `check`.
pub fn check(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let root = std::fs::canonicalize(&root).unwrap_or(root);
    match publish_mod::publish_check(&root) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}
