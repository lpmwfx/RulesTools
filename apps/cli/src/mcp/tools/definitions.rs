use crate::mcp::ToolDef;

/// fn `all`.
pub fn all() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "scan_file".into(),
            description: "Scan a single source file for rule violations.\n\nCall this after every Edit or Write to verify compliance.\nReturns CLEAN or a list of violations to fix.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to the file to scan" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "scan_tree".into(),
            description: "Scan an entire project tree for rule violations.\n\nWalks all source files, runs enabled checks, writes proj/ISSUES.\nReturns grouped output with guidance and decision trees.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "check_staged".into(),
            description: "Pre-commit check — scan staged files and report violations.\n\nDesigned for pre-commit hooks. Returns CLEAN or violation list.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to git repo root" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "setup".into(),
            description: "Install FULL RulesTools integration.\n\nCreates proj/rulestools.toml, adds rulestools-scanner to Cargo.toml [build-dependencies], creates/updates build.rs with scan_project(), installs pre-commit hook, and installs .claude/settings.json PostToolUse hook.\nIdempotent — safe to run every session.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "security_scan".into(),
            description: "Security-focused scan — checks for secrets, injection, unsafe patterns.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "init_project".into(),
            description: "Initialize a new project with full scaffolding.\n\n\
                Creates directory structure, stub source files, Cargo.toml, proj/ files, and .gitignore.\n\
                Kinds: tool, cli, library, slint-app, workspace.\n\
                Existing files are never overwritten.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" },
                    "kind": {
                        "type": "string",
                        "description": "Project kind: tool, cli, library, slint-app, workspace",
                        "enum": ["tool", "cli", "library", "slint-app", "workspace"]
                    },
                    "name": { "type": "string", "description": "Project name (default: directory name)" }
                },
                "required": ["path", "kind"]
            }),
        },
        ToolDef {
            name: "new_project".into(),
            description: "Create a new project with full scaffolding and options.\n\n\
                Creates directory structure, stub source files, Cargo.toml, proj/ files.\n\
                Supports platforms, themes, MCP crate, extras, and preview mode.\n\
                Kinds: tool, cli, library, website, slint-app, workspace.\n\
                Existing files are never overwritten.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" },
                    "kind": {
                        "type": "string",
                        "description": "Project kind",
                        "enum": ["tool", "cli", "library", "website", "slint-app", "workspace"]
                    },
                    "name": { "type": "string", "description": "Project name (default: directory name)" },
                    "platforms": {
                        "type": "array",
                        "items": { "type": "string", "enum": ["desktop", "mobile", "small"] },
                        "description": "Target platforms (SlintApp/Super only)"
                    },
                    "themes": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Theme names (SlintApp/Super only)"
                    },
                    "mcp": { "type": "boolean", "description": "Add MCP server crate (workspace only)" },
                    "extras": {
                        "type": "array",
                        "items": { "type": "string", "enum": ["lib", "shared", "doc"] },
                        "description": "Extra folders/crates to add"
                    },
                    "preview": { "type": "boolean", "description": "Preview only — show what would be created" }
                },
                "required": ["path", "kind"]
            }),
        },
        ToolDef {
            name: "update_project".into(),
            description: "Add features to an existing project without changing its kind.\n\n\
                Detects current project kind and adds platforms, themes, crates, or folders.\n\
                Existing files are never overwritten. Kind is never changed.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" },
                    "platforms": {
                        "type": "array",
                        "items": { "type": "string", "enum": ["desktop", "mobile", "small"] },
                        "description": "Platforms to add (SlintApp/Super only)"
                    },
                    "themes": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Themes to add (SlintApp/Super only)"
                    },
                    "crate_name": { "type": "string", "description": "Crate name to add (workspace only)" },
                    "folders": {
                        "type": "array",
                        "items": { "type": "string", "enum": ["lib", "shared", "doc"] },
                        "description": "Extra folders to add"
                    },
                    "preview": { "type": "boolean", "description": "Preview only" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "upgrade_project".into(),
            description: "Upgrade a project to a higher kind (structural transformation).\n\n\
                Changes ProjectKind upward (never down). Scaffolds new structure\n\
                and provides move guidance for existing files.\n\
                Order: tool < library/website < cli < slint-app < workspace".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" },
                    "to": {
                        "type": "string",
                        "description": "Target kind to upgrade to",
                        "enum": ["tool", "cli", "library", "website", "slint-app", "workspace"]
                    },
                    "preview": { "type": "boolean", "description": "Preview only" }
                },
                "required": ["path", "to"]
            }),
        },
        ToolDef {
            name: "publish_plan".into(),
            description: "Analyze project and show publish targets, version, and pre-checks.\n\n\
                Reads [publish] config, checks git state, scanner status.\n\
                Returns targets with version info and changelog.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "publish_run".into(),
            description: "Execute publish to a target (github/forgejo/archive).\n\n\
                Runs pre-publish checks (scanner, tests, clean git, token),\n\
                then builds, tags, and creates release. Use preview for dry run.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" },
                    "target": {
                        "type": "string",
                        "description": "Publish target",
                        "enum": ["github", "forgejo", "archive"]
                    },
                    "preview": { "type": "boolean", "description": "Preview only — run checks without publishing" }
                },
                "required": ["path", "target"]
            }),
        },
        ToolDef {
            name: "publish_status".into(),
            description: "Show what is published where.\n\n\
                Queries GitHub/Forgejo APIs for latest release info.\n\
                Shows version, date, and URL per configured target.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "publish_init".into(),
            description: "Initialize a pub-repo for dev/pub separation.\n\n\
                Creates ../{name}-pub/ directory, git init, adds remote,\n\
                writes [publish.repo] config to proj/rulestools.toml.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" },
                    "remote": { "type": "string", "description": "Remote URL for the pub-repo (e.g. git@github.com:user/repo.git)" }
                },
                "required": ["path", "remote"]
            }),
        },
        ToolDef {
            name: "publish_sync".into(),
            description: "Sync dev-repo to pub-repo (whitelist copy).\n\n\
                Only whitelisted files/dirs are copied. Excluded patterns never copied.\n\
                Hardcoded safety: proj/, .claude/, target/, .env*, *.key always excluded.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" },
                    "preview": { "type": "boolean", "description": "Preview only — show what would be synced" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "generate_docs".into(),
            description: "Report documentation coverage for a project.\n\n\
                Counts pub items and checks for /// doc comments.\n\
                Returns coverage stats (items, undocumented count, percentage).".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" }
                },
                "required": ["path"]
            }),
        },
        ToolDef {
            name: "publish_check".into(),
            description: "Validate pub-repo for leaks and sync status.\n\n\
                Walks pub-repo files and reports:\n\
                - LEAKED: excluded files found in pub-repo\n\
                - OUT-OF-SYNC: files that differ from dev-repo\n\
                - CLEAN: all checks pass".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to project root" }
                },
                "required": ["path"]
            }),
        },
    ]
}
