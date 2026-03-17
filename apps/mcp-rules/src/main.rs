mod protocol;
mod registry;
mod rules;

/// MCP server for AI rule lookup.
///
/// Replaces the Python rules-mcp package.
/// Communicates via JSON-RPC over stdio (MCP protocol).
fn main() {
    let tool_defs = rules::definitions();
    protocol::run_server("rules", tool_defs, rules::handle);
}
