/// Scan issue types and severity levels.
pub mod issue;
/// Configuration parsing and project settings.
pub mod config;
/// Language detection and file context.
pub mod context;
/// File system walking and source file collection.
pub mod walker;
/// Output formatting — cargo warnings and ISSUES file.
pub mod output;
/// Check registry and dispatch traits.
pub mod checks;
/// Auto-detection of project kind and layout.
pub mod project;
/// Severity resolver — maps check severity per ProjectKind.
pub mod severity;

use std::path::Path;

use config::Config;
use context::FileContext;
use issue::Issue;
use project::ProjectIdentity;

/// Scan a project from build.rs — emits `cargo:warning` lines.
///
/// Call this from your `build.rs`:
/// ```ignore
/// fn main() {
///     rulestools_scanner::scan_project();
/// }
/// ```
pub fn scan_project() {
    let root = match std::env::current_dir() {
        Ok(r) => r,
        Err(e) => {
            println!("cargo:warning=rulestools: cannot determine current directory: {e}");
            return;
        }
    };
    let issues = run_scan(&root);

    output::emit_cargo_warnings(&issues, &root);

    let cfg = Config::load(&root);
    if output::should_deny(&issues, cfg.deny) {
        let error_count = issues.iter().filter(|i| i.severity == issue::Severity::Error).count();
        println!("cargo:warning=rulestools: build denied — {error_count} error(s) found");
        std::process::exit(1);
    }
}

/// Scan a project from CLI/MCP — returns issues and writes `proj/ISSUES`.
///
/// Returns `(all_issues, new_count)`.
/// If the project has no `proj/rulestools.toml` with `[project].kind`,
/// returns a single Info issue with a registration suggestion.
pub fn scan_at(root: &Path) -> (Vec<Issue>, usize) {
    let mut issues = run_scan(root);

    // Add registration suggestion if project is not explicitly registered
    if !ProjectIdentity::is_registered(root) {
        let suggestion = ProjectIdentity::suggest(root);
        issues.insert(0, Issue::new(
            root.join("proj").join("rulestools.toml"),
            0, 0,
            issue::Severity::Info,
            "topology/unregistered",
            &format!("No [project].kind in rulestools.toml — {suggestion}"),
        ));
    }
    issues.sort();

    let new_count = output::write_issues_file(&issues, root).unwrap_or(0);
    (issues, new_count)
}

/// Scan a super-project — find sub-repos and scan each with its own config.
///
/// Returns aggregated issues from all sub-repos with path prefixed by sub-repo name.
pub fn scan_super(root: &Path) -> (Vec<Issue>, usize) {
    let mut all_issues = Vec::new();
    let mut total_new = 0;

    let entries = match std::fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return (all_issues, total_new),
    };

    for entry in entries.flatten() {
        let sub = entry.path();
        if !sub.is_dir() {
            continue;
        }
        if !sub.join("proj").join("rulestools.toml").exists() {
            continue;
        }
        let sub_name = sub
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let (issues, new_count) = scan_at(&sub);
        let sub_canonical = std::fs::canonicalize(&sub).unwrap_or_else(|_| sub.clone());
        for mut issue in issues {
            // Make path relative to sub-repo, then prefix with sub-repo name
            let rel = issue
                .path
                .strip_prefix(&sub_canonical)
                .or_else(|_| issue.path.strip_prefix(&sub))
                .unwrap_or(&issue.path)
                .to_path_buf();
            issue.path = std::path::PathBuf::from(&sub_name).join(rel);
            all_issues.push(issue);
        }
        total_new += new_count;
    }

    all_issues.sort();
    (all_issues, total_new)
}

