use crate::config::Config;
use crate::context::FileContext;
use crate::issue::{Issue, Severity};
use std::path::Path;

/// Override suffixes — allowed in any layer regardless of expected suffix.
const OVERRIDE_SUFFIXES: &[&str] = &["_sta", "_cfg", "_test"];

/// Path segment → expected type suffix.
const LAYER_MAP: &[(&str, &str)] = &[
    ("ui", "_ui"),
    ("adapter", "_adp"),
    ("adp", "_adp"),
    ("core", "_core"),
    ("gateway", "_gtw"),
    ("gtw", "_gtw"),
    ("pal", "_pal"),
    ("shared", "_x"),
];

/// Check Level 3 topology suffix: public types must carry a layer suffix.
///
/// `pub struct Engine_core` in `src/core/` → OK
/// `pub struct Engine` in `src/core/` → Error
pub fn check(
    file_ctx: &FileContext,
    lines: &[&str],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
    path: &Path,
) {
    // Skip test files
    if file_ctx.is_test_file {
        return;
    }

    let normalized = path.to_string_lossy().replace('\\', "/");

    // Skip test paths
    if normalized.contains("/tests/") || normalized.ends_with("_test.rs") {
        return;
    }

    // Detect layer from path
    let expected_suffix = match detect_layer(&normalized) {
        Some(suffix) => suffix,
        None => return, // No topology folder or app/ layer → skip
    };

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Must start with pub
        if !trimmed.starts_with("pub") {
            continue;
        }

        if let Some(ident) = extract_type_ident(trimmed) {
            // Check if it has the expected suffix or an override suffix
            if has_override_suffix(ident) {
                continue;
            }
            if !ident.ends_with(expected_suffix) {
                issues.push(Issue::new(
                    path,
                    i + 1,
                    0,
                    Severity::Error,
                    "topology/suffix",
                    &format!(
                        "pub type `{ident}` missing suffix `{expected_suffix}` — rename to `{ident}{expected_suffix}`",
                    ),
                ));
            }
        }
    }
}

/// Detect layer suffix from path segments like `src/{layer}/` or `crates/{layer}/`.
fn detect_layer(path: &str) -> Option<&'static str> {
    for prefix in &["src/", "crates/"] {
        if let Some(pos) = path.find(prefix) {
            let after = &path[pos + prefix.len()..];
            let segment = after.split('/').next().unwrap_or("");

            // app/ layer → skip (no suffix required)
            if segment == "app" {
                return None;
            }

            for (folder, suffix) in LAYER_MAP {
                if segment == *folder {
                    return Some(suffix);
                }
            }
        }
    }
    None
}

/// Extract the type identifier from a `pub` declaration line.
/// Returns None for fn, const, static, mod, use declarations.
fn extract_type_ident(trimmed: &str) -> Option<&str> {
    // Strip visibility: `pub ` or `pub(crate) ` or `pub(super) ` etc.
    let after_vis = if trimmed.starts_with("pub(") {
        // Find closing paren
        let close = trimmed.find(')')?;
        let rest = &trimmed[close + 1..];
        rest.trim_start()
    } else if trimmed.starts_with("pub ") {
        &trimmed[4..]
    } else {
        return None;
    };

    // Match keyword
    let after_kw = if after_vis.starts_with("struct ") {
        &after_vis[7..]
    } else if after_vis.starts_with("enum ") {
        &after_vis[5..]
    } else if after_vis.starts_with("trait ") {
        &after_vis[6..]
    } else if after_vis.starts_with("type ") {
        &after_vis[5..]
    } else {
        return None; // fn, const, static, mod, use → skip
    };

    // Extract identifier: up to whitespace, <, (, {, :, ;, where
    let ident = after_kw
        .split(|c: char| c.is_whitespace() || c == '<' || c == '(' || c == '{' || c == ':' || c == ';')
        .next()
        .unwrap_or("");

    if ident.is_empty() {
        return None;
    }
    Some(ident)
}

/// Check if identifier ends with an override suffix.
fn has_override_suffix(ident: &str) -> bool {
    OVERRIDE_SUFFIXES.iter().any(|s| ident.ends_with(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Language;

    fn make_ctx() -> FileContext {
        FileContext {
            language: Language::Rust,
            is_test_file: false,
            is_mother_file: false,
            is_definition_file: false,
        }
    }

    #[test]
    fn correct_suffix_in_core() {
        let ctx = make_ctx();
        let lines = vec!["pub struct Engine_core {"];
        let mut issues = Vec::new();
        check(&ctx, &lines, &Config::default(), &mut issues, Path::new("src/core/engine.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn missing_suffix_in_core() {
        let ctx = make_ctx();
        let lines = vec!["pub struct Engine {"];
        let mut issues = Vec::new();
        check(&ctx, &lines, &Config::default(), &mut issues, Path::new("src/core/engine.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("_core"));
    }

    #[test]
    fn wrong_suffix_in_core() {
        let ctx = make_ctx();
        let lines = vec!["pub struct Engine_adp {"];
        let mut issues = Vec::new();
        check(&ctx, &lines, &Config::default(), &mut issues, Path::new("src/core/engine.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("_core"));
    }

    #[test]
    fn override_sta_allowed() {
        let ctx = make_ctx();
        let lines = vec!["pub struct State_sta {"];
        let mut issues = Vec::new();
        check(&ctx, &lines, &Config::default(), &mut issues, Path::new("src/core/state.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn override_cfg_allowed() {
        let ctx = make_ctx();
        let lines = vec!["pub struct Config_cfg {"];
        let mut issues = Vec::new();
        check(&ctx, &lines, &Config::default(), &mut issues, Path::new("src/gateway/config.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn app_layer_skip() {
        let ctx = make_ctx();
        let lines = vec!["pub struct App {"];
        let mut issues = Vec::new();
        check(&ctx, &lines, &Config::default(), &mut issues, Path::new("src/app/main.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn no_topology_folder_skip() {
        let ctx = make_ctx();
        let lines = vec!["pub struct Foo {"];
        let mut issues = Vec::new();
        check(&ctx, &lines, &Config::default(), &mut issues, Path::new("src/main.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn pub_crate_checked() {
        let ctx = make_ctx();
        let lines = vec!["pub(crate) struct Foo {"];
        let mut issues = Vec::new();
        check(&ctx, &lines, &Config::default(), &mut issues, Path::new("src/core/foo.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("_core"));
    }

    #[test]
    fn enum_checked() {
        let ctx = make_ctx();
        let lines = vec!["pub enum Kind {"];
        let mut issues = Vec::new();
        check(&ctx, &lines, &Config::default(), &mut issues, Path::new("src/adapter/kind.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("_adp"));
    }

    #[test]
    fn trait_checked() {
        let ctx = make_ctx();
        let lines = vec!["pub trait Store {"];
        let mut issues = Vec::new();
        check(&ctx, &lines, &Config::default(), &mut issues, Path::new("src/gateway/store.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("_gtw"));
    }

    #[test]
    fn fn_not_checked() {
        let ctx = make_ctx();
        let lines = vec!["pub fn new() {"];
        let mut issues = Vec::new();
        check(&ctx, &lines, &Config::default(), &mut issues, Path::new("src/core/engine.rs"));
        assert!(issues.is_empty());
    }

    #[test]
    fn crates_path_works() {
        let ctx = make_ctx();
        let lines = vec!["pub struct Foo {"];
        let mut issues = Vec::new();
        check(&ctx, &lines, &Config::default(), &mut issues, Path::new("crates/core/src/lib.rs"));
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("_core"));
    }
}
