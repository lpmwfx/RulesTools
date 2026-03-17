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
pub fn scan_at(root: &Path) -> (Vec<Issue>, usize) {
    let mut issues = run_scan(root);
    issues.sort();

    let new_count = output::write_issues_file(&issues, root).unwrap_or(0);
    (issues, new_count)
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
