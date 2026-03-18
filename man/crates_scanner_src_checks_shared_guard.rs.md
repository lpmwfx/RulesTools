# crates/scanner/src/checks/shared_guard.rs

## `pub fn check( _file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 9 · fn*

Check that files in shared/ have no internal project imports.

shared/ must be dependency-free — only std and external crates allowed.

---

