# crates/scanner/src/checks/borrowed_containers.rs

## `pub fn check( file_ctx: &FileContext, lines: &[&str], _cfg: &Config, issues: &mut Vec<Issue>, path: &Path, )`

*Line 21 · fn*

Check for borrowed container types in function parameters.
`&Vec<T>` should be `&[T]`, `&String` should be `&str`.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
