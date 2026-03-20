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



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
