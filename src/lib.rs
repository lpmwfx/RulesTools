pub mod issue;
pub mod config;
pub mod context;
pub mod walker;
pub mod output;
pub mod checks;

use std::path::Path;

use config::Config;
use context::FileContext;
use issue::Issue;

/// Scan a project from build.rs — emits `cargo:warning` lines.
///
/// Call this from your `build.rs`:
/// ```ignore
/// fn main() {
///     rulestools::scan_project();
/// }
/// ```
pub fn scan_project() {
    let root = std::env::current_dir().expect("cannot determine current directory");
    let issues = run_scan(&root);

    output::emit_cargo_warnings(&issues, &root);

    let cfg = Config::load(&root);
    if output::should_deny(&issues, cfg.deny) {
        panic!("rulestools: build denied — {} error(s) found", issues.iter().filter(|i| i.severity == issue::Severity::Error).count());
    }
}

/// Scan a project from CLI/MCP — returns issues and writes `proj/ISSUES`.
///
/// Returns `(all_issues, new_count)`.
pub fn scan_at(root: &Path) -> (Vec<Issue>, usize) {
    let mut issues = run_scan(root);
    issues.sort();

    let new_count = output::write_issues_file(&issues, root).unwrap_or(0);
    (issues, new_count)
}

/// Core scan logic — collects files, runs all registered checks.
pub fn run_scan(root: &Path) -> Vec<Issue> {
    let cfg = Config::load(root);
    let files = walker::collect_files(root, &[]);
    let registry = checks::registry();

    let mut issues = Vec::new();

    // Read all file contents for cross-file and per-file checks
    let mut file_contents: Vec<(std::path::PathBuf, String)> = Vec::new();
    for path in &files {
        if let Ok(content) = std::fs::read_to_string(path) {
            file_contents.push((path.clone(), content));
        }
    }

    // Per-file checks
    for (path, content) in &file_contents {
        let ctx = match FileContext::from_path(path) {
            Some(c) => c,
            None => continue,
        };

        let lines: Vec<&str> = content.lines().collect();

        for check in &registry {
            if !check.applies_to(ctx.language) {
                continue;
            }
            if !cfg.is_enabled(&check.id) {
                continue;
            }
            if let checks::CheckKind::PerFile(func) = &check.kind {
                func(&ctx, &lines, &cfg, &mut issues, path);
            }
        }
    }

    // Cross-file checks
    for check in &registry {
        if !cfg.is_enabled(&check.id) {
            continue;
        }
        if let checks::CheckKind::CrossFile(func) = &check.kind {
            func(&file_contents, &cfg, &mut issues);
        }
    }

    // Tree checks
    let all_paths: Vec<std::path::PathBuf> = files.clone();
    for check in &registry {
        if !cfg.is_enabled(&check.id) {
            continue;
        }
        if let checks::CheckKind::Tree(func) = &check.kind {
            func(&all_paths, &cfg, &mut issues);
        }
    }

    issues
}
