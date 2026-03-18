use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Known source file extensions.
const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "slint", "py", "js", "mjs", "cjs", "jsx", "ts", "tsx",
    "css", "scss", "kt", "kts", "cs",
    "cpp", "cxx", "cc", "h", "hpp", "hxx",
    "html", "htm",
];

/// Default directories to exclude from scanning.
const EXCLUDE_DIRS: &[&str] = &[
    "target", "node_modules", ".git", "__pycache__", "dist", "build",
    ".venv", "venv", ".tox", ".mypy_cache", ".pytest_cache",
    "bin", "obj", ".gradle", "vendor", "third_party", "external",
];

/// Find the workspace root by walking up from `start` looking for Cargo.toml
/// with `[workspace]`.
pub fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                if content.contains("[workspace]") {
                    return Some(current);
                }
            }
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Collect all source files under `root`, respecting exclude patterns.
pub fn collect_files(root: &Path, exclude_patterns: &[String]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for entry in WalkDir::new(root).into_iter().filter_entry(|e| {
        if !e.file_type().is_dir() {
            return true;
        }
        let dir_name = e.file_name().to_string_lossy();
        !EXCLUDE_DIRS.iter().any(|ex| *ex == dir_name.as_ref())
    }) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();

        // Check extension
        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e,
            None => continue,
        };
        if !SOURCE_EXTENSIONS.contains(&ext) {
            continue;
        }

        // Check exclude patterns
        let path_str = path.to_string_lossy();
        if exclude_patterns.iter().any(|pat| {
            glob::Pattern::new(pat)
                .map(|p| p.matches(&path_str))
                .unwrap_or(false)
        }) {
            continue;
        }

        files.push(path.to_path_buf());
    }

    files
}

/// Check if a file extension is a known source extension.
pub fn is_source_extension(ext: &str) -> bool {
    SOURCE_EXTENSIONS.contains(&ext)
}

/// Directories that contain metadata/docs, not source code.
const METADATA_DIRS: &[&str] = &["proj", "doc", "docs", "man"];

/// Check if a path is inside a metadata directory (proj/, doc/, man/).
/// Files in these dirs should not have code checks run on them,
/// but placement checks should still verify no code files exist there.
pub fn is_metadata_path(path: &Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/");
    for dir in METADATA_DIRS {
        let pattern = format!("/{dir}/");
        if normalized.contains(&pattern) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_source_extensions() {
        assert!(is_source_extension("rs"));
        assert!(is_source_extension("slint"));
        assert!(is_source_extension("py"));
        assert!(is_source_extension("js"));
        assert!(is_source_extension("ts"));
        assert!(is_source_extension("tsx"));
        assert!(is_source_extension("css"));
        assert!(is_source_extension("scss"));
        assert!(is_source_extension("kt"));
        assert!(is_source_extension("cs"));
        assert!(!is_source_extension("md"));
        assert!(!is_source_extension("toml"));
        assert!(!is_source_extension("yaml"));
        assert!(!is_source_extension("txt"));
    }

    #[test]
    fn exclude_pattern_matching() {
        let patterns = vec!["**/generated/**".to_string()];
        let path_str = "src/generated/bindings.rs";
        let matches = patterns.iter().any(|pat| {
            glob::Pattern::new(pat)
                .map(|p| p.matches(path_str))
                .unwrap_or(false)
        });
        assert!(matches);
    }

    #[test]
    fn exclude_pattern_no_match() {
        let patterns = vec!["**/generated/**".to_string()];
        let path_str = "src/main.rs";
        let matches = patterns.iter().any(|pat| {
            glob::Pattern::new(pat)
                .map(|p| p.matches(path_str))
                .unwrap_or(false)
        });
        assert!(!matches);
    }
}
