# crates/documenter/src/lib.rs

## `pub mod manifest;`

*Line 2 · mod*

Serde types for documentation items and manifests.

---

## `pub mod parser;`

*Line 4 · mod*

Source file parser — extracts pub items and `///` doc comments.

---

## `pub mod generator;`

*Line 6 · mod*

`man/` directory generator — writes JSON + Markdown.

---

## `pub mod docgen;`

*Line 8 · mod*

Auto-documentation via AI or stub generation.

---

## `pub fn document_project()`

*Line 22 · fn*

Build-time entry point — call from `build.rs`.

Scans for undocumented pub items, generates stubs,
emits `cargo:warning` for each documented item.

```ignore
fn main() {
    rulestools_documenter::document_project();
}
```

---

## `pub fn generate_docs(root: &Path, project_name: &str)`

*Line 53 · fn*

Generate `man/` directory for a project.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
