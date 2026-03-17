use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod publish;
mod scaffold;

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

    /// Initialize a new project with full scaffolding.
    #[command(long_about = "Initialize a new project with full scaffolding.\n\n\
        Creates directory structure, stub source files, Cargo.toml,\n\
        proj/ files, and .gitignore based on the chosen project kind.\n\
        Existing files are never overwritten.\n\
        \n\
        Kinds:\n  \
        tool       — minimal (src/main.rs, proj/)\n  \
        cli        — CLI app with clap (src/main.rs, src/shared/, doc/)\n  \
        library    — library crate (src/lib.rs)\n  \
        slint-app  — Slint GUI with topology folders\n  \
        workspace  — Cargo workspace with crates/{app,core,adapter,gateway,pal,ui}\n\
        \n\
        Examples:\n  \
        rulestools init /path/to/project --kind tool\n  \
        rulestools init . --kind cli --name my-app\n  \
        rulestools init /path/to/project --kind workspace")]
    Init {
        /// Path to the project root directory.
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Project kind: tool, cli, library, slint-app, workspace.
        #[arg(long)]
        kind: String,

        /// Project name (default: directory name).
        #[arg(long)]
        name: Option<String>,
    },

    /// Create a new project with full scaffolding and options.
    #[command(long_about = "Create a new project with full scaffolding and options.\n\n\
        Like `init` but with extra options: platforms, themes, MCP crate, extras.\n\
        Supports --preview to see what would be created without writing.\n\
        Supports --format json for machine-readable output.\n\
        \n\
        Kinds:\n  \
        tool       — minimal (src/main.rs, proj/)\n  \
        cli        — CLI app with clap (src/main.rs, src/shared/, doc/)\n  \
        library    — library crate (src/lib.rs)\n  \
        website    — web project (index.html, package.json, src/)\n  \
        slint-app  — Slint GUI with topology folders\n  \
        workspace  — Cargo workspace with crates/{app,core,adapter,gateway,pal,ui}\n\
        \n\
        Examples:\n  \
        rulestools new /path --kind slint-app --platforms desktop,mobile\n  \
        rulestools new /path --kind workspace --mcp --extras doc,shared\n  \
        rulestools new /path --kind cli --preview --format json")]
    New {
        /// Path to the project root directory.
        path: PathBuf,

        /// Project kind: tool, cli, library, website, slint-app, workspace.
        #[arg(long)]
        kind: String,

        /// Project name (default: directory name).
        #[arg(long)]
        name: Option<String>,

        /// Target platforms (comma-separated): desktop, mobile, small.
        #[arg(long, default_value = "")]
        platforms: String,

        /// Theme names (comma-separated), e.g. win3ui-fluent,macos.
        #[arg(long, default_value = "")]
        themes: String,

        /// Add MCP server crate (workspace only).
        #[arg(long)]
        mcp: bool,

        /// Extra folders/crates (comma-separated): lib, shared, doc.
        #[arg(long, default_value = "")]
        extras: String,

        /// Preview mode — show what would be created without writing.
        #[arg(long)]
        preview: bool,

        /// Output format: text or json.
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Add features to an existing project (within current kind).
    #[command(long_about = "Add features to an existing project without changing its kind.\n\n\
        Detects current project kind and adds platforms, themes, crates, or folders.\n\
        Existing files are never overwritten.\n\
        \n\
        Examples:\n  \
        rulestools update . --add-platform mobile\n  \
        rulestools update . --add-theme macos --preview\n  \
        rulestools update . --add-crate mcp --add-folder doc")]
    Update {
        /// Path to the project root.
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Add platforms (comma-separated): desktop, mobile, small.
        #[arg(long, default_value = "")]
        add_platform: String,

        /// Add themes (comma-separated), e.g. win3ui-fluent,macos.
        #[arg(long, default_value = "")]
        add_theme: String,

        /// Add a workspace crate by name.
        #[arg(long)]
        add_crate: Option<String>,

        /// Add folders (comma-separated): lib, shared, doc.
        #[arg(long, default_value = "")]
        add_folder: String,

        /// Preview mode — show what would be created without writing.
        #[arg(long)]
        preview: bool,

        /// Output format: text or json.
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Upgrade a project to a higher kind (structural transformation).
    #[command(long_about = "Upgrade a project to a higher kind.\n\n\
        Changes ProjectKind upward (never down). Scaffolds new structure\n\
        and provides move guidance for existing files.\n\
        \n\
        Upgrade order: tool < library/website < cli < slint-app < workspace\n\
        \n\
        Examples:\n  \
        rulestools upgrade . --to cli\n  \
        rulestools upgrade . --to slint-app --preview\n  \
        rulestools upgrade . --to workspace --format json")]
    Upgrade {
        /// Path to the project root.
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Target kind to upgrade to.
        #[arg(long)]
        to: String,

        /// Preview mode — show what would change without writing.
        #[arg(long)]
        preview: bool,

        /// Output format: text or json.
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Publish, distribute, and sync projects.
    #[command(subcommand)]
    Publish(PublishCmd),

    /// Report, list, or close issues on Forgejo.
    #[command(subcommand)]
    Issue(IssueCmd),
}

#[derive(Subcommand)]
enum PublishCmd {
    /// Analyze project and show publish targets, version, pre-checks.
    #[command(long_about = "Analyze project and show publish targets, version, pre-checks.\n\n\
        Reads [publish] config from proj/rulestools.toml, checks git state,\n\
        scanner status, and lists configured targets with version info.\n\
        \n\
        Examples:\n  \
        rulestools publish plan .                # show publish plan\n  \
        rulestools publish plan . --format json  # JSON output for MCP")]
    Plan {
        /// Path to the project root.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output format: text or json.
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Execute publish to a target (github/forgejo/archive).
    #[command(long_about = "Execute publish to a specific target.\n\n\
        Runs pre-publish checks (scanner, tests, clean git tree, token),\n\
        then builds, tags, and creates a release on the target.\n\
        Use --preview to see checks without publishing.\n\
        \n\
        Examples:\n  \
        rulestools publish run . --target github           # publish to GitHub\n  \
        rulestools publish run . --target archive          # create local archive\n  \
        rulestools publish run . --target github --preview # dry run")]
    Run {
        /// Path to the project root.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Target: github, forgejo, archive.
        #[arg(long)]
        target: String,
        /// Preview mode — run checks without publishing.
        #[arg(long)]
        preview: bool,
    },

    /// Show what is published where.
    #[command(long_about = "Show published versions for each configured target.\n\n\
        Queries GitHub/Forgejo APIs for latest release info.\n\
        Shows version, date, and URL per target.\n\
        \n\
        Examples:\n  \
        rulestools publish status .                # show status\n  \
        rulestools publish status . --format json  # JSON output")]
    Status {
        /// Path to the project root.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output format: text or json.
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Initialize a pub-repo and configure sync.
    #[command(long_about = "Initialize a pub-repo for dev/pub separation.\n\n\
        Creates ../{name}-pub/ directory, initializes git, adds remote,\n\
        and writes [publish.repo] config to proj/rulestools.toml.\n\
        Runs initial sync in preview mode.\n\
        \n\
        Examples:\n  \
        rulestools publish init . --remote git@github.com:user/repo.git")]
    Init {
        /// Path to the project root.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Remote URL for the pub-repo.
        #[arg(long)]
        remote: String,
    },

    /// Sync dev-repo to pub-repo (whitelist copy).
    #[command(long_about = "Sync files from dev-repo to pub-repo based on include/exclude config.\n\n\
        Only whitelisted files/dirs are copied. Excluded patterns are NEVER copied.\n\
        Hardcoded safety: proj/, .claude/, target/, .git/, .env*, *.key always excluded.\n\
        \n\
        Examples:\n  \
        rulestools publish sync .            # sync now\n  \
        rulestools publish sync . --preview  # show what would be synced")]
    Sync {
        /// Path to the project root.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Preview mode — show what would be synced without writing.
        #[arg(long)]
        preview: bool,
    },

    /// Validate pub-repo for leaks and sync status.
    #[command(long_about = "Check pub-repo for leaked files and sync status.\n\n\
        Walks all files in pub-repo and checks:\n  \
        - Leaked: files that should be excluded but are present\n  \
        - Out-of-sync: files that differ from dev-repo\n\
        \n\
        Examples:\n  \
        rulestools publish check .")]
    Check {
        /// Path to the project root.
        #[arg(default_value = ".")]
        path: PathBuf,
    },
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
    /// Add a label to an existing issue.
    #[command(name = "add-label")]
    AddLabel {
        /// Issue number.
        number: u64,
        /// Label name to add.
        label: String,
    },
    /// Create a new label in the Forgejo repo.
    #[command(name = "create-label")]
    CreateLabel {
        /// Label name.
        name: String,
        /// Label color (hex without #, e.g. "e11d48").
        #[arg(long, default_value = "0075ca")]
        color: String,
        /// Label description.
        #[arg(long, default_value = "")]
        description: String,
    },
    /// List all available labels in the Forgejo repo.
    #[command(name = "list-labels")]
    ListLabels,
    /// Add a comment to an existing issue.
    Comment {
        /// Issue number.
        number: u64,
        /// Comment body.
        body: String,
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
        Commands::Init { path, kind, name } => cmd_init(&path, &kind, name.as_deref()),
        Commands::New {
            path, kind, name, platforms, themes, mcp, extras, preview, format,
        } => cmd_new(&path, &kind, name.as_deref(), &platforms, &themes, mcp, &extras, preview, &format),
        Commands::Update {
            path, add_platform, add_theme, add_crate, add_folder, preview, format,
        } => cmd_update(&path, &add_platform, &add_theme, add_crate.as_deref(), &add_folder, preview, &format),
        Commands::Upgrade {
            path, to, preview, format,
        } => cmd_upgrade(&path, &to, preview, &format),
        Commands::Publish(cmd) => cmd_publish(cmd),
        Commands::Issue(cmd) => cmd_issue(cmd),
    }
}

fn cmd_init(path: &PathBuf, kind_str: &str, name: Option<&str>) {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());

    let kind = match rulestools_scanner::project::ProjectKind::from_str(kind_str) {
        Some(k) => k,
        None => {
            eprintln!(
                "Unknown kind: {kind_str}\nValid kinds: tool, cli, library, website, slint-app, workspace"
            );
            std::process::exit(1);
        }
    };

    // Map "workspace" input to Super kind (workspace = scaffold, super = scan behavior)
    let project_name = name
        .map(String::from)
        .unwrap_or_else(|| {
            root.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project")
                .to_string()
        });

    match scaffold::scaffold_project(&root, kind, &project_name) {
        Ok(summary) => {
            println!("{summary}");
            println!();
            cmd_detect(path);
        }
        Err(e) => {
            eprintln!("Scaffold failed: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_new(
    path: &PathBuf,
    kind_str: &str,
    name: Option<&str>,
    platforms_str: &str,
    themes_str: &str,
    mcp: bool,
    extras_str: &str,
    preview: bool,
    format: &str,
) {
    let root = if path.exists() {
        std::fs::canonicalize(path).unwrap_or_else(|_| path.clone())
    } else {
        std::fs::create_dir_all(path).unwrap_or_else(|e| {
            eprintln!("Cannot create directory: {e}");
            std::process::exit(1);
        });
        std::fs::canonicalize(path).unwrap_or_else(|_| path.clone())
    };

    let kind = match rulestools_scanner::project::ProjectKind::from_str(kind_str) {
        Some(k) => k,
        None => {
            eprintln!(
                "Unknown kind: {kind_str}\nValid kinds: tool, cli, library, website, slint-app, workspace"
            );
            std::process::exit(1);
        }
    };

    let project_name = name
        .map(String::from)
        .unwrap_or_else(|| {
            root.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project")
                .to_string()
        });

    let platforms: Vec<scaffold::Platform> = platforms_str
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .filter_map(|s| scaffold::Platform::from_str(s))
        .collect();

    let themes: Vec<String> = themes_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let extras: Vec<scaffold::Extra> = extras_str
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .filter_map(|s| scaffold::Extra::from_str(s))
        .collect();

    let opts = scaffold::ScaffoldOptions {
        name: project_name.clone(),
        kind,
        platforms,
        themes,
        mcp,
        extras,
        preview,
    };

    match scaffold::scaffold_with_options(&root, &opts) {
        Ok(result) => {
            if format == "json" {
                let tree = scaffold::render_tree(&project_name, &result.created);
                let json = serde_json::json!({
                    "name": project_name,
                    "kind": kind_str,
                    "preview": preview,
                    "created": result.created,
                    "skipped": result.skipped,
                    "tree": tree,
                });
                println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
            } else {
                println!("{}", result.summary);
                for path in &result.created {
                    println!("  {path}");
                }
                if !result.skipped.is_empty() {
                    println!("\nSkipped:");
                    for s in &result.skipped {
                        println!("  {s}");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Scaffold failed: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_update(
    path: &PathBuf,
    platforms_str: &str,
    themes_str: &str,
    crate_name: Option<&str>,
    folders_str: &str,
    preview: bool,
    format: &str,
) {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());

    let platforms: Vec<scaffold::Platform> = platforms_str
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .filter_map(|s| scaffold::Platform::from_str(s))
        .collect();

    let themes: Vec<String> = themes_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let folders: Vec<scaffold::Extra> = folders_str
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .filter_map(|s| scaffold::Extra::from_str(s))
        .collect();

    let opts = scaffold::UpdateOptions {
        platforms,
        themes,
        crate_name: crate_name.map(String::from),
        folders,
        preview,
    };

    match scaffold::update_project(&root, &opts) {
        Ok(result) => {
            if format == "json" {
                let json = serde_json::json!({
                    "preview": preview,
                    "created": result.created,
                    "skipped": result.skipped,
                });
                println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
            } else {
                println!("{}", result.summary);
                for path in &result.created {
                    println!("  {path}");
                }
                if !result.skipped.is_empty() {
                    println!("\nSkipped:");
                    for s in &result.skipped {
                        println!("  {s}");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Update failed: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_upgrade(path: &PathBuf, to_str: &str, preview: bool, format: &str) {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());

    let to_kind = match rulestools_scanner::project::ProjectKind::from_str(to_str) {
        Some(k) => k,
        None => {
            eprintln!(
                "Unknown kind: {to_str}\nValid kinds: tool, cli, library, website, slint-app, workspace"
            );
            std::process::exit(1);
        }
    };

    match scaffold::upgrade_project(&root, to_kind, preview) {
        Ok(result) => {
            if format == "json" {
                let guidance: Vec<serde_json::Value> = result
                    .move_guidance
                    .iter()
                    .map(|g| {
                        serde_json::json!({
                            "from": g.from,
                            "to": g.to,
                            "reason": g.reason,
                        })
                    })
                    .collect();
                let json = serde_json::json!({
                    "from": result.from_kind.as_str(),
                    "to": result.to_kind.as_str(),
                    "preview": preview,
                    "created": result.created,
                    "move_guidance": guidance,
                    "manual_steps": result.manual_steps,
                });
                println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
            } else {
                let label = if preview { "Preview" } else { "Upgraded" };
                println!(
                    "{}: {:?} -> {:?}",
                    label, result.from_kind, result.to_kind
                );

                if !result.created.is_empty() {
                    println!("\nCreated:");
                    for path in &result.created {
                        println!("  {path}");
                    }
                }

                if !result.move_guidance.is_empty() {
                    println!("\nMove guidance:");
                    for g in &result.move_guidance {
                        println!("  {} -> {}", g.from, g.to);
                        println!("    {}", g.reason);
                    }
                }

                if !result.manual_steps.is_empty() {
                    println!("\nManual steps:");
                    for step in &result.manual_steps {
                        println!("  - {step}");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Upgrade failed: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_publish(cmd: PublishCmd) {
    match cmd {
        PublishCmd::Plan { path, format } => {
            let root = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            match publish::publish_plan(&root, &format) {
                Ok(output) => print!("{output}"),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
        PublishCmd::Run { path, target, preview } => {
            let root = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            match publish::publish_run(&root, &target, preview) {
                Ok(output) => println!("{output}"),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
        PublishCmd::Status { path, format } => {
            let root = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            match publish::publish_status(&root, &format) {
                Ok(output) => print!("{output}"),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
        PublishCmd::Init { path, remote } => {
            let root = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            match publish::publish_init(&root, &remote) {
                Ok(output) => println!("{output}"),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
        PublishCmd::Sync { path, preview } => {
            let root = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            match publish::publish_sync(&root, preview) {
                Ok(output) => print!("{output}"),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
        PublishCmd::Check { path } => {
            let root = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            match publish::publish_check(&root) {
                Ok(output) => print!("{output}"),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
    }
}

fn cmd_scan(path: &PathBuf, deny: bool) {
    let root = std::fs::canonicalize(path).unwrap_or_else(|_| path.clone());
    let identity = rulestools_scanner::project::ProjectIdentity::detect(&root);

    println!("rulestools: {:?} / {:?}", identity.kind, identity.layout);

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
        IssueCmd::AddLabel { number, label } => cmd_issue_add_label(number, &label),
        IssueCmd::CreateLabel { name, color, description } => cmd_issue_create_label(&name, &color, &description),
        IssueCmd::ListLabels => cmd_issue_list_labels(),
        IssueCmd::Comment { number, body } => cmd_issue_comment(number, &body),
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

fn cmd_issue_add_label(number: u64, label: &str) {
    let token = match forgejo_token() {
        Ok(t) => t,
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    };

    let label_ids = match resolve_label_ids(&token, &[label]) {
        Ok(ids) if !ids.is_empty() => ids,
        Ok(_) => { eprintln!("Label not found: {label}"); std::process::exit(1); }
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    };

    let payload = serde_json::json!({ "labels": label_ids });
    match ureq::post(&format!("{FORGEJO_API}/issues/{number}/labels"))
        .set("Authorization", &format!("token {token}"))
        .set("Content-Type", "application/json")
        .send_string(&payload.to_string())
    {
        Ok(_) => println!("Label '{label}' added to issue #{number}"),
        Err(e) => { eprintln!("Failed to add label: {e}"); std::process::exit(1); }
    }
}

fn cmd_issue_create_label(name: &str, color: &str, description: &str) {
    let token = match forgejo_token() {
        Ok(t) => t,
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    };

    let payload = serde_json::json!({
        "name": name,
        "color": format!("#{color}"),
        "description": description,
    });

    match ureq::post(&format!("{FORGEJO_API}/labels"))
        .set("Authorization", &format!("token {token}"))
        .set("Content-Type", "application/json")
        .send_string(&payload.to_string())
    {
        Ok(resp) => {
            if let Ok(json) = resp.into_string().and_then(|s| Ok(serde_json::from_str::<serde_json::Value>(&s).unwrap_or_default())) {
                let id = json.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                println!("Label '{name}' created (id: {id})");
            }
        }
        Err(e) => { eprintln!("Failed to create label: {e}"); std::process::exit(1); }
    }
}

fn cmd_issue_list_labels() {
    let token = match forgejo_token() {
        Ok(t) => t,
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    };

    match ureq::get(&format!("{FORGEJO_API}/labels?limit=50"))
        .set("Authorization", &format!("token {token}"))
        .call()
    {
        Ok(resp) => {
            if let Ok(body) = resp.into_string() {
                if let Ok(labels) = serde_json::from_str::<Vec<serde_json::Value>>(&body) {
                    if labels.is_empty() {
                        println!("No labels found");
                        return;
                    }
                    for label in &labels {
                        let name = label.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                        let color = label.get("color").and_then(|v| v.as_str()).unwrap_or("");
                        let desc = label.get("description").and_then(|v| v.as_str()).unwrap_or("");
                        if desc.is_empty() {
                            println!("{name}  ({color})");
                        } else {
                            println!("{name}  ({color}) — {desc}");
                        }
                    }
                }
            }
        }
        Err(e) => { eprintln!("Failed to list labels: {e}"); std::process::exit(1); }
    }
}

fn cmd_issue_comment(number: u64, body: &str) {
    let token = match forgejo_token() {
        Ok(t) => t,
        Err(e) => { eprintln!("{e}"); std::process::exit(1); }
    };

    let payload = serde_json::json!({ "body": body });
    match ureq::post(&format!("{FORGEJO_API}/issues/{number}/comments"))
        .set("Authorization", &format!("token {token}"))
        .set("Content-Type", "application/json")
        .send_string(&payload.to_string())
    {
        Ok(_) => println!("Comment added to issue #{number}"),
        Err(e) => { eprintln!("Failed to add comment: {e}"); std::process::exit(1); }
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
