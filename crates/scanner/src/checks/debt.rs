use std::path::Path;

use crate::config::Config;
use crate::context::FileContext;
use crate::issue::{Issue, Severity};

/// Debt markers that must not be committed.
const DEBT_MARKERS: &[(&str, &str)] = &[
    ("TODO", "resolve TODO before committing"),
    ("FIXME", "resolve FIXME before committing"),
    ("HACK", "remove HACK before committing"),
    ("NOCOMMIT", "NOCOMMIT marker — do not commit"),
];

/// Check for TODO/FIXME/HACK/NOCOMMIT markers.
pub fn check(
    _file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    for (line_num, line) in lines.iter().enumerate() {
        let upper = line.to_uppercase();
        for (marker, message) in DEBT_MARKERS {
            if upper.contains(marker) {
                issues.push(Issue::new(
                    path,
                    line_num + 1,
                    1,
                    Severity::Warning,
                    "global/tech-debt",
                    *message,
                ));
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Language;

    fn make_ctx() -> FileContext {
        FileContext {
            language: Language::Rust,
            is_test_file: false,
            is_mother_file: false,
            is_definition_file: false,
        }
    }

    #[test]
    fn finds_todo() {
        let mut issues = Vec::new();
        let lines = vec!["// TODO: fix this"];
        check(&make_ctx(), &lines, &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("TODO"));
    }

    #[test]
    fn finds_nocommit() {
        let mut issues = Vec::new();
        let lines = vec!["// NOCOMMIT debug code"];
        check(&make_ctx(), &lines, &Config::default(), &mut issues, Path::new("a.rs"));
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn clean_file() {
        let mut issues = Vec::new();
        let lines = vec!["fn main() {}", "let x = 1;"];
        check(&make_ctx(), &lines, &Config::default(), &mut issues, Path::new("a.rs"));
        assert!(issues.is_empty());
    }
}
