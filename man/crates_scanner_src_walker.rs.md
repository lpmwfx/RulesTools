# crates/scanner/src/walker.rs

## `pub fn find_workspace_root(start: &Path) -> Option<PathBuf>`

*Line 21 · fn*

Find the workspace root by walking up from `start` looking for Cargo.toml
with `[workspace]`.

---

## `pub fn collect_files(root: &Path, exclude_patterns: &[String]) -> Vec<PathBuf>`

*Line 39 · fn*

Collect all source files under `root`, respecting exclude patterns.

---

## `pub fn is_source_extension(ext: &str) -> bool`

*Line 86 · fn*

Check if a file extension is a known source extension.

---

## `pub fn is_metadata_path(path: &Path) -> bool`

*Line 96 · fn*

Check if a path is inside a metadata directory (proj/, doc/, man/).
Files in these dirs should not have code checks run on them,
but placement checks should still verify no code files exist there.

---

