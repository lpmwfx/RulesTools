# crates/scanner/src/checks/js_modules.rs

## `pub fn check_jsdoc_required( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 20 · fn*

Check that exported functions/consts have JSDoc comments.

---

## `pub fn check_no_require( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 49 · fn*

Check for CommonJS require() and module.exports — use ES modules.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
