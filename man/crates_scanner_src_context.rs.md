# crates/scanner/src/context.rs

## `pub enum Language`

*Line 5 · enum*

Supported source languages.

---

## `pub fn from_extension(ext: &str) -> Option<Self>`

*Line 20 · fn*

Detect language from file extension.

---

## `pub fn from_path(path: &Path) -> Option<Self>`

*Line 37 · fn*

Detect language from a file path.

---

## `pub fn name(self) -> &'static str`

*Line 44 · fn*

fn `name`.

---

## `pub struct FileContext`

*Line 62 · struct*

Context for a single file being scanned.

---

## `pub fn from_path(path: &Path) -> Option<Self>`

*Line 71 · fn*

Build context from a file path.

---

## `pub fn is_comment(line: &str, lang: Language) -> bool`

*Line 133 · fn*

Check if a line is a comment in the given language.

---

## `pub fn is_const_def(line: &str) -> bool`

*Line 146 · fn*

Check if a line is a const/static definition (Rust).

---

## `pub fn is_test_context(lines: &[&str], index: usize) -> bool`

*Line 152 · fn*

Check if lines around an index indicate a test context (Rust #[test] or #[cfg(test)]).

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
