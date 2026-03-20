# crates/scanner/src/checks/slint_checks.rs

## `pub fn check_doc_required( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 22 · fn*

Check that exported components/structs have `///` doc comments.

---

## `pub fn check_zero_literal( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 47 · fn*

Check for hardcoded hex colors and px values in component files.

---

## `pub fn check_globals_structure( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 83 · fn*

Check that global/definition files have no control flow.

---

## `pub fn check_no_hardcoded_string( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 110 · fn*

Check for hardcoded text strings in component files.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
