# crates/scanner/src/checks/cpp_checks.rs

## `pub fn check_naming( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 18 · fn*

Check C++ naming conventions: PascalCase classes, snake_case/camelCase functions.

---

## `pub fn check_doc_required( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 50 · fn*

Check C++ public items have doc comments.

---

## `pub fn check_safety( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 81 · fn*

Check for raw memory management — prefer smart pointers.

---

