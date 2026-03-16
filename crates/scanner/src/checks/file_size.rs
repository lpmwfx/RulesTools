use std::path::Path;

use crate::config::Config;
use crate::context::{FileContext, Language};
use crate::issue::{Issue, Severity};

/// Soft/hard code-line limits per language.
fn limits(lang: Language) -> (usize, usize) {
    match lang {
        Language::Css => (120, 150),
        _ => (200, 250),
    }
}

/// Count code lines — excludes blanks, comments, pure string lines.
fn count_code_lines(lines: &[&str], lang: Language) -> usize {
    let mut count = 0;
    let mut in_block_comment = false;

    for line in lines {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Block comment tracking
        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }
        if trimmed.starts_with("/*") {
            in_block_comment = !trimmed.contains("*/");
            continue;
        }

        // Line comments
        match lang {
            Language::Python => {
                if trimmed.starts_with('#') {
                    continue;
                }
            }
            _ => {
                if trimmed.starts_with("//") {
                    continue;
                }
            }
        }

        count += 1;
    }

    count
}

/// Check file size against language-specific limits.
pub fn check(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    let (soft, hard) = limits(file_ctx.language);
    let code_lines = count_code_lines(lines, file_ctx.language);

    if code_lines >= hard {
        issues.push(Issue::new(
            path,
            lines.len(),
            1,
            Severity::Error,
            "global/file-limits",
            format!(
                "file has {code_lines} code lines (limit {hard}) — split the module before adding anything"
            ),
        ));
    } else if code_lines >= soft {
        issues.push(Issue::new(
            path,
            lines.len(),
            1,
            Severity::Warning,
            "global/file-limits",
            format!(
                "file has {code_lines} code lines (soft limit {soft}) — plan the split now"
            ),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_skips_blanks_and_comments() {
        let lines: Vec<&str> = vec![
            "// comment",
            "",
            "fn main() {",
            "    // inline comment",
            "    let x = 1;",
            "}",
        ];
        assert_eq!(count_code_lines(&lines, Language::Rust), 3);
    }

    #[test]
    fn count_skips_python_comments() {
        let lines: Vec<&str> = vec!["# comment", "", "x = 1", "y = 2"];
        assert_eq!(count_code_lines(&lines, Language::Python), 2);
    }

    #[test]
    fn count_skips_block_comments() {
        let lines: Vec<&str> = vec![
            "/* start",
            "   middle",
            "   end */",
            "fn foo() {}",
        ];
        assert_eq!(count_code_lines(&lines, Language::Rust), 1);
    }

    #[test]
    fn css_has_lower_limits() {
        assert_eq!(limits(Language::Css), (120, 150));
        assert_eq!(limits(Language::Rust), (200, 250));
    }
}
