use serde_json::Value;
use crate::mcp::ToolResult;
use crate::commands::project as project_cmd;
use super::get_path;

/// fn `setup`.
pub fn setup(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    match project_cmd::setup_internal(&root) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

/// fn `init_project`.
pub fn init_project(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    let kind = match args.get("kind").and_then(|v| v.as_str()) {
        Some(k) => k,
        None => return ToolResult::error("Missing required parameter: kind"),
    };

    let name = args.get("name").and_then(|v| v.as_str());

    match project_cmd::init_internal(&root, kind, name) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

/// fn `new_project`.
pub fn new_project(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    let kind = match args.get("kind").and_then(|v| v.as_str()) {
        Some(k) => k,
        None => return ToolResult::error("Missing required parameter: kind"),
    };

    let name = args.get("name").and_then(|v| v.as_str());

    let platforms_str = args.get("platforms")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(","))
        .unwrap_or_default();

    let themes_str = args.get("themes")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(","))
        .unwrap_or_default();

    let mcp = args.get("mcp").and_then(|v| v.as_bool()).unwrap_or(false);

    let extras_str = args.get("extras")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(","))
        .unwrap_or_default();

    let preview = args.get("preview").and_then(|v| v.as_bool()).unwrap_or(false);

    match project_cmd::new_internal(&root, kind, name, &platforms_str, &themes_str, mcp, &extras_str, preview, "json") {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

/// fn `update_project`.
pub fn update_project(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    let platforms_str = args.get("platforms")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(","))
        .unwrap_or_default();

    let themes_str = args.get("themes")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(","))
        .unwrap_or_default();

    let crate_name = args.get("crate_name").and_then(|v| v.as_str());

    let folders_str = args.get("folders")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(","))
        .unwrap_or_default();

    let preview = args.get("preview").and_then(|v| v.as_bool()).unwrap_or(false);

    match project_cmd::update_internal(&root, &platforms_str, &themes_str, crate_name, &folders_str, preview, "json") {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

/// fn `upgrade_project`.
pub fn upgrade_project(args: &Value) -> ToolResult {
    let root = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    let to = match args.get("to").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return ToolResult::error("Missing required parameter: to"),
    };

    let preview = args.get("preview").and_then(|v| v.as_bool()).unwrap_or(false);

    match project_cmd::upgrade_internal(&root, to, preview, "json") {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}
