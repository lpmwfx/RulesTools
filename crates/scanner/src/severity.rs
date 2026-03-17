use std::collections::HashMap;

use crate::issue::Severity;
use crate::project::ProjectKind;

/// Maps check IDs to severity based on ProjectKind.
///
/// Same checks, different enforcement per project kind.
/// Tool = most relaxed, SlintApp = full enforcement.
pub struct SeverityResolver {
    overrides: HashMap<String, Severity>,
}

impl SeverityResolver {
    /// Build resolver for a given ProjectKind.
    pub fn for_kind(kind: ProjectKind) -> Self {
        let mut overrides = HashMap::new();

        match kind {
            ProjectKind::Tool => {
                // Most relaxed — skip everything except secrets
                for id in TOOL_SKIP {
                    overrides.insert(id.to_string(), Severity::Skip);
                }
            }
            ProjectKind::CliApp | ProjectKind::Library => {
                // Warn on code quality, skip topology
                for id in CLI_WARN {
                    overrides.insert(id.to_string(), Severity::Warning);
                }
                for id in CLI_SKIP {
                    overrides.insert(id.to_string(), Severity::Skip);
                }
            }
            ProjectKind::SlintApp | ProjectKind::Super => {
                // Full enforcement — Error is default, no overrides needed
            }
        }

        Self { overrides }
    }

    /// Resolve the final severity for a check.
    pub fn resolve(&self, check_id: &str, default: Severity) -> Severity {
        // secrets always error — cannot be overridden
        if check_id == "global/secrets" {
            return Severity::Error;
        }
        self.overrides.get(check_id).copied().unwrap_or(default)
    }
}

const TOOL_SKIP: &[&str] = &[
    "rust/constants/no-magic-number",
    "rust/constants/no-hardcoded-path",
    "rust/constants/no-hardcoded-duration",
    "rust/constants/no-hardcoded-url",
    "rust/errors/no-unwrap",
    "rust/docs/doc-required",
    "rust/types/no-string-match",
    "rust/naming/no-noise-names",
    "rust/modules/no-utils",
    "rust/modules/no-sibling-coupling",
    "rust/modules/shared-guard",
    "rust/ownership/clone-spam",
    "rust/threading/no-static-mut",
    "rust/safety/unsafe-needs-comment",
    "uiux/mother-child/mother-too-many-fns",
    "topology/layer-violation",
    "topology/placement",
    "topology/naming",
    "global/nesting",
    "global/file-limits",
    "global/tech-debt",
];

const CLI_WARN: &[&str] = &[
    "rust/constants/no-magic-number",
    "rust/constants/no-hardcoded-path",
    "rust/constants/no-hardcoded-duration",
    "rust/constants/no-hardcoded-url",
    "rust/errors/no-unwrap",
    "rust/docs/doc-required",
    "rust/types/no-string-match",
    "rust/naming/no-noise-names",
    "rust/modules/no-utils",
    "rust/modules/no-sibling-coupling",
    "rust/modules/shared-guard",
    "rust/ownership/clone-spam",
    "uiux/mother-child/mother-too-many-fns",
    "topology/placement",
    "global/nesting",
    "global/file-limits",
    "global/tech-debt",
];

const CLI_SKIP: &[&str] = &[
    "topology/layer-violation",
    "topology/naming",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_skips_most() {
        let r = SeverityResolver::for_kind(ProjectKind::Tool);
        assert_eq!(r.resolve("rust/errors/no-unwrap", Severity::Error), Severity::Skip);
        assert_eq!(r.resolve("global/nesting", Severity::Error), Severity::Skip);
        assert_eq!(r.resolve("topology/layer-violation", Severity::Error), Severity::Skip);
    }

    #[test]
    fn tool_keeps_secrets() {
        let r = SeverityResolver::for_kind(ProjectKind::Tool);
        assert_eq!(r.resolve("global/secrets", Severity::Error), Severity::Error);
    }

    #[test]
    fn cli_warns() {
        let r = SeverityResolver::for_kind(ProjectKind::CliApp);
        assert_eq!(r.resolve("rust/errors/no-unwrap", Severity::Error), Severity::Warning);
        assert_eq!(r.resolve("global/nesting", Severity::Error), Severity::Warning);
    }

    #[test]
    fn cli_skips_topology() {
        let r = SeverityResolver::for_kind(ProjectKind::CliApp);
        assert_eq!(r.resolve("topology/layer-violation", Severity::Error), Severity::Skip);
    }

    #[test]
    fn slint_app_full_enforcement() {
        let r = SeverityResolver::for_kind(ProjectKind::SlintApp);
        assert_eq!(r.resolve("rust/errors/no-unwrap", Severity::Error), Severity::Error);
        assert_eq!(r.resolve("topology/layer-violation", Severity::Error), Severity::Error);
        assert_eq!(r.resolve("global/nesting", Severity::Error), Severity::Error);
    }

    #[test]
    fn secrets_always_error() {
        for kind in [ProjectKind::Tool, ProjectKind::CliApp, ProjectKind::Library, ProjectKind::SlintApp] {
            let r = SeverityResolver::for_kind(kind);
            assert_eq!(r.resolve("global/secrets", Severity::Warning), Severity::Error);
        }
    }

    #[test]
    fn library_same_as_cli() {
        let r = SeverityResolver::for_kind(ProjectKind::Library);
        assert_eq!(r.resolve("rust/errors/no-unwrap", Severity::Error), Severity::Warning);
        assert_eq!(r.resolve("topology/layer-violation", Severity::Error), Severity::Skip);
    }

    #[test]
    fn unknown_check_uses_default() {
        let r = SeverityResolver::for_kind(ProjectKind::Tool);
        assert_eq!(r.resolve("some/unknown/check", Severity::Error), Severity::Error);
    }
}
