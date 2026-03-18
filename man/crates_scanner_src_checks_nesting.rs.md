# crates/scanner/src/checks/nesting.rs

## `pub fn check( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 25 · fn*

Check control-flow nesting depth — measures complexity, not just braces.

Counts nesting for control flow (`if`, `for`, `while`, `match`, `loop`,
closures, callbacks) but NOT for struct/enum bodies, type annotations,
or Slint property types.

---

