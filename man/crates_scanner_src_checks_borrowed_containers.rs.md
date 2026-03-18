# crates/scanner/src/checks/borrowed_containers.rs

## `pub fn check( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 21 · fn*

Check for borrowed container types in function parameters.
`&Vec<T>` should be `&[T]`, `&String` should be `&str`.

---

