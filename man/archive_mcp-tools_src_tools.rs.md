# archive/mcp-tools/src/tools.rs

## `pub fn definitions() -> Vec<ToolDef>`

*Line 7 · fn*

Register all tool definitions.

---

## `pub fn handle(name: &str, args: &Value) -> ToolResult`

*Line 345 · fn*

Dispatch tool calls — all delegate to rulestools CLI via subprocess.

---

