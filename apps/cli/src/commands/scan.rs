use std::path::PathBuf;

/// Walk up from file to find project root (directory with Cargo.toml or proj/).
pub fn find_project_root(path: &std::path::Path) -> PathBuf {
    let mut current = if path.is_file() {
        path.parent().unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    };
    loop {
        if current.join("Cargo.toml").exists() || current.join("proj").exists() {
            return current;
        }
        if !current.pop() {
            return path.to_path_buf();
        }
    }
}

/// Find the Rules/ directory — check common locations.
pub fn find_rules_root(project_root: &std::path::Path) -> Option<PathBuf> {
    // 1. RULES_REPO env var
    if let Ok(path) = std::env::var("RULES_REPO") {
        let p = PathBuf::from(&path);
        if p.join("guidance").exists() {
            return Some(p);
        }
    }
    // 2. Sibling Rules/ directory (in superprojekt like Rules-dev)
    if let Some(parent) = project_root.parent() {
        let sibling = parent.join("Rules");
        if sibling.join("guidance").exists() {
            return Some(sibling);
        }
    }
    // 3. Rules/ inside project root
    let inside = project_root.join("Rules");
    if inside.join("guidance").exists() {
        return Some(inside);
    }
    None
}

/// fn `scan_internal`.
pub fn scan_internal(path: &std::path::Path, deny: bool) -> Result<String, String> {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let identity = rulestools_scanner::project::ProjectIdentity::detect(&root);

    let mut output = format!("rulestools: {:?} / {:?}\n", identity.kind, identity.layout);

    let (issues, new_count) = if identity.kind == rulestools_scanner::project::ProjectKind::Super {
        rulestools_scanner::scan_super(&root)
    } else {
        rulestools_scanner::scan_at(&root)
    };

    let error_count = issues
        .iter()
        .filter(|i| i.severity == rulestools_scanner::issue::Severity::Error)
        .count();
    let warning_count = issues
        .iter()
        .filter(|i| i.severity == rulestools_scanner::issue::Severity::Warning)
        .count();

    if issues.is_empty() {
        output.push_str("rulestools: 0 issues\n");
    } else {
        output.push_str(&format!(
            "rulestools: {} issues ({} errors, {} warnings, {} new)\n",
            issues.len(),
            error_count,
            warning_count,
            new_count,
        ));
        let rules_root = find_rules_root(&root);
        let grouped = rulestools_scanner::output::format_grouped_with_guidance(
            &issues,
            &root,
            rules_root.as_deref(),
        );
        output.push_str(&grouped);
    }

    // Run documenter: insert /// stubs + generate man/
    if identity.kind != rulestools_scanner::project::ProjectKind::Super {
        let doc_summary = super::generate::gen_internal(&root);
        output.push_str(&doc_summary);
    }

    if deny && error_count > 0 {
        Err(output)
    } else {
        Ok(output)
    }
}

/// fn `cmd_scan`.
pub fn cmd_scan(path: &PathBuf, deny: bool) {
    match scan_internal(path, deny) {
        Ok(output) => print!("{output}"),
        Err(output) => {
            print!("{output}");
            std::process::exit(1);
        }
    }
}

/// fn `scan_file_internal`.
pub fn scan_file_internal(file: &std::path::Path, format: &str) -> Result<String, String> {
    use rulestools_scanner::{checks, config::Config, context::FileContext, project::ProjectIdentity};

    let path = std::fs::canonicalize(file).unwrap_or_else(|_| file.to_path_buf());
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Cannot read file: {e}"))?;

    let file_ctx = match FileContext::from_path(&path) {
        Some(c) => c,
        None => return Ok("SKIP — unsupported file type".into()),
    };

    let project_root = find_project_root(&path);
    let cfg = Config::load(&project_root);
    let identity = ProjectIdentity::detect(&project_root);
    let resolver = rulestools_scanner::severity::SeverityResolver::for_kind(identity.kind);
    let registry = checks::registry();

    let lines: Vec<&str> = content.lines().collect();
    let mut issues = Vec::new();

    for check in &registry {
        if !check.applies_to(file_ctx.language) {
            continue;
        }
        if !cfg.is_enabled(&check.id) || !identity.kind.allows_check(&check.id) {
            continue;
        }
        if let checks::CheckKind::PerFile(func) = &check.kind {
            func(&file_ctx, &lines, &cfg, &mut issues, &path);
        }
    }

    issues = issues
        .into_iter()
        .map(|mut issue| {
            issue.severity = resolver.resolve(&issue.rule_id, issue.severity);
            issue
        })
        .filter(|issue| issue.severity != rulestools_scanner::issue::Severity::Skip)
        .collect();

    if format == "json" {
        let json_issues: Vec<serde_json::Value> = issues.iter().map(|i| {
            serde_json::json!({
                "path": i.path.to_string_lossy(),
                "line": i.line,
                "col": i.col,
                "severity": i.severity.label(),
                "rule_id": i.rule_id,
                "message": i.message,
            })
        }).collect();
        Ok(serde_json::to_string(&json_issues).unwrap_or_default())
    } else {
        if issues.is_empty() {
            Ok("CLEAN — no violations found".into())
        } else {
            let mut out = String::new();
            for issue in &issues {
                out.push_str(&issue.display_line());
                out.push('\n');
            }
            let error_count = issues.iter().filter(|i| i.severity == rulestools_scanner::issue::Severity::Error).count();
            out.push_str(&format!("\n{error_count} error(s), {} warning(s)", issues.len() - error_count));
            Ok(out)
        }
    }
}

/// fn `cmd_scan_file`.
pub fn cmd_scan_file(file: &PathBuf, format: &str) {
    match scan_file_internal(file, format) {
        Ok(output) => println!("{output}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
