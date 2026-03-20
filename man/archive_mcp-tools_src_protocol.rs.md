# archive/mcp-tools/src/protocol.rs

## `pub struct Request`

*Line 7 · struct*

JSON-RPC request from MCP client.

---

## `pub struct Response`

*Line 18 · struct*

JSON-RPC response to MCP client.

---

## `pub struct RpcError`

*Line 30 · struct*

struct `RpcError`.

---

## `pub struct ToolDef`

*Line 37 · struct*

MCP tool definition for tools/list.

---

## `pub struct ToolResult`

*Line 46 · struct*

MCP tool call result.

---

## `pub struct ContentBlock`

*Line 54 · struct*

struct `ContentBlock`.

---

## `pub fn success(id: Option<Value>, result: Value) -> Self`

*Line 62 · fn*

fn `success`.

---

## `pub fn error(id: Option<Value>, code: i32, message: impl Into<String>) -> Self`

*Line 67 · fn*

fn `error`.

---

## `pub fn text(msg: impl Into<String>) -> Self`

*Line 79 · fn*

fn `text`.

---

## `pub fn error(msg: impl Into<String>) -> Self`

*Line 87 · fn*

fn `error`.

---

## `pub fn run_server(tools: Vec<ToolDef>, handler: impl Fn(&str, &Value) -> ToolResult)`

*Line 96 · fn*

Run the MCP server stdio loop.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
