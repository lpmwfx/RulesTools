use std::collections::HashSet;
use std::path::PathBuf;

use crate::config::Config;
use crate::issue::{Issue, Severity};

/// BANNED folder/crate names → canonical replacement.
const BANNED_NAMES: &[(&str, &str)] = &[
    ("infra", "gateway"),
    ("infrastructure", "gateway"),
    ("helpers", "shared"),
    ("utils", "shared"),
    ("common", "shared"),
    ("models", "core"),
    ("services", "core"),
    ("api", "adapter"),
    ("packages", "lib"),
    ("modules", "lib"),
];

/// Legal topology folder names at Level 1 (crates/ or src/ top-level).
const LEGAL_TOPOLOGY_NAMES: &[&str] = &[
    "core", "adapter", "adp", "gateway", "gtw", "pal", "ui", "app", "mcp", "shared", "lib",
];

/// Check Level 1 topology naming: folder/crate names at topology boundary.
///
/// 1. BANNED names → error with suggestion
/// 2. Unknown topology folders → error (only registered names allowed)
pub fn check(
    paths: &[PathBuf],
    cfg: &Config,
    issues: &mut Vec<Issue>,
) {
    let mut checked_dirs: HashSet<String> = HashSet::new();

    // Read allowed crates from config [topology].crates
    let extra_allowed = cfg.param_str("topology/crates", "");

    for path in paths {
        let normalized = path.to_string_lossy().replace('\\', "/");

        // Check src/<name>/ topology folders
        check_topology_segment(&normalized, "src/", &extra_allowed, &mut checked_dirs, path, issues);
        // Check crates/<name>/ topology folders
        check_topology_segment(&normalized, "crates/", &extra_allowed, &mut checked_dirs, path, issues);
    }
}

fn check_topology_segment(
    normalized: &str,
    prefix: &str,
    extra_allowed: &str,
    checked_dirs: &mut HashSet<String>,
    path: &std::path::Path,
    issues: &mut Vec<Issue>,
) {
    // Find the topology segment: prefix + name
    if let Some(prefix_pos) = normalized.find(prefix) {
        let after_prefix = &normalized[prefix_pos + prefix.len()..];
        let name = after_prefix.split('/').next().unwrap_or("");
        if name.is_empty() {
            return;
        }

        let key = format!("{prefix}{name}");
        if checked_dirs.contains(&key) {
            return;
        }
        checked_dirs.insert(key);

        // Skip non-topology entries (files, not dirs)
        if !after_prefix.contains('/') {
            return; // This is a file directly in src/ or crates/, not a subfolder
        }

        // Check for banned names
        for (banned, replacement) in BANNED_NAMES {
            if name == *banned {
                issues.push(Issue::new(
                    path,
                    0,
                    0,
                    Severity::Error,
                    "topology/naming",
                    &format!(
                        "banned folder name `{prefix}{name}` — rename to `{prefix}{replacement}`",
                    ),
                ));
                return;
            }
        }

        // Check against legal names (only if crates/ — src/ subfolders are more flexible)
        if prefix == "crates/" {
            let is_legal = LEGAL_TOPOLOGY_NAMES.contains(&name)
                || extra_allowed.contains(name);
            if !is_legal {
                issues.push(Issue::new(
                    path,
                    0,
                    0,
                    Severity::Error,
                    "topology/naming",
                    &format!(
                        "unregistered crate `{prefix}{name}` — only registered topology names allowed in crates/",
                    ),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banned_name_infra() {
        let paths = vec![PathBuf::from("src/infra/db.rs")];
        let mut issues = Vec::new();
        check(&paths, &Config::default(), &mut issues);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("gateway"));
    }

    #[test]
    fn banned_name_helpers() {
        let paths = vec![PathBuf::from("src/helpers/utils.rs")];
        let mut issues = Vec::new();
        check(&paths, &Config::default(), &mut issues);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("shared"));
    }

    #[test]
    fn banned_name_in_crates() {
        let paths = vec![PathBuf::from("crates/infra/src/lib.rs")];
        let mut issues = Vec::new();
        check(&paths, &Config::default(), &mut issues);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("gateway"));
    }

    #[test]
    fn legal_names_ok() {
        let paths = vec![
            PathBuf::from("src/core/engine.rs"),
            PathBuf::from("src/adapter/hub.rs"),
            PathBuf::from("src/gateway/db.rs"),
            PathBuf::from("src/pal/windows.rs"),
            PathBuf::from("src/ui/menu.rs"),
        ];
        let mut issues = Vec::new();
        check(&paths, &Config::default(), &mut issues);
        assert!(issues.is_empty());
    }

    #[test]
    fn legal_crate_names_ok() {
        let paths = vec![
            PathBuf::from("crates/core/src/lib.rs"),
            PathBuf::from("crates/adapter/src/lib.rs"),
            PathBuf::from("crates/app/src/main.rs"),
            PathBuf::from("crates/mcp/src/main.rs"),
        ];
        let mut issues = Vec::new();
        check(&paths, &Config::default(), &mut issues);
        assert!(issues.is_empty());
    }

    #[test]
    fn unregistered_crate_error() {
        let paths = vec![PathBuf::from("crates/ai-helper/src/lib.rs")];
        let mut issues = Vec::new();
        check(&paths, &Config::default(), &mut issues);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("unregistered"));
    }

    #[test]
    fn nested_ok() {
        // Deep nesting inside a topology layer is fine (Level 2+)
        let paths = vec![PathBuf::from("src/core/parser/lexer/token.rs")];
        let mut issues = Vec::new();
        check(&paths, &Config::default(), &mut issues);
        assert!(issues.is_empty()); // core is legal
    }

    #[test]
    fn files_in_src_ok() {
        // Direct files in src/ (main.rs, lib.rs) are not topology folders
        let paths = vec![PathBuf::from("src/main.rs")];
        let mut issues = Vec::new();
        check(&paths, &Config::default(), &mut issues);
        assert!(issues.is_empty());
    }

    #[test]
    fn each_banned_only_reported_once() {
        let paths = vec![
            PathBuf::from("src/infra/db.rs"),
            PathBuf::from("src/infra/cache.rs"),
            PathBuf::from("src/infra/config.rs"),
        ];
        let mut issues = Vec::new();
        check(&paths, &Config::default(), &mut issues);
        assert_eq!(issues.len(), 1); // reported once, not per file
    }
}
