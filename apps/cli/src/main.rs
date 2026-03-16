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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { path, deny } => cmd_scan(&path, deny),
        Commands::Check { path } => cmd_scan(&path, true),
        Commands::List { path } => cmd_list(&path),
        Commands::Detect { path } => cmd_detect(&path),
        Commands::Gen { path } => cmd_gen(&path),
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
        for issue in &issues {
            println!("{}", issue.display_line());
        }
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
