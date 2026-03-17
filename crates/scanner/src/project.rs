use std::path::Path;

/// What kind of project is this — determines which checks apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    /// Rust + Slint GUI app — full check-set (gateway, mother-child, tokens, UI).
    SlintApp,
    /// Rust CLI binary — no gateway/mother-child/UI checks.
    CliApp,
    /// Standalone reusable crate — grows organically, topology when needed.
    Library,
    /// Scripts and tooling within a project — most relaxed.
    Tool,
    /// Multi-repo superprojekt — contains multiple sub-repos, each with own config.
    Super,
}

/// Project layout — single crate or Cargo workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// Single `src/` crate.
    Single,
    /// Cargo workspace with `crates/`/`apps/` members.
    Workspace,
}

/// Auto-detected project identity.
#[derive(Debug, Clone)]
pub struct ProjectIdentity {
    pub kind: ProjectKind,
    pub layout: Layout,
}

impl ProjectIdentity {
    /// Detect project kind and layout. Explicit `[project].kind` in
    /// `proj/rulestools.toml` overrides auto-detection.
    pub fn detect(root: &Path) -> Self {
        let layout = detect_layout(root);
        let kind = read_explicit_kind(root).unwrap_or_else(|| detect_kind(root));
        Self { kind, layout }
    }

    /// Check if project has explicit registration (proj/rulestools.toml with [project].kind).
    pub fn is_registered(root: &Path) -> bool {
        read_explicit_kind(root).is_some()
    }

    /// Suggest the best matching ProjectKind for an unregistered project.
    pub fn suggest(root: &Path) -> String {
        let kind = detect_kind(root);
        let layout = detect_layout(root);
        let kind_str = match kind {
            ProjectKind::SlintApp => "slint-app",
            ProjectKind::CliApp => "cli",
            ProjectKind::Library => "library",
            ProjectKind::Tool => "tool",
            ProjectKind::Super => "super",
        };
        format!(
            "Project looks like {} ({}). Register with:\n  [project]\n  kind = \"{}\"",
            kind_str,
            match layout { Layout::Workspace => "workspace", Layout::Single => "single crate" },
            kind_str,
        )
    }
}

/// Detect workspace vs single crate from Cargo.toml.
fn detect_layout(root: &Path) -> Layout {
    let cargo_toml = root.join("Cargo.toml");
    if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
        if content.contains("[workspace]") {
            return Layout::Workspace;
        }
    }
    Layout::Single
}

/// Read explicit kind from `proj/rulestools.toml` `[project].kind`.
fn read_explicit_kind(root: &Path) -> Option<ProjectKind> {
    let config_path = root.join("proj").join("rulestools.toml");
    let content = std::fs::read_to_string(&config_path).ok()?;
    let table: toml::Table = content.parse().ok()?;
    let kind_str = table.get("project")?.as_table()?.get("kind")?.as_str()?;
    ProjectKind::from_str(kind_str)
}

/// Detect project kind from filesystem signals.
fn detect_kind(root: &Path) -> ProjectKind {
    if has_slint_files(root) {
        return ProjectKind::SlintApp;
    }
    if has_binary_entry(root) {
        return ProjectKind::CliApp;
    }
    ProjectKind::Library
}

/// Check for .slint files in ui/ directory or slint-build in Cargo.toml.
fn has_slint_files(root: &Path) -> bool {
    let ui_dir = root.join("ui");
    if ui_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&ui_dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "slint" {
                        return true;
                    }
                }
            }
        }
    }
    let cargo_toml = root.join("Cargo.toml");
    if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
        if content.contains("slint-build") || content.contains("slintscanners") {
            return true;
        }
    }
    false
}

/// Check for src/main.rs or [[bin]] in Cargo.toml.
fn has_binary_entry(root: &Path) -> bool {
    if root.join("src").join("main.rs").exists() {
        return true;
    }
    let cargo_toml = root.join("Cargo.toml");
    if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
        if content.contains("[[bin]]") {
            return true;
        }
    }
    false
}

