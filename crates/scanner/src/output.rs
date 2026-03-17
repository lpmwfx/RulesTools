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

/// Error group with guidance text.
pub struct IssueGroup {
    pub name: &'static str,
    pub guidance: &'static str,
    pub reference: &'static str,
    pub issues: Vec<Issue>,
}

/// Group classification for a rule ID.
fn classify_rule(rule_id: &str) -> &'static str {
    match rule_id {
        id if id.starts_with("topology/") => "TOPOLOGY",
        id if id.contains("layer-violation") => "TOPOLOGY",
        id if id.contains("sibling-import") => "TOPOLOGY",
        id if id.contains("shared-guard") => "PURITY",
        id if id.contains("single-gateway") => "PURITY",
        id if id.contains("mother") || id.contains("shared-candidate") => "MOTHER-CHILD",
        id if id.contains("child-has-state") => "MOTHER-CHILD",
        id if id.contains("magic-number") || id.contains("hardcoded") => "LITERALS",
        id if id.contains("unwrap") || id.contains("panic") || id.contains("expect")
            || id.contains("unsafe") => "SAFETY",
        id if id.contains("secrets") => "SAFETY",
        _ => "HYGIENE",
    }
}

/// Guidance text per group.
fn group_guidance(name: &str) -> (&'static str, &'static str) {
    match name {
        "TOPOLOGY" => (
            "Wrong layer. _ui->_adp only, _core->_pal only. Route through adapter.",
            "global/topology.md",
        ),
        "PURITY" => (
            "Zero-dependency contract. shared/ has no internal imports. One gateway per UI.",
            "global/stereotypes.md",
        ),
        "MOTHER-CHILD" => (
            "Mother delegates, children are stateless. Extract shared logic first, then split.",
            "global/mother-tree.md",
        ),
        "LITERALS" => (
            "Extract to named const/config/token. Name by PURPOSE not value.",
            "global/config-driven.md",
        ),
        "SAFETY" => (
            "Crash-free code. Use ? or unwrap_or_default(). // SAFETY: on unsafe.",
            "rust/errors.md",
        ),
        "HYGIENE" => (
            "Readability. Small files, flat logic, clear names.",
            "global/file-limits.md",
        ),
        _ => ("", ""),
    }
}

/// Group order for consistent output.
const GROUP_ORDER: &[&str] = &[
    "TOPOLOGY", "PURITY", "MOTHER-CHILD", "LITERALS", "SAFETY", "HYGIENE",
];

/// Format issues as grouped output with guidance per group.
///
/// If `rules_root` is provided, loads decision trees from Rules/guidance/*.toml.
pub fn format_grouped(issues: &[Issue], base: &Path) -> String {
    format_grouped_with_guidance(issues, base, None)
}

/// Format with optional guidance trees from Rules/guidance/.
pub fn format_grouped_with_guidance(issues: &[Issue], base: &Path, rules_root: Option<&Path>) -> String {
    use std::collections::BTreeMap;

    // Classify issues into groups
    let mut groups: BTreeMap<&str, Vec<&Issue>> = BTreeMap::new();
    for issue in issues {
        let group = classify_rule(&issue.rule_id);
        groups.entry(group).or_default().push(issue);
    }

    let mut output = String::new();

    for &group_name in GROUP_ORDER {
        let group_issues = match groups.get(group_name) {
            Some(issues) if !issues.is_empty() => issues,
            _ => continue,
        };

        let error_count = group_issues
            .iter()
            .filter(|i| matches!(i.severity, crate::issue::Severity::Critical | crate::issue::Severity::Error))
            .count();
        let warn_count = group_issues.len() - error_count;

        let (guidance, reference) = group_guidance(group_name);

        // Group header
        let counts = if warn_count > 0 && error_count > 0 {
            format!("{error_count} errors, {warn_count} warnings")
        } else if error_count > 0 {
            format!("{error_count} errors")
        } else {
            format!("{warn_count} warnings")
        };
        output.push_str(&format!("\n=== {group_name} ({counts}) ===\n"));
        output.push_str(&format!("{guidance}\n"));
        output.push_str(&format!("See: {reference}\n"));

        // Load decision tree if available
        if let Some(root) = rules_root {
            let tree_file = group_name.to_lowercase().replace("-", "-");
            let tree_path = root.join("guidance").join(format!("{tree_file}.toml"));
            if let Some(tree_text) = load_decision_tree(&tree_path) {
                output.push_str(&tree_text);
            }
        }
        output.push('\n');

        // Issues in group
        for issue in group_issues {
            let rel = issue.relative_path(base);
            output.push_str(&format!(
                "  {} {}:{}:{} — {}\n",
                issue.severity.label().to_uppercase(),
                rel.display(),
                issue.line,
                issue.col,
                issue.message,
            ));
        }
    }

    output
}

/// Load a decision tree from a TOML file and format as text.
fn load_decision_tree(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let table: toml::Table = content.parse().ok()?;
    let nodes = table.get("node")?.as_array()?;

    let mut output = String::new();
    for node in nodes {
        let node = node.as_table()?;
        if let Some(question) = node.get("question").and_then(|v| v.as_str()) {
            output.push_str(&format!("\n  ? {question}\n"));
        }
        if let Some(branches) = node.get("branch").and_then(|v| v.as_array()) {
            for branch in branches {
                let branch = branch.as_table()?;
                let condition = branch.get("condition").and_then(|v| v.as_str()).unwrap_or("?");
                let action = branch.get("action").and_then(|v| v.as_str()).unwrap_or("");
                output.push_str(&format!("    -> {condition}: {action}\n"));
            }
        }
    }
    Some(output)
}

/// Check if build should be denied.
/// Critical issues always deny. Error issues deny when deny=true.
pub fn should_deny(issues: &[Issue], deny: bool) -> bool {
    // Critical always blocks
    if issues.iter().any(|i| i.severity == crate::issue::Severity::Critical) {
        return true;
    }
    if !deny {
        return false;
    }
    issues.iter().any(|i| i.severity == crate::issue::Severity::Error)
}
