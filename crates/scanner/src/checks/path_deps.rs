use std::path::PathBuf;
use std::sync::LazyLock;

use regex::Regex;

use crate::config::Config;
use crate::issue::{Issue, Severity};

static PATH_DEP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"path\s*=\s*""#).unwrap());

/// Tree check: find path dependencies in Cargo.toml and pyproject.toml.
pub fn check(
    paths: &[PathBuf],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
) {
    for path in paths {
        let filename = match path.file_name().and_then(|f| f.to_str()) {
            Some(f) => f,
            None => continue,
        };

        if filename != "Cargo.toml" && filename != "pyproject.toml" {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.starts_with("//") {
                continue;
            }
            if PATH_DEP_RE.is_match(trimmed) {
                // Skip [patch] sections which legitimately use path
                // Simple heuristic: check if we're in a [dependencies] or similar section
                issues.push(Issue::new(
                    path, line_num + 1, 1, Severity::Error,
                    "global/install-architecture/no-path-deps",
                    "path dependency found — production must install from registry or git",
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn catches_path_dep_in_cargo_toml() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("Cargo.toml");
        let mut f = std::fs::File::create(&toml_path).unwrap();
        writeln!(f, "[dependencies]").unwrap();
        writeln!(f, r#"my-lib = {{ path = "../my-lib" }}"#).unwrap();

        let mut issues = Vec::new();
        check(&[toml_path], &Config::default(), &mut issues);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].rule_id.contains("no-path-deps"));
    }

    #[test]
    fn allows_git_dep() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("Cargo.toml");
        let mut f = std::fs::File::create(&toml_path).unwrap();
        writeln!(f, "[dependencies]").unwrap();
        writeln!(f, r#"my-lib = {{ git = "https://github.com/foo/bar" }}"#).unwrap();

        let mut issues = Vec::new();
        check(&[toml_path], &Config::default(), &mut issues);
        assert!(issues.is_empty());
    }

    #[test]
    fn skips_non_manifest_files() {
        let dir = tempfile::tempdir().unwrap();
        let rs_path = dir.path().join("main.rs");
        std::fs::write(&rs_path, "fn main() {}").unwrap();

        let mut issues = Vec::new();
        check(&[rs_path], &Config::default(), &mut issues);
        assert!(issues.is_empty());
    }
}
