# crates/scanner/src/checks/slint_mother_child.rs

## `pub fn check( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 10 · fn*

Check Slint mother-child topology rules.

1. Children must be stateless — no `in-out property` (except <=> delegation)
2. Child files cannot import from sibling views

---

