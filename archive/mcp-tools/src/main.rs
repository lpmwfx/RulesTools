mod protocol;
mod tools;

/// MCP server for scan, setup, init, security scan.
///
/// Replaces the Python rulestools-mcp package.
/// Communicates via JSON-RPC over stdio (MCP protocol).
fn main() {
    let tool_defs = tools::definitions();
    protocol::run_server(tool_defs, tools::handle);
}
