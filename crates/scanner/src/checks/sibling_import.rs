use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::Config;
use crate::issue::{Issue, Severity};

/// Check that child modules do not import sibling children directly.
///
/// Children must route through mother (mod.rs) or extract to shared/.
/// `use super::<sibling>` in a child file is an error.
pub fn check(
    contents: &[(PathBuf, String)],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
) {
    // Group files by parent directory
    let mut dir_children: HashMap<PathBuf, Vec<(PathBuf, String)>> = HashMap::new();
    for (path, content) in contents {
        let normalized = path.to_string_lossy().replace('\\', "/");
        // Skip mother files — they ARE the routing point
        let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
        if filename == "mod.rs" || filename == "lib.rs" || filename == "main.rs" {
            continue;
        }
        // Skip test files
        if filename.starts_with("test_") || filename.ends_with("_test.rs")
            || filename.ends_with("_tests.rs") || filename == "tests.rs"
            || normalized.contains("/tests/")
        {
            continue;
        }
        if let Some(parent) = path.parent() {
            dir_children
                .entry(parent.to_path_buf())
                .or_default()
                .push((path.clone(), content.clone()));
        }
    }

    // For each directory, collect sibling module names
    for (_dir, children) in &dir_children {
        let sibling_names: Vec<String> = children
            .iter()
            .filter_map(|(p, _)| {
                p.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            })
            .collect();

        // Check each child for sibling imports
        for (path, content) in children {
            for (i, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                    continue;
                }

                // Parse use super::<name>
                let target = if trimmed.starts_with("use super::") {
                    extract_super_target(trimmed)
                } else if trimmed.starts_with("pub use super::") {
                    extract_super_target(trimmed)
                } else {
                    continue;
                };

                if let Some(target_name) = target {
                    // Check if target is a sibling (not mod/self)
                    if target_name != "self" && sibling_names.contains(&target_name) {
                        let own_name = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("?");
                        if target_name != own_name {
                            issues.push(Issue::new(
                                path,
                                i + 1,
                                1,
                                Severity::Error,
                                "rust/modules/no-sibling-import",
                                &format!(
                                    "sibling import `{}` — route through mother (mod.rs) or extract to shared/",
                                    target_name,
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }
}

/// Extract the first module name from `use super::<name>`.
fn extract_super_target(line: &str) -> Option<String> {
    let after_super = if line.contains("use super::") {
        let idx = line.find("use super::")? + "use super::".len();
        &line[idx..]
    } else {
        return None;
    };
    let name: String = after_super
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sibling_import_detected() {
        let contents = vec![
            (PathBuf::from("src/gateway/db.rs"), "pub fn connect() {}\n".to_string()),
            (
                PathBuf::from("src/gateway/cache.rs"),
                "use super::db;\npub fn cached() {}\n".to_string(),
            ),
        ];
        let mut issues = Vec::new();
        check(&contents, &Config::default(), &mut issues);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "rust/modules/no-sibling-import");
        assert!(issues[0].message.contains("db"));
    }

    #[test]
    fn mother_import_ok() {
        // mod.rs is a mother — not in children list, so importing from super is fine
        let contents = vec![
            (PathBuf::from("src/gateway/mod.rs"), "pub mod db;\npub mod cache;\n".to_string()),
            (
                PathBuf::from("src/gateway/db.rs"),
                "use super::cache;\n".to_string(), // import sibling
            ),
            (PathBuf::from("src/gateway/cache.rs"), "pub fn cached() {}\n".to_string()),
        ];
        let mut issues = Vec::new();
        check(&contents, &Config::default(), &mut issues);
        // db.rs imports cache.rs (sibling) — this should error
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn no_siblings_no_issues() {
        let contents = vec![(
            PathBuf::from("src/gateway/db.rs"),
            "use std::collections::HashMap;\n".to_string(),
        )];
        let mut issues = Vec::new();
        check(&contents, &Config::default(), &mut issues);
        assert!(issues.is_empty());
    }

    #[test]
    fn different_dirs_no_conflict() {
        let contents = vec![
            (PathBuf::from("src/core/calc.rs"), "pub fn calc() {}\n".to_string()),
            (
                PathBuf::from("src/gateway/db.rs"),
                "use super::calc;\n".to_string(), // calc is not a sibling of db
            ),
        ];
        let mut issues = Vec::new();
        check(&contents, &Config::default(), &mut issues);
        assert!(issues.is_empty()); // different directories
    }

    #[test]
    fn test_files_skipped() {
        let contents = vec![
            (PathBuf::from("src/gateway/db.rs"), "pub fn connect() {}\n".to_string()),
            (
                PathBuf::from("src/gateway/test_db.rs"),
                "use super::db;\n".to_string(),
            ),
        ];
        let mut issues = Vec::new();
        check(&contents, &Config::default(), &mut issues);
        assert!(issues.is_empty()); // test files exempt
    }

    #[test]
    fn comments_ignored() {
        let contents = vec![
            (PathBuf::from("src/gateway/db.rs"), "pub fn connect() {}\n".to_string()),
            (
                PathBuf::from("src/gateway/cache.rs"),
                "// use super::db;\n".to_string(),
            ),
        ];
        let mut issues = Vec::new();
        check(&contents, &Config::default(), &mut issues);
        assert!(issues.is_empty());
    }
}
