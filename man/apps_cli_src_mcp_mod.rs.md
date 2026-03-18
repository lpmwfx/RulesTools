# apps/cli/src/mcp/mod.rs

## `pub mod tools;`

*Line 2 · mod*

mod `tools`.

---

## `pub mod rules;`

*Line 4 · mod*

mod `rules`.

---

## `pub struct Request`

*Line 12 · struct*

JSON-RPC request from MCP client.

---

## `pub struct Response`

*Line 23 · struct*

JSON-RPC response to MCP client.

---

## `pub struct RpcError`

*Line 35 · struct*

struct `RpcError`.

---

## `pub struct ToolDef`

*Line 42 · struct*

MCP tool definition for tools/list.

---

## `pub struct ToolResult`

*Line 51 · struct*

MCP tool call result.

---

## `pub struct ContentBlock`

*Line 59 · struct*

struct `ContentBlock`.

---

## `pub fn success(id: Option<Value>, result: Value) -> Self`

*Line 67 · fn*

fn `success`.

---

## `pub fn error(id: Option<Value>, code: i32, message: impl Into<String>) -> Self`

*Line 72 · fn*

fn `error`.

---

## `pub fn text(msg: impl Into<String>) -> Self`

*Line 84 · fn*

fn `text`.

---

## `pub fn error(msg: impl Into<String>) -> Self`

*Line 92 · fn*

fn `error`.

---

## `pub fn run_server( server_name: &str, tools: Vec<ToolDef>, handler: impl Fn(&str, &Value) -> ToolResult, )`

*Line 101 · fn*

Run the MCP server stdio loop.

---

