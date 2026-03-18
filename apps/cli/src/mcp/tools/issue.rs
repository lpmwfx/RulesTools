use serde_json::Value;
use crate::mcp::ToolResult;
use crate::commands::issue as issue_cmd;

pub fn report_issue(args: &Value) -> ToolResult {
    let title = match args.get("title").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return ToolResult::error("Missing required parameter: title"),
    };
    let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
    let user_labels = args.get("labels").and_then(|v| v.as_str()).unwrap_or("");

    // Always include ai-reported
    let labels = if user_labels.is_empty() {
        "ai-reported".to_string()
    } else if user_labels.contains("ai-reported") {
        user_labels.to_string()
    } else {
        format!("ai-reported,{user_labels}")
    };

    match issue_cmd::report_internal(title, body, &labels) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

pub fn list_issues(args: &Value) -> ToolResult {
    let state = args.get("state").and_then(|v| v.as_str()).unwrap_or("open");
    let labels = args.get("labels").and_then(|v| v.as_str()).unwrap_or("");

    match issue_cmd::list_internal(state, labels) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

pub fn add_label(args: &Value) -> ToolResult {
    let number = match args.get("number").and_then(|v| v.as_u64()) {
        Some(n) => n,
        None => return ToolResult::error("Missing required parameter: number"),
    };
    let label = match args.get("label").and_then(|v| v.as_str()) {
        Some(l) => l,
        None => return ToolResult::error("Missing required parameter: label"),
    };

    match issue_cmd::add_label_internal(number, label) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

pub fn list_labels(_args: &Value) -> ToolResult {
    match issue_cmd::list_labels_internal() {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

pub fn comment_issue(args: &Value) -> ToolResult {
    let number = match args.get("number").and_then(|v| v.as_u64()) {
        Some(n) => n,
        None => return ToolResult::error("Missing required parameter: number"),
    };
    let body = match args.get("body").and_then(|v| v.as_str()) {
        Some(b) => b,
        None => return ToolResult::error("Missing required parameter: body"),
    };

    match issue_cmd::comment_internal(number, body) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

pub fn create_label(args: &Value) -> ToolResult {
    let name = match args.get("name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return ToolResult::error("Missing required parameter: name"),
    };
    let color = args.get("color").and_then(|v| v.as_str()).unwrap_or("0075ca");
    let desc = args.get("description").and_then(|v| v.as_str()).unwrap_or("");

    match issue_cmd::create_label_internal(name, color, desc) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}

pub fn close_issue(args: &Value) -> ToolResult {
    let number = match args.get("number").and_then(|v| v.as_u64()) {
        Some(n) => n,
        None => return ToolResult::error("Missing required parameter: number"),
    };
    let comment = args.get("comment").and_then(|v| v.as_str()).unwrap_or("");

    match issue_cmd::close_internal(number, comment) {
        Ok(output) => ToolResult::text(output),
        Err(e) => ToolResult::error(e),
    }
}
