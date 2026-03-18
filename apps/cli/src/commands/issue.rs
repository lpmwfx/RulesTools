const FORGEJO_API: &str = "https://git.lpmintra.com/api/v1/repos/lpmwfx/issues";

/// fn `forgejo_token`.
pub fn forgejo_token() -> Result<String, String> {
    std::env::var("FORGEJO_TOKEN")
        .map_err(|_| "FORGEJO_TOKEN environment variable not set".to_string())
}

/// fn `resolve_label_ids`.
pub fn resolve_label_ids(token: &str, names: &[&str]) -> Result<Vec<u64>, String> {
    let resp = ureq::get(&format!("{FORGEJO_API}/labels?limit=50"))
        .set("Authorization", &format!("token {token}"))
        .call()
        .map_err(|e| format!("Cannot fetch labels: {e}"))?;

    let body = resp.into_string().map_err(|e| format!("Cannot read response: {e}"))?;
    let all_labels: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap_or_default();

    let mut ids = Vec::new();
    for name in names {
        for label in &all_labels {
            if label.get("name").and_then(|v| v.as_str()) == Some(name) {
                if let Some(id) = label.get("id").and_then(|v| v.as_u64()) {
                    ids.push(id);
                }
            }
        }
    }
    Ok(ids)
}

// --- Internal functions (return Result for MCP reuse) ---

/// fn `report_internal`.
pub fn report_internal(title: &str, body: &str, labels_str: &str) -> Result<String, String> {
    let token = forgejo_token()?;

    let labels: Vec<&str> = labels_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    let label_ids = resolve_label_ids(&token, &labels).unwrap_or_default();

    let payload = serde_json::json!({
        "title": title,
        "body": body,
        "labels": label_ids,
    });

    let resp = ureq::post(&format!("{FORGEJO_API}/issues"))
        .set("Authorization", &format!("token {token}"))
        .set("Content-Type", "application/json")
        .send_string(&payload.to_string())
        .map_err(|e| format!("Failed to create issue: {e}"))?;

    let json = resp.into_string()
        .map_err(|e| format!("Cannot read response: {e}"))
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).map_err(|e| format!("Cannot parse response: {e}")))?;

    let number = json.get("number").and_then(|v| v.as_u64()).unwrap_or(0);
    let url = json.get("html_url").and_then(|v| v.as_str()).unwrap_or("?");
    Ok(format!("Issue #{number} created: {url}"))
}

/// fn `list_internal`.
pub fn list_internal(state: &str, labels_str: &str) -> Result<String, String> {
    let token = forgejo_token()?;

    let mut url = format!("{FORGEJO_API}/issues?state={state}&limit=50");
    if !labels_str.is_empty() {
        url.push_str(&format!("&labels={labels_str}"));
    }

    let resp = ureq::get(&url)
        .set("Authorization", &format!("token {token}"))
        .call()
        .map_err(|e| format!("Failed to list issues: {e}"))?;

    let body = resp.into_string().map_err(|e| format!("Cannot read response: {e}"))?;
    let issues: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap_or_default();

    if issues.is_empty() {
        return Ok("No issues found".into());
    }

    let mut out = String::new();
    for issue in &issues {
        let number = issue.get("number").and_then(|v| v.as_u64()).unwrap_or(0);
        let title = issue.get("title").and_then(|v| v.as_str()).unwrap_or("?");
        let state = issue.get("state").and_then(|v| v.as_str()).unwrap_or("?");
        let labels: Vec<&str> = issue.get("labels")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|l| l.get("name").and_then(|n| n.as_str())).collect())
            .unwrap_or_default();
        out.push_str(&format!("#{number} [{state}] {title}  {}\n", labels.join(", ")));
    }
    Ok(out)
}

