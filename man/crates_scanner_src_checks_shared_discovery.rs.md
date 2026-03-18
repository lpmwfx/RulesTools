# crates/scanner/src/checks/shared_discovery.rs

## `pub fn check( contents: &[(PathBuf, String)], _cfg: &Config, issues: &mut Vec<Issue>, )`

*Line 16 · fn*

Cross-file check: find duplicate pub fn names across child files in the same module.
Functions that appear in 2+ siblings are candidates for `shared/` extraction.

---

