# crates/scanner/src/checks/python_checks.rs

## `pub fn check_missing_annotations( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 24 · fn*

Check for missing type annotations, Optional[], and global keyword.

---

## `pub fn check_naming_conventions( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 85 · fn*

Check Python naming conventions: PascalCase classes, snake_case functions.

---

## `pub fn check_boundary_validation( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 136 · fn*

Check for json.loads without model validation nearby.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
