# crates/scanner/src/issue.rs

## `pub enum Severity`

*Line 7 · enum*

Severity level for scan issues.

---

## `pub fn label(self) -> &'static str`

*Line 27 · fn*

fn `label`.

---

## `pub struct Issue`

*Line 46 · struct*

A single scan issue — one violation of one rule in one file.

---

## `pub fn new( path: impl Into<PathBuf>, line: usize, col: usize, severity: Severity, rule_id: impl Into<String>, message: impl Into<String>, ) -> Self`

*Line 57 · fn*

fn `new`.

---

## `pub fn rule_ref(&self) -> &'static str`

*Line 77 · fn*

The Rules file that documents this check.
Maps rule_id prefix to the .md file an AI should read.

---

## `pub fn identity_key(&self) -> String`

*Line 126 · fn*

Key used to identify this issue across scans (for [NEW]/[KNOWN] delta).

---

## `pub fn display_line(&self) -> String`

*Line 137 · fn*

Format as VSCode problem-matcher compatible line.

---

## `pub fn relative_path(&self, base: &Path) -> PathBuf`

*Line 164 · fn*

Returns the relative path if possible, otherwise the original.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
