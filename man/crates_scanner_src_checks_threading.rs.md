# crates/scanner/src/checks/threading.rs

## `pub fn check( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 24 · fn*

Check threading patterns in Rust code.

---

## `pub fn check_fire_and_forget( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 68 · fn*

Check for fire-and-forget spawns — `tokio::spawn(...)` or `thread::spawn(...)`
without a `let` binding on the same line.

---

