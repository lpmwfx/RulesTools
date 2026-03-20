# archive/mcp-rules/src/protocol.rs

## `pub struct Request`

*Line 7 · struct*

struct `Request`.

---

## `pub struct Response`

*Line 18 · struct*

struct `Response`.

---

## `pub struct RpcError`

*Line 30 · struct*

struct `RpcError`.

---

## `pub struct ToolDef`

*Line 37 · struct*

struct `ToolDef`.

---

## `pub struct ToolResult`

*Line 46 · struct*

struct `ToolResult`.

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

*Line 74 · fn*

fn `text`.

---

## `pub fn error(msg: impl Into<String>) -> Self`

*Line 82 · fn*

fn `error`.

---

## `pub fn run_server(server_name: &str, tools: Vec<ToolDef>, handler: impl Fn(&str, &Value) -> ToolResult)`

*Line 91 · fn*

fn `run_server`.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
