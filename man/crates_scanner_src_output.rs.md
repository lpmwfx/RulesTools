# crates/scanner/src/output.rs

## `pub fn emit_cargo_warnings(issues: &[Issue], base: &Path)`

*Line 7 · fn*

Emit issues as `cargo:warning` lines (for build.rs integration).

---

## `pub fn write_issues_file(issues: &[Issue], project_root: &Path) -> std::io::Result<usize>`

*Line 25 · fn*

Write issues to `proj/ISSUES` with [NEW]/[KNOWN] delta markers.

Returns the number of new issues found.

---

## `pub struct IssueGroup`

*Line 76 · struct*

Error group with guidance text.

---

## `pub fn format_grouped(issues: &[Issue], base: &Path) -> String`

*Line 140 · fn*

Format issues as grouped output with guidance per group.

If `rules_root` is provided, loads decision trees from Rules/guidance/*.toml.

---

## `pub fn format_grouped_with_guidance(issues: &[Issue], base: &Path, rules_root: Option<&Path>) -> String`

*Line 145 · fn*

Format with optional guidance trees from Rules/guidance/.

---

## `pub fn should_deny(issues: &[Issue], deny: bool) -> bool`

*Line 236 · fn*

Check if build should be denied.
Critical issues always deny. Error issues deny when deny=true.

---

