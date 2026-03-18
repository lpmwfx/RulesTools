# crates/scanner/src/checks/topology_suffix.rs

## `pub fn check( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 25 · fn*

Check Level 3 topology suffix: public types must carry a layer suffix.

`pub struct Engine_core` in `src/core/` → OK
`pub struct Engine` in `src/core/` → Error

---

