/// PostToolUse hook — scan file after Edit/Write (reads JSON from stdin).
///
/// Called by Claude Code `.claude/settings.json` PostToolUse hook.
/// Reads tool invocation JSON from stdin, extracts file_path,
/// scans the file, and prints violations to stderr (advisory).
/// Always exits 0 — never blocks edits.
pub fn cmd_hook() {
    let input = match std::io::read_to_string(std::io::stdin()) {
        Ok(s) => s,
        Err(_) => return,
    };

    let json: serde_json::Value = match serde_json::from_str(&input) {
        Ok(v) => v,
        Err(_) => return,
    };

    let file_path = json
        .get("tool_input")
        .and_then(|v| v.get("file_path"))
        .and_then(|v| v.as_str());

    let file_path = match file_path {
        Some(p) => p,
        None => return,
    };

    // Only scan supported file types
    let supported = [".rs", ".slint", ".py", ".js", ".ts", ".css"];
    if !supported.iter().any(|ext| file_path.ends_with(ext)) {
        return;
    }

    let path = std::path::Path::new(file_path);
    if !path.exists() {
        return;
    }

    match super::scan::scan_file_internal(path, "text") {
        Ok(output) => {
            if !output.starts_with("CLEAN") && !output.starts_with("SKIP") {
                eprintln!("[rulestools] {file_path}:\n{output}");
            }
        }
        Err(_) => {}
    }
}
