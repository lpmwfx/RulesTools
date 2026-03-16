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
    ///
    /// Walks all source files, runs enabled checks, and writes results
    /// to proj/ISSUES with [NEW]/[KNOWN] delta markers.
    ///
    /// Examples:
    ///   rulestools scan .
    ///   rulestools scan /path/to/project
    ///   rulestools scan . --deny
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
    ///
    /// Same as `scan --deny` but designed for pre-commit hooks.
    ///
    /// Examples:
    ///   rulestools check .
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
    ///
    /// Shows each check ID, which languages it applies to, and whether
    /// it is enabled or disabled in the project config.
    ///
    /// Examples:
    ///   rulestools list .
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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { path, deny } => cmd_scan(&path, deny),
        Commands::Check { path } => cmd_scan(&path, true),
        Commands::List { path } => cmd_list(&path),
    }
}

fn cmd_scan(path: &PathBuf, deny: bool) {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());

    let (issues, new_count) = rulestools::scan_at(&root);

    let error_count = issues
        .iter()
        .filter(|i| i.severity == rulestools::issue::Severity::Error)
        .count();
    let warning_count = issues
        .iter()
        .filter(|i| i.severity == rulestools::issue::Severity::Warning)
        .count();

    // Print summary
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

        // Print issues to stdout
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
    let cfg = rulestools::config::Config::load(&root);
    let registry = rulestools::checks::registry();

    if registry.is_empty() {
        println!("rulestools: 0 checks registered (phase 1 skeleton)");
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
        let status = if cfg.is_enabled(&entry.id) {
            "enabled"
        } else {
            "disabled"
        };
        println!("{:<40} {:<20} {}", entry.id, lang_str, status);
    }
}
