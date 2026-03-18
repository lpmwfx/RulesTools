# crates/scanner/src/checks/clone_spam.rs

## `pub fn check( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 20 · fn*

Check for excessive .clone() calls within a single function.

---

