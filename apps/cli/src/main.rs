use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Unified static code scanner — enforces Rules repo conventions across all languages.
///
/// Scans source files for coding rule violations and writes results to proj/ISSUES.
/// Supports Rust, Slint, Python, JavaScript/TypeScript, CSS, Kotlin, and C#.
#[derive(Parser)]
#[command(name = "rulestools", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan a project for rule violations.
    #[command(long_about = "Scan a project for rule violations.\n\n\
        Walks all source files under the given path, runs all enabled checks,\n\
        and writes results to proj/ISSUES with [NEW]/[KNOWN] delta markers.\n\
        \n\
        Exit code 0: no errors (warnings/info allowed)\n\
        Exit code 1: errors found (or --deny and any issues)\n\
        \n\
        Examples:\n  \
        rulestools scan .                   # scan current directory\n  \
        rulestools scan /path/to/project    # scan specific project\n  \
        rulestools scan . --deny            # fail on any error")]
    Scan {
        /// Path to the project root to scan.
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Deny build if any error-severity issues are found.
        #[arg(long)]
        deny: bool,
    },

    /// Check files (pre-commit mode) — scan and exit non-zero on errors.
    #[command(long_about = "Check files for pre-commit — scan and exit non-zero on errors.\n\n\
        Equivalent to `scan --deny`. Designed for use in pre-commit hooks\n\
        and CI pipelines where a non-zero exit code should block the commit.\n\
        \n\
        Examples:\n  \
        rulestools check .                  # check current directory\n  \
        rulestools check /path/to/project   # check specific project")]
    Check {
        /// Path to the project root to check.
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// List all registered checks and their status.
    #[command(long_about = "List all registered checks and their status.\n\n\
        Shows each check ID, the languages it applies to, and whether\n\
        it is enabled or disabled based on proj/rulestools.toml.\n\
        \n\
        Examples:\n  \
        rulestools list .                   # list checks for current directory\n  \
        rulestools list /path/to/project    # list checks for specific project")]
    List {
        /// Path to the project root (for reading config).
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Show auto-detected project identity.
    #[command(long_about = "Show auto-detected project identity.\n\n\
        Displays the project kind and layout as detected from the filesystem.\n\
        Useful for verifying which check-set will be applied.\n\
        \n\
        Examples:\n  \
        rulestools detect .                 # detect current directory\n  \
        rulestools detect /path/to/project  # detect specific project")]
    Detect {
        /// Path to the project root.
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Generate man/ documentation for a project.
    #[command(long_about = "Generate man/ documentation directory.\n\n\
        Scans all .rs and .slint files for pub items and their /// doc comments.\n\
        Writes JSON + Markdown to man/ with MANIFEST.\n\
        \n\
        Examples:\n  \
        rulestools gen .                    # generate for current directory\n  \
        rulestools gen /path/to/project     # generate for specific project")]
    Gen {
        /// Path to the project root.
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Scan a single file for rule violations (for MCP/IDE integration).
    #[command(name = "scan-file")]
    ScanFile {
        /// Absolute path to the file to scan.
        file: PathBuf,

        /// Output format.
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Report, list, or close issues on Forgejo.
    #[command(subcommand)]
    Issue(IssueCmd),
}

#[derive(Subcommand)]
enum IssueCmd {
    /// Report a new issue to Forgejo.
    Report {
        /// Issue title.
        #[arg(long)]
        title: String,
        /// Issue body (description).
        #[arg(long, default_value = "")]
        body: String,
        /// Labels (comma-separated).
        #[arg(long, default_value = "ai-reported")]
        labels: String,
    },
    /// List issues from Forgejo.
    List {
        /// Filter by state: open, closed, all.
        #[arg(long, default_value = "open")]
        state: String,
        /// Filter by labels (comma-separated).
        #[arg(long, default_value = "")]
        labels: String,
    },
    /// Close an issue by number.
    Close {
        /// Issue number.
        number: u64,
        /// Optional closing comment.
        #[arg(long, default_value = "")]
        comment: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { path, deny } => cmd_scan(&path, deny),
        Commands::Check { path } => cmd_scan(&path, true),
        Commands::List { path } => cmd_list(&path),
        Commands::Detect { path } => cmd_detect(&path),
        Commands::Gen { path } => cmd_gen(&path),
        Commands::ScanFile { file, format } => cmd_scan_file(&file, &format),
        Commands::Issue(cmd) => cmd_issue(cmd),
    }
}

fn cmd_scan(path: &PathBuf, deny: bool) {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
    let identity = rulestools_scanner::project::ProjectIdentity::detect(&root);

    println!("rulestools: {:?} / {:?}", identity.kind, identity.layout);

    let (issues, new_count) = rulestools_scanner::scan_at(&root);

    let error_count = issues
        .iter()
        .filter(|i| i.severity == rulestools_scanner::issue::Severity::Error)
        .count();
    let warning_count = issues
        .iter()
        .filter(|i| i.severity == rulestools_scanner::issue::Severity::Warning)
        .count();

    if issues.is_empty() {
        println!("rulestools: 0 issues");
    } else {
        println!(
            "rulestools: {} issues ({} errors, {} warnings, {} new)",
            issues.len(),
            error_count,
            warning_count,
            new_count,
        );
        // Grouped output with guidance + decision trees
        // Try to find Rules/ directory for guidance trees
        let rules_root = find_rules_root(&root);
        let grouped = rulestools_scanner::output::format_grouped_with_guidance(
            &issues,
            &root,
            rules_root.as_deref(),
        );
        print!("{grouped}");
    }

    if deny && error_count > 0 {
        std::process::exit(1);
    }
}

fn cmd_list(path: &PathBuf) {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
    let cfg = rulestools_scanner::config::Config::load(&root);
    let identity = rulestools_scanner::project::ProjectIdentity::detect(&root);
    let registry = rulestools_scanner::checks::registry();

    println!("rulestools: {:?} / {:?}", identity.kind, identity.layout);

    if registry.is_empty() {
        println!("0 checks registered (skeleton — checks added in next phase)");
        return;
    }

    println!("{:<40} {:<20} {}", "CHECK", "LANGUAGES", "STATUS");
    println!("{}", "-".repeat(70));

    for entry in &registry {
        let langs: Vec<&str> = entry.languages.iter().map(|l| l.name()).collect();
        let lang_str = if langs.is_empty() {
            "all".to_string()
        } else {
            langs.join(", ")
        };
        let active = cfg.is_enabled(&entry.id) && identity.kind.allows_check(&entry.id);
        let status = if active { "enabled" } else { "disabled" };
        println!("{:<40} {:<20} {}", entry.id, lang_str, status);
    }
}

fn cmd_gen(path: &PathBuf) {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
    let project_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
    rulestools_documenter::generate_docs(&root, project_name);
    println!("rulestools: man/ generated for {project_name}");
}

/// Find the Rules/ directory — check common locations.
fn find_rules_root(project_root: &std::path::Path) -> Option<PathBuf> {
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

fn cmd_scan_file(file: &PathBuf, format: &str) {
    use rulestools_scanner::{checks, config::Config, context::FileContext, project::ProjectIdentity};

    let path = std::fs::canonicalize(file).unwrap_or_else(|_| file.clone());
    if !path.exists() {
        eprintln!("File not found: {}", path.display());
        std::process::exit(1);
    }

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Cannot read file: {e}");
            std::process::exit(1);
        }
    };

    let file_ctx = match FileContext::from_path(&path) {
        Some(c) => c,
        None => {
            println!("SKIP — unsupported file type");
            return;
        }
    };

    // Find project root
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

    // Apply severity resolver
    issues = issues
        .into_iter()
        .map(|mut issue| {
            issue.severity = resolver.resolve(&issue.rule_id, issue.severity);
            issue
        })
        .filter(|issue| issue.severity != rulestools_scanner::issue::Severity::Skip)
        .collect();

    if format == "json" {
        // JSON output for MCP integration
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
        println!("{}", serde_json::to_string(&json_issues).unwrap_or_default());
    } else {
        if issues.is_empty() {
            println!("CLEAN — no violations found");
        } else {
            for issue in &issues {
                println!("{}", issue.display_line());
            }
            let error_count = issues.iter().filter(|i| i.severity == rulestools_scanner::issue::Severity::Error).count();
            println!("\n{error_count} error(s), {} warning(s)", issues.len() - error_count);
        }
    }
}

/// Walk up from file to find project root (directory with Cargo.toml or proj/).
fn find_project_root(path: &std::path::Path) -> PathBuf {
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

// --- Forgejo issue reporter ---

const FORGEJO_API: &str = "https://git.lpmintra.com/api/v1/repos/lpmwfx/issues";

fn forgejo_token() -> Result<String, String> {
    std::env::var("FORGEJO_TOKEN")
        .map_err(|_| "FORGEJO_TOKEN environment variable not set".to_string())
}

fn cmd_issue(cmd: IssueCmd) {
    match cmd {
        IssueCmd::Report { title, body, labels } => cmd_issue_report(&title, &body, &labels),
        IssueCmd::List { state, labels } => cmd_issue_list(&state, &labels),
        IssueCmd::Close { number, comment } => cmd_issue_close(number, &comment),
    }
}

fn cmd_issue_report(title: &str, body: &str, labels_str: &str) {
    let token = match forgejo_token() {
        Ok(t) => t,
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    };

    let labels: Vec<&str> = labels_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();

    // Get label IDs from names
    let label_ids = match resolve_label_ids(&token, &labels) {
        Ok(ids) => ids,
        Err(e) => { eprintln!("Cannot resolve labels: {e}"); Vec::new() }
    };

    let payload = serde_json::json!({
        "title": title,
        "body": body,
        "labels": label_ids,
    });

    match ureq::post(&format!("{FORGEJO_API}/issues"))
        .set("Authorization", &format!("token {token}"))
        .set("Content-Type", "application/json")
        .send_string(&payload.to_string())
    {
        Ok(resp) => {
            if let Ok(json) = resp.into_string().and_then(|s| Ok(serde_json::from_str::<serde_json::Value>(&s).unwrap_or_default())) {
                let number = json.get("number").and_then(|v| v.as_u64()).unwrap_or(0);
                let url = json.get("html_url").and_then(|v| v.as_str()).unwrap_or("?");
                println!("Issue #{number} created: {url}");
            }
        }
        Err(e) => { eprintln!("Failed to create issue: {e}"); std::process::exit(1); }
    }
}

fn cmd_issue_list(state: &str, labels_str: &str) {
    let token = match forgejo_token() {
        Ok(t) => t,
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    };

    let mut url = format!("{FORGEJO_API}/issues?state={state}&limit=50");
    if !labels_str.is_empty() {
        url.push_str(&format!("&labels={labels_str}"));
    }

    match ureq::get(&url)
        .set("Authorization", &format!("token {token}"))
        .call()
    {
        Ok(resp) => {
            if let Ok(body) = resp.into_string() {
                if let Ok(issues) = serde_json::from_str::<Vec<serde_json::Value>>(&body) {
                    if issues.is_empty() {
                        println!("No issues found");
                        return;
                    }
                    for issue in &issues {
                        let number = issue.get("number").and_then(|v| v.as_u64()).unwrap_or(0);
                        let title = issue.get("title").and_then(|v| v.as_str()).unwrap_or("?");
                        let state = issue.get("state").and_then(|v| v.as_str()).unwrap_or("?");
                        let labels: Vec<&str> = issue.get("labels")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|l| l.get("name").and_then(|n| n.as_str())).collect())
                            .unwrap_or_default();
                        println!("#{number} [{state}] {title}  {}", labels.join(", "));
                    }
                }
            }
        }
        Err(e) => { eprintln!("Failed to list issues: {e}"); std::process::exit(1); }
    }
}

fn cmd_issue_close(number: u64, comment: &str) {
    let token = match forgejo_token() {
        Ok(t) => t,
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    };

    // Add comment if provided
    if !comment.is_empty() {
        let payload = serde_json::json!({ "body": comment });
        let _ = ureq::post(&format!("{FORGEJO_API}/issues/{number}/comments"))
            .set("Authorization", &format!("token {token}"))
            .set("Content-Type", "application/json")
            .send_string(&payload.to_string());
    }

    // Close issue
    let payload = serde_json::json!({ "state": "closed" });
    match ureq::patch(&format!("{FORGEJO_API}/issues/{number}"))
        .set("Authorization", &format!("token {token}"))
        .set("Content-Type", "application/json")
        .send_string(&payload.to_string())
    {
        Ok(_) => println!("Issue #{number} closed"),
        Err(e) => { eprintln!("Failed to close issue: {e}"); std::process::exit(1); }
    }
}

fn resolve_label_ids(token: &str, names: &[&str]) -> Result<Vec<u64>, String> {
    let resp = ureq::get(&format!("{FORGEJO_API}/labels?limit=50"))
        .set("Authorization", &format!("token {token}"))
        .call()
        .map_err(|e| format!("Cannot fetch labels: {e}"))?;

    let body = resp.into_string().map_err(|e| format!("Cannot read response: {e}"))?;
    let all_labels: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap_or_default();

    let mut ids = Vec::new();
    for name in names {
        for label in &all_labels {
            if label.get("name").and_then(|v| v.as_str()) == Some(name) {
                if let Some(id) = label.get("id").and_then(|v| v.as_u64()) {
                    ids.push(id);
                }
            }
        }
    }
    Ok(ids)
}

fn cmd_detect(path: &PathBuf) {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
    let identity = rulestools_scanner::project::ProjectIdentity::detect(&root);
    let cfg = rulestools_scanner::config::Config::load(&root);

    println!("Project:  {}", root.display());
    println!("Kind:     {:?}", identity.kind);
    println!("Layout:   {:?}", identity.layout);
    println!("Languages: {:?}", cfg.languages);
    println!("Deny:     {}", cfg.deny);
    println!();
    println!("Skipped check categories:");
    for cat in identity.kind.skipped_categories() {
        println!("  - {cat}");
    }
}
