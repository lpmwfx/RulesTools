# crates/scanner/src/checks/unsafe_comment.rs

## `pub fn check( _file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 7 · fn*

Check that every `unsafe` block or `unsafe fn` has a `// SAFETY:` comment.

---

