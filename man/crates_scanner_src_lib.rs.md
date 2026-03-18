# crates/scanner/src/lib.rs

## `pub mod issue;`

*Line 2 · mod*

Scan issue types and severity levels.

---

## `pub mod config;`

*Line 4 · mod*

Configuration parsing and project settings.

---

## `pub mod context;`

*Line 6 · mod*

Language detection and file context.

---

## `pub mod walker;`

*Line 8 · mod*

File system walking and source file collection.

---

## `pub mod output;`

*Line 10 · mod*

Output formatting — cargo warnings and ISSUES file.

---

## `pub mod checks;`

*Line 12 · mod*

Check registry and dispatch traits.

---

## `pub mod project;`

*Line 14 · mod*

Auto-detection of project kind and layout.

---

## `pub mod severity;`

*Line 16 · mod*

Severity resolver — maps check severity per ProjectKind.

---

## `pub fn scan_project()`

*Line 33 · fn*

Scan a project from build.rs — emits `cargo:warning` lines.

Call this from your `build.rs`:
```ignore
fn main() {
    rulestools_scanner::scan_project();
}
```

---

## `pub fn scan_at(root: &Path) -> (Vec<Issue>, usize)`

*Line 58 · fn*

Scan a project from CLI/MCP — returns issues and writes `proj/ISSUES`.

Returns `(all_issues, new_count)`.
If the project has no `proj/rulestools.toml` with `[project].kind`,
returns a single Info issue with a registration suggestion.

---

## `pub fn scan_super(root: &Path) -> (Vec<Issue>, usize)`

*Line 81 · fn*

Scan a super-project — find sub-repos and scan each with its own config.

Returns aggregated issues from all sub-repos with path prefixed by sub-repo name.

---

## `pub fn run_scan(root: &Path) -> Vec<Issue>`

*Line 125 · fn*

Core scan logic — collects files, runs all registered checks.

---

