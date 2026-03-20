# apps/cli/src/commands/scan.rs

## `pub fn find_project_root(path: &std::path::Path) -> PathBuf`

*Line 4 · fn*

Walk up from file to find project root (directory with Cargo.toml or proj/).

---

## `pub fn find_rules_root(project_root: &std::path::Path) -> Option<PathBuf>`

*Line 21 · fn*

Find the Rules/ directory — check common locations.

---

## `pub fn scan_internal(path: &std::path::Path, deny: bool) -> Result<String, String>`

*Line 45 · fn*

fn `scan_internal`.

---

## `pub fn cmd_scan(path: &PathBuf, deny: bool)`

*Line 100 · fn*

fn `cmd_scan`.

---

## `pub fn scan_file_internal(file: &std::path::Path, format: &str) -> Result<String, String>`

*Line 111 · fn*

fn `scan_file_internal`.

---

## `pub fn cmd_scan_file(file: &PathBuf, format: &str)`

*Line 186 · fn*

fn `cmd_scan_file`.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
