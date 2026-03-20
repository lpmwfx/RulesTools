# crates/scanner/src/checks/js_safety.rs

## `pub fn check_no_var( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 16 · fn*

Check for `var` declarations — use `let` or `const` instead.

---

## `pub fn check_no_console_log( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 44 · fn*

Check for `console.log` — use a structured logger.

---

## `pub fn check_no_eval( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 72 · fn*

Check for `eval()` — security risk.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