impl ProjectKind {
    /// Parse from string (for `proj/rulestools.toml` `[project].kind`).
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "slint-app" | "slint_app" => Some(ProjectKind::SlintApp),
            "cli" | "cli-app" | "cli_app" => Some(ProjectKind::CliApp),
            "library" | "lib" => Some(ProjectKind::Library),
            "tool" => Some(ProjectKind::Tool),
            "super" | "super-project" => Some(ProjectKind::Super),
            _ => None,
        }
    }

    /// Check categories that are skipped for this project kind.
    pub fn skipped_categories(&self) -> &'static [&'static str] {
        match self {
            ProjectKind::SlintApp => &[],
            ProjectKind::CliApp => &[
                "gateway/",
                "uiux/",
                "slint/",
            ],
            ProjectKind::Library => &[
                "gateway/",
                "uiux/",
                "slint/",
                "rust/types/no-println",
            ],
            ProjectKind::Tool => &[
                "gateway/",
                "uiux/",
                "slint/",
                "rust/types/no-println",
                "rust/constants/no-hardcoded-path",
            ],
            ProjectKind::Super => &[], // super delegates to sub-repos
        }
    }

    /// Whether a check ID should run for this project kind.
    pub fn allows_check(&self, check_id: &str) -> bool {
        for prefix in self.skipped_categories() {
            if check_id.starts_with(prefix) || check_id == *prefix {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slint_app_allows_all() {
        let kind = ProjectKind::SlintApp;
        assert!(kind.allows_check("gateway/io/layer-violation"));
        assert!(kind.allows_check("uiux/mother-child/extract-orchestrator"));
        assert!(kind.allows_check("rust/errors/no-unwrap"));
        assert!(kind.allows_check("slint/tokens"));
    }

    #[test]
    fn cli_app_skips_gateway_and_ui() {
        let kind = ProjectKind::CliApp;
        assert!(!kind.allows_check("gateway/io/layer-violation"));
        assert!(!kind.allows_check("uiux/mother-child/extract-orchestrator"));
        assert!(!kind.allows_check("slint/tokens"));
        assert!(kind.allows_check("rust/errors/no-unwrap"));
        assert!(kind.allows_check("rust/naming/no-noise-names"));
    }

    #[test]
    fn library_skips_println_and_topology() {
        let kind = ProjectKind::Library;
        assert!(!kind.allows_check("gateway/io/layer-violation"));
        assert!(!kind.allows_check("uiux/mother-child/extract-orchestrator"));
        assert!(!kind.allows_check("rust/types/no-println"));
        assert!(kind.allows_check("rust/errors/no-unwrap"));
        assert!(kind.allows_check("rust/constants/no-magic-number"));
        assert!(kind.allows_check("rust/constants/no-hardcoded-path"));
    }

    #[test]
    fn tool_most_relaxed() {
        let kind = ProjectKind::Tool;
        assert!(!kind.allows_check("gateway/io/layer-violation"));
        assert!(!kind.allows_check("rust/types/no-println"));
        assert!(!kind.allows_check("rust/constants/no-hardcoded-path"));
        assert!(kind.allows_check("rust/errors/no-unwrap"));
        assert!(kind.allows_check("rust/constants/no-magic-number"));
    }

    #[test]
    fn from_str_parsing() {
        assert_eq!(ProjectKind::from_str("slint-app"), Some(ProjectKind::SlintApp));
        assert_eq!(ProjectKind::from_str("cli"), Some(ProjectKind::CliApp));
        assert_eq!(ProjectKind::from_str("library"), Some(ProjectKind::Library));
        assert_eq!(ProjectKind::from_str("lib"), Some(ProjectKind::Library));
        assert_eq!(ProjectKind::from_str("tool"), Some(ProjectKind::Tool));
        assert_eq!(ProjectKind::from_str("super"), Some(ProjectKind::Super));
        assert_eq!(ProjectKind::from_str("unknown"), None);
    }

    #[test]
    fn skipped_categories_count() {
        assert_eq!(ProjectKind::SlintApp.skipped_categories().len(), 0);
        assert_eq!(ProjectKind::CliApp.skipped_categories().len(), 3);
        assert_eq!(ProjectKind::Library.skipped_categories().len(), 4);
        assert_eq!(ProjectKind::Tool.skipped_categories().len(), 5);
    }
}
