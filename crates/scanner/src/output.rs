use std::collections::HashSet;
use std::path::Path;

use crate::issue::Issue;

/// Emit issues as `cargo:warning` lines (for build.rs integration).
pub fn emit_cargo_warnings(issues: &[Issue], base: &Path) {
    for issue in issues {
        let rel = issue.relative_path(base);
        println!(
            "cargo:warning={}:{}:{}: {} {}: {}",
            rel.display(),
            issue.line,
            issue.col,
            issue.severity,
            issue.rule_id,
            issue.message,
        );
    }
}

/// Write issues to `proj/ISSUES` with [NEW]/[KNOWN] delta markers.
///
/// Returns the number of new issues found.
pub fn write_issues_file(issues: &[Issue], project_root: &Path) -> std::io::Result<usize> {
    let issues_path = project_root.join("proj").join("ISSUES");

    // Read previous issues for delta detection
    let previous_keys: HashSet<String> = if issues_path.exists() {
        std::fs::read_to_string(&issues_path)
            .unwrap_or_default()
            .lines()
            .filter(|line| !line.starts_with('#') && !line.is_empty())
            .map(|line| {
                // Strip [NEW]/[KNOWN] prefix if present
                let stripped = line
                    .trim_start_matches("[NEW] ")
                    .trim_start_matches("[KNOWN] ");
                stripped.to_string()
            })
            .collect()
    } else {
        HashSet::new()
    };

    let mut new_count = 0;
    let mut output = String::new();

    if issues.is_empty() {
        output.push_str("# No issues found\n");
    } else {
        output.push_str(&format!("# {} issues\n\n", issues.len()));

        for issue in issues {
            let line_str = issue.display_line();
            let is_known = previous_keys.contains(&line_str);
            let prefix = if is_known { "[KNOWN]" } else { "[NEW]" };
            if !is_known {
                new_count += 1;
            }
            output.push_str(&format!("{prefix} {line_str}\n"));
        }
    }

    // Ensure proj/ exists
    let proj_dir = project_root.join("proj");
    if !proj_dir.exists() {
        std::fs::create_dir_all(&proj_dir)?;
    }

    std::fs::write(&issues_path, output)?;
    Ok(new_count)
}

/// Check if build should be denied (any Error-severity issues and deny=true).
pub fn should_deny(issues: &[Issue], deny: bool) -> bool {
    if !deny {
        return false;
    }
    issues.iter().any(|i| i.severity == crate::issue::Severity::Error)
}
