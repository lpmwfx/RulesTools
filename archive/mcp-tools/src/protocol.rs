use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};

/// JSON-RPC request from MCP client.
#[derive(Deserialize)]
pub struct Request {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// JSON-RPC response to MCP client.
#[derive(Serialize)]
pub struct Response {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

#[derive(Serialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

/// MCP tool definition for tools/list.
#[derive(Serialize, Clone)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// MCP tool call result.
#[derive(Serialize)]
pub struct ToolResult {
    pub content: Vec<ContentBlock>,
    #[serde(rename = "isError", skip_serializing_if = "std::ops::Not::not")]
    pub is_error: bool,
}

#[derive(Serialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

impl Response {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self { jsonrpc: "2.0".into(), id, result: Some(result), error: None }
    }

    pub fn error(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(RpcError { code, message: message.into() }),
        }
    }
}

impl ToolResult {
    pub fn text(msg: impl Into<String>) -> Self {
        Self {
            content: vec![ContentBlock { content_type: "text".into(), text: msg.into() }],
            is_error: false,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            content: vec![ContentBlock { content_type: "text".into(), text: msg.into() }],
            is_error: true,
        }
    }
}

/// Run the MCP server stdio loop.
pub fn run_server(tools: Vec<ToolDef>, handler: impl Fn(&str, &Value) -> ToolResult) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let req: Request = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = Response::error(None, -32700, format!("Parse error: {e}"));
                write_response(&mut stdout_lock, &resp);
                continue;
            }
        };

        let resp = match req.method.as_str() {
            "initialize" => {
                let result = serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "rulestools",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                });
                Response::success(req.id, result)
            }

            "notifications/initialized" => continue,

            "tools/list" => {
                let result = serde_json::json!({ "tools": tools });
                Response::success(req.id, result)
            }

            "tools/call" => {
                let tool_name = req.params.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let arguments = req.params.get("arguments")
                    .cloned()
                    .unwrap_or(Value::Object(Default::default()));

                let tool_result = handler(tool_name, &arguments);
                let result = serde_json::to_value(&tool_result).unwrap_or_default();
                Response::success(req.id, result)
            }

            _ => Response::error(req.id, -32601, format!("Unknown method: {}", req.method)),
        };

        write_response(&mut stdout_lock, &resp);
    }
}

fn write_response(writer: &mut impl Write, resp: &Response) {
    if let Ok(json) = serde_json::to_string(resp) {
        let _ = writeln!(writer, "{json}");
        let _ = writer.flush();
    }
}