/// Read a single issue by number — returns title, body, labels, state, comments.
pub fn read_internal(number: u64) -> Result<String, String> {
    let token = forgejo_token()?;

    let resp = ureq::get(&format!("{FORGEJO_API}/issues/{number}"))
        .set("Authorization", &format!("token {token}"))
        .call()
        .map_err(|e| format!("Failed to read issue: {e}"))?;

    let body = resp.into_string().map_err(|e| format!("Cannot read response: {e}"))?;
    let issue: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| format!("Cannot parse response: {e}"))?;

    let title = issue.get("title").and_then(|v| v.as_str()).unwrap_or("?");
    let state = issue.get("state").and_then(|v| v.as_str()).unwrap_or("?");
    let issue_body = issue.get("body").and_then(|v| v.as_str()).unwrap_or("");
    let labels: Vec<&str> = issue.get("labels")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|l| l.get("name").and_then(|n| n.as_str())).collect())
        .unwrap_or_default();
    let created = issue.get("created_at").and_then(|v| v.as_str()).unwrap_or("?");

    let mut out = format!("#{number} [{state}] {title}\nLabels: {}\nCreated: {created}\n\n{issue_body}", labels.join(", "));

    // Fetch comments
    let comments_resp = ureq::get(&format!("{FORGEJO_API}/issues/{number}/comments"))
        .set("Authorization", &format!("token {token}"))
        .call();

    if let Ok(resp) = comments_resp {
        if let Ok(body) = resp.into_string() {
            let comments: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap_or_default();
            if !comments.is_empty() {
                out.push_str(&format!("\n\n--- {} comment(s) ---\n", comments.len()));
                for comment in &comments {
                    let author = comment.get("user")
                        .and_then(|u| u.get("login"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    let body = comment.get("body").and_then(|v| v.as_str()).unwrap_or("");
                    let date = comment.get("created_at").and_then(|v| v.as_str()).unwrap_or("?");
                    out.push_str(&format!("\n[{author} @ {date}]\n{body}\n"));
                }
            }
        }
    }

    Ok(out)
}

/// fn `close_internal`.
pub fn close_internal(number: u64, comment: &str) -> Result<String, String> {
    let token = forgejo_token()?;

    if !comment.is_empty() {
        let payload = serde_json::json!({ "body": comment });
        let _ = ureq::post(&format!("{FORGEJO_API}/issues/{number}/comments"))
            .set("Authorization", &format!("token {token}"))
            .set("Content-Type", "application/json")
            .send_string(&payload.to_string());
    }

    let payload = serde_json::json!({ "state": "closed" });
    ureq::patch(&format!("{FORGEJO_API}/issues/{number}"))
        .set("Authorization", &format!("token {token}"))
        .set("Content-Type", "application/json")
        .send_string(&payload.to_string())
        .map_err(|e| format!("Failed to close issue: {e}"))?;

    Ok(format!("Issue #{number} closed"))
}

/// fn `add_label_internal`.
pub fn add_label_internal(number: u64, label: &str) -> Result<String, String> {
    let token = forgejo_token()?;

    let label_ids = resolve_label_ids(&token, &[label])?;
    if label_ids.is_empty() {
        return Err(format!("Label not found: {label}"));
    }

    let payload = serde_json::json!({ "labels": label_ids });
    ureq::post(&format!("{FORGEJO_API}/issues/{number}/labels"))
        .set("Authorization", &format!("token {token}"))
        .set("Content-Type", "application/json")
        .send_string(&payload.to_string())
        .map_err(|e| format!("Failed to add label: {e}"))?;

    Ok(format!("Label '{label}' added to issue #{number}"))
}

/// fn `create_label_internal`.
pub fn create_label_internal(name: &str, color: &str, description: &str) -> Result<String, String> {
    let token = forgejo_token()?;

    let payload = serde_json::json!({
        "name": name,
        "color": format!("#{color}"),
        "description": description,
    });

    let resp = ureq::post(&format!("{FORGEJO_API}/labels"))
        .set("Authorization", &format!("token {token}"))
        .set("Content-Type", "application/json")
        .send_string(&payload.to_string())
        .map_err(|e| format!("Failed to create label: {e}"))?;

    let json = resp.into_string()
        .map_err(|e| format!("Cannot read response: {e}"))
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).map_err(|e| format!("Cannot parse response: {e}")))?;

    let id = json.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
    Ok(format!("Label '{name}' created (id: {id})"))
}

