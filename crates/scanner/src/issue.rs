use std::cmp::Ordering;
use std::fmt;
use std::path::{Path, PathBuf};

/// Severity level for scan issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    Critical,
    Error,
    Warning,
    Info,
    Skip,
}

impl Severity {
    fn rank(self) -> u8 {
        match self {
            Severity::Critical => 0,
            Severity::Error => 1,
            Severity::Warning => 2,
            Severity::Info => 3,
            Severity::Skip => 4,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Skip => "skip",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// A single scan issue — one violation of one rule in one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub path: PathBuf,
    pub line: usize,
    pub col: usize,
    pub severity: Severity,
    pub rule_id: String,
    pub message: String,
}

impl Issue {
    pub fn new(
        path: impl Into<PathBuf>,
        line: usize,
        col: usize,
        severity: Severity,
        rule_id: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            line,
            col,
            severity,
            rule_id: rule_id.into(),
            message: message.into(),
        }
    }

    /// The Rules file that documents this check.
    /// Maps rule_id prefix to the .md file an AI should read.
    pub fn rule_ref(&self) -> &'static str {
        match self.rule_id.as_str() {
            id if id.starts_with("global/file-limits") => "global/file-limits.md",
            id if id.starts_with("global/nesting") => "global/nesting.md",
            id if id.starts_with("global/tech-debt") => "global/tech-debt.md",
            id if id.starts_with("global/secrets") => "global/secrets.md",
            id if id.starts_with("rust/constants") => "rust/constants.md",
            id if id.starts_with("rust/errors") => "rust/errors.md",
            id if id.starts_with("rust/docs") => "rust/docs.md",
            id if id.starts_with("rust/types") => "rust/types.md",
            id if id.starts_with("rust/naming") => "rust/naming.md",
            id if id.starts_with("rust/modules/shared-guard") => "global/stereotypes.md",
            id if id.starts_with("rust/modules/shared-candidate") => "global/mother-tree.md",
            id if id.starts_with("rust/modules/no-sibling-import") => "global/mother-tree.md",
            id if id.starts_with("rust/modules") => "rust/modules.md",
            id if id.starts_with("rust/ownership") => "rust/ownership.md",
            id if id.starts_with("rust/safety") => "rust/safety.md",
            id if id.starts_with("rust/threading") => "rust/threading.md",
            id if id.starts_with("topology/layer") => "global/topology.md",
            id if id.starts_with("topology/placement") => "global/topology.md",
            id if id.starts_with("topology/suffix") => "global/naming-suffix.md",
            id if id.starts_with("topology/naming") => "global/naming-suffix.md",
            id if id.starts_with("topology/unregistered") => "global/topology.md",
            id if id.starts_with("js/safety") => "js/safety.md",
            id if id.starts_with("uiux/mother-child") => "uiux/mother-child.md",
            id if id.starts_with("uiux/state-flow") => "uiux/state-flow.md",
            _ => "",
        }
    }

    /// Key used to identify this issue across scans (for [NEW]/[KNOWN] delta).
    pub fn identity_key(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.path.display(),
            self.line,
            self.rule_id,
            self.message,
        )
    }

    /// Format as VSCode problem-matcher compatible line.
    pub fn display_line(&self) -> String {
        let rule_ref = self.rule_ref();
        if rule_ref.is_empty() {
            format!(
                "{}:{}:{}: {} {}: {}",
                self.path.display(),
                self.line,
                self.col,
                self.severity,
                self.rule_id,
                self.message,
            )
        } else {
            format!(
                "{}:{}:{}: {} {}: {} [{}]",
                self.path.display(),
                self.line,
                self.col,
                self.severity,
                self.rule_id,
                self.message,
                rule_ref,
            )
        }
    }

    /// Returns the relative path if possible, otherwise the original.
    pub fn relative_path(&self, base: &Path) -> PathBuf {
        self.path
            .strip_prefix(base)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| self.path.clone())
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.display_line())
    }
}

impl Ord for Issue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.severity
            .rank()
            .cmp(&other.severity.rank())
            .then_with(|| self.path.cmp(&other.path))
            .then_with(|| self.line.cmp(&other.line))
            .then_with(|| self.col.cmp(&other.col))
            .then_with(|| self.rule_id.cmp(&other.rule_id))
    }
}

impl PartialOrd for Issue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_matches_vscode_pattern() {
        let issue = Issue::new("src/main.rs", 42, 5, Severity::Error, "rust/errors/no-unwrap", "unwrap() in non-test code");
        assert_eq!(
            issue.to_string(),
            "src/main.rs:42:5: error rust/errors/no-unwrap: unwrap() in non-test code [rust/errors.md]"
        );
    }

    #[test]
    fn rule_ref_maps_correctly() {
        let issue = Issue::new("x.rs", 1, 1, Severity::Error, "rust/safety/unsafe-needs-comment", "m");
        assert_eq!(issue.rule_ref(), "rust/safety.md");

        let issue = Issue::new("x.rs", 1, 1, Severity::Error, "topology/layer-violation", "m");
        assert_eq!(issue.rule_ref(), "global/topology.md");

        let issue = Issue::new("x.rs", 1, 1, Severity::Error, "rust/modules/shared-guard", "m");
        assert_eq!(issue.rule_ref(), "global/stereotypes.md");
    }

    #[test]
    fn sort_order_critical_first() {
        let crit = Issue::new("a.rs", 1, 1, Severity::Critical, "r0", "c");
        let err = Issue::new("a.rs", 1, 1, Severity::Error, "r1", "e");
        let warn = Issue::new("a.rs", 1, 1, Severity::Warning, "r2", "w");
        let info = Issue::new("a.rs", 1, 1, Severity::Info, "r3", "i");

        let mut issues = vec![info.clone(), err.clone(), warn.clone(), crit.clone()];
        issues.sort();
        assert_eq!(issues, vec![crit, err, warn, info]);
    }

    #[test]
    fn sort_order_same_severity_by_path_then_line() {
        let a10 = Issue::new("a.rs", 10, 1, Severity::Error, "r1", "m");
        let a5 = Issue::new("a.rs", 5, 1, Severity::Error, "r1", "m");
        let b1 = Issue::new("b.rs", 1, 1, Severity::Error, "r1", "m");

        let mut issues = vec![b1.clone(), a10.clone(), a5.clone()];
        issues.sort();
        assert_eq!(issues, vec![a5, a10, b1]);
    }

    #[test]
    fn identity_key_stable() {
        let issue = Issue::new("src/lib.rs", 10, 3, Severity::Warning, "rust/naming/bool-prefix", "missing is_ prefix");
        assert_eq!(
            issue.identity_key(),
            "src/lib.rs:10:rust/naming/bool-prefix:missing is_ prefix"
        );
    }

    #[test]
    fn severity_labels() {
        assert_eq!(Severity::Critical.label(), "critical");
        assert_eq!(Severity::Error.label(), "error");
        assert_eq!(Severity::Warning.label(), "warning");
        assert_eq!(Severity::Info.label(), "info");
        assert_eq!(Severity::Skip.label(), "skip");
    }
}
