# crates/scanner/src/checks/layer_violation.rs

## `pub fn check( contents: &[(PathBuf, String)], _cfg: &Config, issues: &mut Vec<Issue>, )`

*Line 86 · fn*

Check for cross-layer import violations.

Legal import matrix:
  ui      → adapter, shared
  core    → pal, shared
  gateway → pal, shared
  pal     → shared only
  adapter → all (hub)
  app     → all (entry point)
  shared  → nothing internal

---