/// fn `list_labels_internal`.
pub fn list_labels_internal() -> Result<String, String> {
    let token = forgejo_token()?;

    let resp = ureq::get(&format!("{FORGEJO_API}/labels?limit=50"))
        .set("Authorization", &format!("token {token}"))
        .call()
        .map_err(|e| format!("Failed to list labels: {e}"))?;

    let body = resp.into_string().map_err(|e| format!("Cannot read response: {e}"))?;
    let labels: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap_or_default();

    if labels.is_empty() {
        return Ok("No labels found".into());
    }

    let mut out = String::new();
    for label in &labels {
        let name = label.get("name").and_then(|v| v.as_str()).unwrap_or("?");
        let color = label.get("color").and_then(|v| v.as_str()).unwrap_or("");
        let desc = label.get("description").and_then(|v| v.as_str()).unwrap_or("");
        if desc.is_empty() {
            out.push_str(&format!("{name}  ({color})\n"));
        } else {
            out.push_str(&format!("{name}  ({color}) — {desc}\n"));
        }
    }
    Ok(out)
}

/// fn `comment_internal`.
pub fn comment_internal(number: u64, body: &str) -> Result<String, String> {
    let token = forgejo_token()?;

    let payload = serde_json::json!({ "body": body });
    ureq::post(&format!("{FORGEJO_API}/issues/{number}/comments"))
        .set("Authorization", &format!("token {token}"))
        .set("Content-Type", "application/json")
        .send_string(&payload.to_string())
        .map_err(|e| format!("Failed to add comment: {e}"))?;

    Ok(format!("Comment added to issue #{number}"))
}

// --- CLI wrappers ---

/// fn `cmd_issue_report`.
pub fn cmd_issue_report(title: &str, body: &str, labels: &str) {
    match report_internal(title, body, labels) {
        Ok(output) => println!("{output}"),
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    }
}

/// fn `cmd_issue_list`.
pub fn cmd_issue_list(state: &str, labels: &str) {
    match list_internal(state, labels) {
        Ok(output) => print!("{output}"),
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    }
}

/// fn `cmd_issue_close`.
pub fn cmd_issue_close(number: u64, comment: &str) {
    match close_internal(number, comment) {
        Ok(output) => println!("{output}"),
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    }
}

/// fn `cmd_issue_add_label`.
pub fn cmd_issue_add_label(number: u64, label: &str) {
    match add_label_internal(number, label) {
        Ok(output) => println!("{output}"),
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    }
}

/// fn `cmd_issue_create_label`.
pub fn cmd_issue_create_label(name: &str, color: &str, description: &str) {
    match create_label_internal(name, color, description) {
        Ok(output) => println!("{output}"),
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    }
}

/// fn `cmd_issue_list_labels`.
pub fn cmd_issue_list_labels() {
    match list_labels_internal() {
        Ok(output) => print!("{output}"),
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    }
}

/// fn `cmd_issue_comment`.
pub fn cmd_issue_comment(number: u64, body: &str) {
    match comment_internal(number, body) {
        Ok(output) => println!("{output}"),
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    }
}

// --- IssueCmd dispatch ---
use crate::IssueCmd;

/// fn `cmd_issue`.
pub fn cmd_issue(cmd: IssueCmd) {
    match cmd {
        IssueCmd::Report { title, body, labels } => cmd_issue_report(&title, &body, &labels),
        IssueCmd::List { state, labels } => cmd_issue_list(&state, &labels),
        IssueCmd::Close { number, comment } => cmd_issue_close(number, &comment),
        IssueCmd::AddLabel { number, label } => cmd_issue_add_label(number, &label),
        IssueCmd::CreateLabel { name, color, description } => cmd_issue_create_label(&name, &color, &description),
        IssueCmd::ListLabels => cmd_issue_list_labels(),
        IssueCmd::Comment { number, body } => cmd_issue_comment(number, &body),
    }
}
