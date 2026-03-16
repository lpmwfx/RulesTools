use std::collections::HashMap;
use std::path::PathBuf;

use regex::Regex;
use std::sync::LazyLock;

use crate::config::Config;
use crate::issue::{Issue, Severity};

static PUB_FN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*pub(?:\([^)]+\))?\s+(?:async\s+)?fn\s+(\w+)").unwrap()
});

/// Cross-file check: find duplicate pub fn names across child files in the same module.
/// Functions that appear in 2+ siblings are candidates for `shared/` extraction.
pub fn check(
    contents: &[(PathBuf, String)],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
) {
    // Group files by parent directory
    let mut dir_groups: HashMap<PathBuf, Vec<(PathBuf, Vec<String>)>> = HashMap::new();

    for (path, content) in contents {
        // Skip test files
        let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
        if filename.starts_with("test_")
            || filename.ends_with("_test.rs")
            || filename.ends_with("_tests.rs")
            || filename == "tests.rs"
        {
            continue;
        }
        // Skip mother files and shared/ directories
        if filename == "mod.rs" || filename == "main.rs" || filename == "lib.rs" {
            continue;
        }
        let path_str = path.to_string_lossy();
        if path_str.contains("shared") || path_str.contains("common") {
            continue;
        }

        // Only Rust files
        if !filename.ends_with(".rs") {
            continue;
        }

        let parent = match path.parent() {
            Some(p) => p.to_path_buf(),
            None => continue,
        };

        // Common trait-like fn names that are expected to appear in many files
        const SKIP_FN_NAMES: &[&str] = &[
            "new", "default", "from", "into", "try_from", "try_into",
            "fmt", "clone", "drop", "deref", "eq", "hash",
            "build", "init", "run", "start", "stop",
        ];

        let fn_names: Vec<String> = content
            .lines()
            .filter_map(|line| {
                PUB_FN_RE.captures(line).and_then(|caps| {
                    let name = caps[1].to_string();
                    if SKIP_FN_NAMES.contains(&name.as_str()) {
                        None
                    } else {
                        Some(name)
                    }
                })
            })
            .collect();

        dir_groups
            .entry(parent)
            .or_default()
            .push((path.clone(), fn_names));
    }

    // For each directory, find fn names that appear in 2+ files
    for (_dir, file_fns) in &dir_groups {
        if file_fns.len() < 2 {
            continue;
        }

        let mut fn_locations: HashMap<&str, Vec<&PathBuf>> = HashMap::new();
        for (path, fns) in file_fns {
            for fname in fns {
                fn_locations.entry(fname.as_str()).or_default().push(path);
            }
        }

        for (fn_name, locations) in &fn_locations {
            if locations.len() >= 2 {
                let file_list: Vec<String> = locations
                    .iter()
                    .filter_map(|p| p.file_name().and_then(|f| f.to_str()).map(String::from))
                    .collect();

                // Report on the first file
                issues.push(Issue::new(
                    locations[0],
                    1,
                    1,
                    Severity::Warning,
                    "rust/modules/shared-candidate",
                    format!(
                        "pub fn `{fn_name}` exists in {} sibling files ({}) — extract to shared/",
                        locations.len(),
                        file_list.join(", "),
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
    fn finds_duplicate_fn_across_siblings() {
        let mut issues = Vec::new();
        let contents = vec![
            (PathBuf::from("src/checks/alpha.rs"), "pub fn validate() {}\npub fn unique_a() {}".to_string()),
            (PathBuf::from("src/checks/beta.rs"), "pub fn validate() {}\npub fn unique_b() {}".to_string()),
        ];
        check(&contents, &Config::default(), &mut issues);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("validate"));
        assert!(issues[0].message.contains("shared"));
    }

    #[test]
    fn no_duplicates_no_issue() {
        let mut issues = Vec::new();
        let contents = vec![
            (PathBuf::from("src/checks/alpha.rs"), "pub fn alpha_check() {}".to_string()),
            (PathBuf::from("src/checks/beta.rs"), "pub fn beta_check() {}".to_string()),
        ];
        check(&contents, &Config::default(), &mut issues);
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_shared_dir() {
        let mut issues = Vec::new();
        let contents = vec![
            (PathBuf::from("src/shared/validate.rs"), "pub fn validate() {}".to_string()),
            (PathBuf::from("src/checks/alpha.rs"), "pub fn validate() {}".to_string()),
        ];
        check(&contents, &Config::default(), &mut issues);
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_mother_files() {
        let mut issues = Vec::new();
        let contents = vec![
            (PathBuf::from("src/mod.rs"), "pub fn validate() {}".to_string()),
            (PathBuf::from("src/alpha.rs"), "pub fn validate() {}".to_string()),
        ];
        check(&contents, &Config::default(), &mut issues);
        assert!(issues.is_empty());
    }
}
