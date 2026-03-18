# crates/scanner/src/checks/css_checks.rs

## `pub fn check_zero_literal( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 20 · fn*

Check for hardcoded hex colors and px values outside :root/custom properties.

---

