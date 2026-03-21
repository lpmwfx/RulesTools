use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod mcp;
/// mod `publish`.
pub mod publish;
/// mod `scaffold`.
pub mod scaffold;

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

    /// Install full RulesTools integration for a project.
    #[command(long_about = "Install full RulesTools integration for a project.\n\n\
        Performs all setup steps:\n  \
        1. Creates proj/ directory\n  \
        2. Creates proj/rulestools.toml with auto-detected kind\n  \
        3. Adds rulestools-scanner to Cargo.toml [build-dependencies]\n  \
        4. Creates/updates build.rs with scan_project()\n  \
        5. Installs .git/hooks/pre-commit\n  \
        6. Installs .claude/settings.json PostToolUse hook\n\n\
        Idempotent — safe to run every session. Skips steps already done.\n\
        \n\
        Examples:\n  \
        rulestools setup .                  # setup current directory\n  \
        rulestools setup /path/to/project   # setup specific project")]
    Setup {
        /// Path to the project root.
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Publish, distribute, and sync projects.
    #[command(subcommand)]
    Publish(PublishCmd),

    /// Report, list, or close issues on Forgejo.
    #[command(subcommand)]
    Issue(IssueCmd),

    /// Start MCP tools server (scan, setup, init, publish — stdio).
    #[command(name = "mcp-tools")]
    McpTools,

    /// Start MCP rules server (rule lookup, search — stdio).
    #[command(name = "mcp-rules")]
    McpRules,

    /// PostToolUse hook — scan file after Edit/Write (reads JSON from stdin).
    #[command(long_about = "PostToolUse hook — scan file after Edit/Write.\n\n\
        Reads tool invocation JSON from stdin, extracts file_path,\n\
        scans the file for rule violations, and prints results to stderr.\n\
        Always exits 0 — advisory only, never blocks edits.\n\
        \n\
        Installed automatically by `rulestools setup` in .claude/settings.json.\n\
        \n\
        Example:\n  \
        echo '{\"tool_name\":\"Edit\",\"tool_input\":{\"file_path\":\"src/main.rs\"}}' | rulestools hook")]
    Hook,
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
/// enum `IssueCmd`.
pub enum IssueCmd {
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
        Commands::Scan { path, deny } => commands::scan::cmd_scan(&path, deny),
        Commands::Check { path } => commands::scan::cmd_check(&path),
        Commands::List { path } => commands::project::cmd_list(&path),
        Commands::Detect { path } => commands::project::cmd_detect(&path),
        Commands::Gen { path } => commands::generate::cmd_gen(&path),
        Commands::ScanFile { file, format } => commands::scan::cmd_scan_file(&file, &format),
        Commands::Init { path, kind, name } => commands::project::cmd_init(&path, &kind, name.as_deref()),
        Commands::New {
            path, kind, name, platforms, themes, mcp, extras, preview, format,
        } => commands::project::cmd_new(&path, &kind, name.as_deref(), &platforms, &themes, mcp, &extras, preview, &format),
        Commands::Update {
            path, add_platform, add_theme, add_crate, add_folder, preview, format,
        } => commands::project::cmd_update(&path, &add_platform, &add_theme, add_crate.as_deref(), &add_folder, preview, &format),
        Commands::Upgrade {
            path, to, preview, format,
        } => commands::project::cmd_upgrade(&path, &to, preview, &format),
        Commands::Setup { path } => commands::project::cmd_setup(&path),
        Commands::Publish(cmd) => cmd_publish(cmd),
        Commands::Issue(cmd) => commands::issue::cmd_issue(cmd),
        Commands::McpTools => mcp::tools::run(),
        Commands::McpRules => mcp::rules::run(),
        Commands::Hook => commands::hook::cmd_hook(),
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
