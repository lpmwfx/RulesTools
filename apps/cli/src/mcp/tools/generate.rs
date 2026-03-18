use serde_json::Value;
use crate::mcp::ToolResult;
use crate::commands::generate as gen_cmd;
use super::get_path;

/// fn `generate_docs`.
pub fn generate_docs(args: &Value) -> ToolResult {
    let path = match get_path(args) {
        Ok(p) => p,
        Err(e) => return e,
    };

    let root = std::fs::canonicalize(&path).unwrap_or(path);
    let output = gen_cmd::gen_internal(&root);
    ToolResult::text(output)
}
