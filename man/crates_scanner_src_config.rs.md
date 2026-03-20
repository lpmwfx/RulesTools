# crates/scanner/src/config.rs

## `pub struct Config`

*Line 6 · struct*

Unified scan configuration — parsed from `proj/rulestools.toml`.

---

## `pub fn parse(content: &str) -> Self`

*Line 16 · fn*

Parse config from TOML string content.

---

## `pub fn load(project_root: &Path) -> Self`

*Line 73 · fn*

Load config from a project directory (reads `proj/rulestools.toml`).

---

## `pub fn is_enabled(&self, check_id: &str) -> bool`

*Line 82 · fn*

Check if a specific check is enabled. Defaults to true if not configured.

---

## `pub fn param_i64(&self, key: &str, default: i64) -> i64`

*Line 87 · fn*

Get a parameter value for a check, with a default fallback.

---

## `pub fn param_str(&self, key: &str, default: &str) -> String`

*Line 95 · fn*

Get a parameter string value.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
