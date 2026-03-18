mod definitions;
mod scan;
mod project;
mod issue;
mod publish;
mod generate;

use serde_json::Value;
use crate::mcp::{self, ToolResult};
use std::path::PathBuf;

/// Run the MCP tools server (stdio loop).
pub fn run() {
    let tool_defs = definitions::all();
    mcp::run_server("rulestools", tool_defs, handle);
}

fn handle(name: &str, args: &Value) -> ToolResult {
    match name {
        "scan_file" => scan::scan_file(args),
        "scan_tree" => scan::scan_tree(args),
        "check_staged" => scan::check_staged(args),
        "security_scan" => scan::security_scan(args),
        "setup" => project::setup(args),
        "init_project" => project::init_project(args),
        "new_project" => project::new_project(args),
        "update_project" => project::update_project(args),
        "upgrade_project" => project::upgrade_project(args),
        "report_issue" => issue::report_issue(args),
        "list_issues" => issue::list_issues(args),
        "add_label" => issue::add_label(args),
        "list_labels" => issue::list_labels(args),
        "comment_issue" => issue::comment_issue(args),
        "create_label" => issue::create_label(args),
        "close_issue" => issue::close_issue(args),
        "publish_plan" => publish::plan(args),
        "publish_run" => publish::run(args),
        "publish_status" => publish::status(args),
        "publish_init" => publish::init(args),
        "publish_sync" => publish::sync(args),
        "generate_docs" => generate::generate_docs(args),
        "publish_check" => publish::check(args),
        _ => ToolResult::error(format!("Unknown tool: {name}")),
    }
}

/// fn `get_path`.
pub fn get_path(args: &Value) -> Result<PathBuf, ToolResult> {
    args.get("path")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .ok_or_else(|| ToolResult::error("Missing required parameter: path"))
}