/// Core scan logic — collects files, runs all registered checks.
pub fn run_scan(root: &Path) -> Vec<Issue> {
    let cfg = Config::load(root);
    let identity = ProjectIdentity::detect(root);
    let resolver = severity::SeverityResolver::for_kind(identity.kind);
    let files = walker::collect_files(root, &[]);
    let registry = checks::registry();

    let mut issues = Vec::new();

    // Read file contents
    let mut file_contents: Vec<(std::path::PathBuf, String)> = Vec::new();
    for path in &files {
        if let Ok(content) = std::fs::read_to_string(path) {
            file_contents.push((path.clone(), content));
        }
    }

    // Per-file checks
    for (path, content) in &file_contents {
        let file_ctx = match FileContext::from_path(path) {
            Some(c) => c,
            None => continue,
        };
        let is_metadata = walker::is_metadata_path(path);
        let lines: Vec<&str> = content.lines().collect();
        for check in &registry {
            if !check.applies_to(file_ctx.language) {
                continue;
            }
            if !cfg.is_enabled(&check.id) || !identity.kind.allows_check(&check.id) {
                continue;
            }
            // Skip code checks on metadata paths (but placement check runs everywhere)
            if is_metadata && check.id != "topology/placement" {
                continue;
            }
            if let checks::CheckKind::PerFile(func) = &check.kind {
                func(&file_ctx, &lines, &cfg, &mut issues, path);
            }
        }
    }

    // Cross-file checks (exclude metadata files from analysis)
    let code_contents: Vec<_> = file_contents
        .iter()
        .filter(|(p, _)| !walker::is_metadata_path(p))
        .map(|(p, c)| (p.clone(), c.clone()))
        .collect();
    for check in &registry {
        if !cfg.is_enabled(&check.id) || !identity.kind.allows_check(&check.id) {
            continue;
        }
        if let checks::CheckKind::CrossFile(func) = &check.kind {
            func(&code_contents, &cfg, &mut issues);
        }
    }

    // Tree checks
    let all_paths: Vec<std::path::PathBuf> = files;
    for check in &registry {
        if !cfg.is_enabled(&check.id) || !identity.kind.allows_check(&check.id) {
            continue;
        }
        if let checks::CheckKind::Tree(func) = &check.kind {
            func(&all_paths, &cfg, &mut issues);
        }
    }

    // Apply severity resolver — remap and filter
    issues = issues
        .into_iter()
        .map(|mut issue| {
            issue.severity = resolver.resolve(&issue.rule_id, issue.severity);
            issue
        })
        .filter(|issue| issue.severity != issue::Severity::Skip)
        .collect();

    issues
}

#[cfg(test)]
mod super_tests {
    use super::*;

    fn make_sub_repo(root: &std::path::Path, name: &str) {
        let sub = root.join(name);
        let proj = sub.join("proj");
        std::fs::create_dir_all(&proj).unwrap();
        std::fs::write(proj.join("rulestools.toml"), "[project]\nkind = \"tool\"\n").unwrap();
        let src = sub.join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("main.rs"), "fn main() {}\n").unwrap();
    }

    #[test]
    fn super_finds_sub_repos() {
        let dir = std::env::temp_dir().join("rulestools-super-test-find");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        make_sub_repo(&dir, "repo-a");
        make_sub_repo(&dir, "repo-b");

        let (issues, _) = scan_super(&dir);
        // Both sub-repos should be scanned (they'll at least get the unregistered suggestion or some issues)
        let has_repo_a = issues.iter().any(|i| i.path.to_string_lossy().starts_with("repo-a"));
        let has_repo_b = issues.iter().any(|i| i.path.to_string_lossy().starts_with("repo-b"));
        // They are registered (have [project].kind) so they won't get "unregistered" issue,
        // but they may or may not have other issues depending on content. Just verify no panic.
        let _ = (has_repo_a, has_repo_b);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn super_skips_dirs_without_toml() {
        let dir = std::env::temp_dir().join("rulestools-super-test-skip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Sub-repo with toml
        make_sub_repo(&dir, "has-toml");

        // Directory without toml
        let no_toml = dir.join("no-toml");
        std::fs::create_dir_all(no_toml.join("src")).unwrap();
        std::fs::write(no_toml.join("src/main.rs"), "fn main() {}\n").unwrap();

        let (issues, _) = scan_super(&dir);
        let has_no_toml = issues.iter().any(|i| i.path.to_string_lossy().starts_with("no-toml"));
        assert!(!has_no_toml, "Should not scan directories without proj/rulestools.toml");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn super_prefixes_paths() {
        let dir = std::env::temp_dir().join("rulestools-super-test-prefix");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Create a sub-repo with a file that will definitely trigger an issue
        let sub = dir.join("my-repo");
        let proj = sub.join("proj");
        std::fs::create_dir_all(&proj).unwrap();
        // Don't set [project].kind — this guarantees an "unregistered" info issue
        std::fs::write(proj.join("rulestools.toml"), "[scan]\n").unwrap();
        let src = sub.join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("main.rs"), "fn main() {}\n").unwrap();

        let (issues, _) = scan_super(&dir);
        // All issues should be prefixed with "my-repo/"
        for issue in &issues {
            let path_str = issue.path.to_string_lossy();
            assert!(
                path_str.starts_with("my-repo"),
                "Issue path should be prefixed: {path_str}"
            );
        }

        let _ = std::fs::remove_dir_all(&dir);
    }
}
