# crates/scanner/src/checks/errors.rs

## `pub fn check( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 24 · fn*

Check for unwrap/expect/panic/todo in non-test Rust code.

---

