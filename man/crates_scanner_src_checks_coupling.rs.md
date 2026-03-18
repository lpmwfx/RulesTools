# crates/scanner/src/checks/coupling.rs

## `pub fn check( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 18 · fn*

Check for sibling coupling via `use super::` and `pub(super)`.

---

