# crates/scanner/src/checks/topology_naming.rs

## `pub fn check( paths: &[PathBuf], cfg: &Config, issues: &mut Vec<Issue>, )`

*Line 30 · fn*

Check Level 1 topology naming: folder/crate names at topology boundary.

1. BANNED names → error with suggestion
2. Unknown topology folders → error (only registered names allowed)

---

