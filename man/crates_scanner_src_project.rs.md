# crates/scanner/src/project.rs

## `pub enum ProjectKind`

*Line 5 · enum*

What kind of project is this — determines which checks apply.

---

## `pub enum Layout`

*Line 22 · enum*

Project layout — single crate or Cargo workspace.

---

## `pub struct ProjectIdentity`

*Line 31 · struct*

Auto-detected project identity.

---

## `pub fn detect(root: &Path) -> Self`

*Line 39 · fn*

Detect project kind and layout. Explicit `[project].kind` in
`proj/rulestools.toml` overrides auto-detection.

---

## `pub fn is_registered(root: &Path) -> bool`

*Line 46 · fn*

Check if project has explicit registration (proj/rulestools.toml with [project].kind).

---

## `pub fn suggest(root: &Path) -> String`

*Line 51 · fn*

Suggest the best matching ProjectKind for an unregistered project.

---

## `pub fn from_str(s: &str) -> Option<Self>`

*Line 150 · fn*

Parse from string (for `proj/rulestools.toml` `[project].kind`).

---

## `pub fn skipped_categories(&self) -> &'static [&'static str]`

*Line 164 · fn*

Check categories that are skipped for this project kind.

---

## `pub fn upgrade_ord(&self) -> u8`

*Line 198 · fn*

Numeric ordering for upgrade validation.
Higher values = more complex project kind.

---

## `pub fn as_str(&self) -> &'static str`

*Line 210 · fn*

Human-readable kind string for config files and display.

---

## `pub fn allows_check(&self, check_id: &str) -> bool`

*Line 222 · fn*

Whether a check ID should run for this project kind.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
