# crates/scanner/src/checks/slint_gateway.rs

## `pub fn check( paths: &[PathBuf], _cfg: &Config, issues: &mut Vec<Issue>, )`

*Line 34 · fn*

Check that all Slint UI callbacks delegate to exactly ONE gateway object.

Multiple different gateway receivers across .slint files = error.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
