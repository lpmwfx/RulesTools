use std::path::PathBuf;

use crate::config::Config;
use crate::issue::{Issue, Severity};

/// Layer topology — which layer is a file in?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layer {
    Ui,
    Adapter,
    Core,
    Gateway,
    Pal,
    Shared,
    App,
    Other,
}

impl Layer {
    /// Detect layer from a file path based on directory position.
    fn from_path(path: &std::path::Path) -> Self {
        let normalized = path.to_string_lossy().replace('\\', "/");
        // Check path segments for topology folders
        if contains_layer_segment(&normalized, "ui") || contains_layer_segment(&normalized, "_ui") {
            return Layer::Ui;
        }
        if contains_layer_segment(&normalized, "adapter") || contains_layer_segment(&normalized, "adp") {
            return Layer::Adapter;
        }
        if contains_layer_segment(&normalized, "core") {
            return Layer::Core;
        }
        if contains_layer_segment(&normalized, "gateway") || contains_layer_segment(&normalized, "gtw") {
            return Layer::Gateway;
        }
        if contains_layer_segment(&normalized, "pal") {
            return Layer::Pal;
        }
        if contains_layer_segment(&normalized, "shared") || contains_layer_segment(&normalized, "common") {
            return Layer::Shared;
        }
        if contains_layer_segment(&normalized, "app") {
            return Layer::App;
        }
        Layer::Other
    }

    /// Legal import targets for this layer.
    fn allowed_imports(self) -> &'static [Layer] {
        match self {
            Layer::Ui => &[Layer::Adapter, Layer::Shared],
            Layer::Core => &[Layer::Pal, Layer::Shared],
            Layer::Gateway => &[Layer::Pal, Layer::Shared],
            Layer::Pal => &[Layer::Shared],
            Layer::Adapter => &[Layer::Core, Layer::Gateway, Layer::Pal, Layer::Ui, Layer::Shared],
            Layer::App => &[Layer::Core, Layer::Adapter, Layer::Gateway, Layer::Pal, Layer::Ui, Layer::Shared],
            Layer::Shared => &[], // shared has no internal deps (enforced by shared-guard)
            Layer::Other => &[],  // no restrictions on files outside topology
        }
    }

    fn label(self) -> &'static str {
        match self {
            Layer::Ui => "ui",
            Layer::Adapter => "adapter",
            Layer::Core => "core",
            Layer::Gateway => "gateway",
            Layer::Pal => "pal",
            Layer::Shared => "shared",
            Layer::App => "app",
            Layer::Other => "other",
        }
    }
}

/// Check for cross-layer import violations.
///
/// Legal import matrix:
///   ui      → adapter, shared
///   core    → pal, shared
///   gateway → pal, shared
///   pal     → shared only
///   adapter → all (hub)
///   app     → all (entry point)
///   shared  → nothing internal
pub fn check(
    contents: &[(PathBuf, String)],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
) {
    for (path, content) in contents {
        let source_layer = Layer::from_path(path);
        if source_layer == Layer::Other || source_layer == Layer::Shared {
            // Other = outside topology, skip. Shared = handled by shared-guard.
            continue;
        }

        let allowed = source_layer.allowed_imports();
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                continue;
            }

            // Parse use crate:: imports
            let target = if trimmed.starts_with("use crate::") {
                extract_first_segment(trimmed, "use crate::")
            } else if trimmed.starts_with("pub use crate::") {
                extract_first_segment(trimmed, "pub use crate::")
            } else {
                continue;
            };

            let target_layer = layer_from_module_name(&target);
            if target_layer == Layer::Other {
                continue; // can't determine layer — skip
            }

            if !allowed.contains(&target_layer) {
                issues.push(Issue::new(
                    path,
                    i + 1,
                    1,
                    Severity::Error,
                    "topology/layer-violation",
                    &format!(
                        "{} may not import {} — {} can only import {}",
                        source_layer.label(),
                        target_layer.label(),
                        source_layer.label(),
                        allowed.iter().map(|l| l.label()).collect::<Vec<_>>().join(", "),
                    ),
                ));
            }
        }
    }
}

/// Extract the first path segment after a prefix like "use crate::".
fn extract_first_segment(line: &str, prefix: &str) -> String {
    let after = &line[prefix.len()..];
    after
        .split(|c: char| c == ':' || c == ';' || c == '{' || c.is_whitespace())
        .next()
        .unwrap_or("")
        .to_string()
}

/// Map a module name to a topology layer.
fn layer_from_module_name(name: &str) -> Layer {
    match name {
        "ui" => Layer::Ui,
        "adapter" | "adp" => Layer::Adapter,
        "core" => Layer::Core,
        "gateway" | "gtw" => Layer::Gateway,
        "pal" => Layer::Pal,
        "shared" | "common" => Layer::Shared,
        "app" => Layer::App,
        _ => Layer::Other,
    }
}

/// Check if a normalized path contains a directory segment in topology position.
fn contains_layer_segment(path: &str, segment: &str) -> bool {
    let patterns = [
        format!("/src/{segment}/"),
        format!("/crates/{segment}/"),
        format!("/{segment}/src/"),
        format!("src/{segment}/"),    // no leading slash (relative paths)
        format!("crates/{segment}/"), // no leading slash (relative paths)
    ];
    // Also match if path starts with segment/
    if path.starts_with(&format!("{segment}/")) {
        return true;
    }
    patterns.iter().any(|p| path.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_importing_adapter_ok() {
        let contents = vec![(
            PathBuf::from("src/ui/menu.rs"),
            "use crate::adapter::Hub;\n".to_string(),
        )];
        let mut issues = Vec::new();
        check(&contents, &Config::default(), &mut issues);
        assert!(issues.is_empty());
    }

    #[test]
    fn ui_importing_core_error() {
        let contents = vec![(
            PathBuf::from("src/ui/menu.rs"),
            "use crate::core::engine;\n".to_string(),
        )];
        let mut issues = Vec::new();
        check(&contents, &Config::default(), &mut issues);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "topology/layer-violation");
        assert!(issues[0].message.contains("ui may not import core"));
    }

    #[test]
    fn ui_importing_gateway_error() {
        let contents = vec![(
            PathBuf::from("src/ui/panel.rs"),
            "use crate::gateway::file_io;\n".to_string(),
        )];
        let mut issues = Vec::new();
        check(&contents, &Config::default(), &mut issues);
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn core_importing_pal_ok() {
        let contents = vec![(
            PathBuf::from("src/core/calc.rs"),
            "use crate::pal::platform;\n".to_string(),
        )];
        let mut issues = Vec::new();
        check(&contents, &Config::default(), &mut issues);
        assert!(issues.is_empty());
    }

    #[test]
    fn core_importing_gateway_error() {
        let contents = vec![(
            PathBuf::from("src/core/calc.rs"),
            "use crate::gateway::file_io;\n".to_string(),
        )];
        let mut issues = Vec::new();
        check(&contents, &Config::default(), &mut issues);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("core may not import gateway"));
    }

    #[test]
    fn adapter_imports_all_ok() {
        let contents = vec![(
            PathBuf::from("src/adapter/hub.rs"),
            "use crate::core::engine;\nuse crate::gateway::db;\nuse crate::ui::view;\n".to_string(),
        )];
        let mut issues = Vec::new();
        check(&contents, &Config::default(), &mut issues);
        assert!(issues.is_empty());
    }

    #[test]
    fn shared_imports_skipped() {
        // shared/ is handled by shared-guard check, not layer-violation
        let contents = vec![(
            PathBuf::from("src/shared/utils.rs"),
            "use crate::core::engine;\n".to_string(),
        )];
        let mut issues = Vec::new();
        check(&contents, &Config::default(), &mut issues);
        assert!(issues.is_empty());
    }

    #[test]
    fn workspace_crate_path() {
        let contents = vec![(
            PathBuf::from("crates/ui/src/menu.rs"),
            "use crate::core::engine;\n".to_string(),
        )];
        let mut issues = Vec::new();
        check(&contents, &Config::default(), &mut issues);
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn pal_importing_core_error() {
        let contents = vec![(
            PathBuf::from("src/pal/windows.rs"),
            "use crate::core::engine;\n".to_string(),
        )];
        let mut issues = Vec::new();
        check(&contents, &Config::default(), &mut issues);
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn comments_ignored() {
        let contents = vec![(
            PathBuf::from("src/ui/menu.rs"),
            "// use crate::core::engine;\n".to_string(),
        )];
        let mut issues = Vec::new();
        check(&contents, &Config::default(), &mut issues);
        assert!(issues.is_empty());
    }

    #[test]
    fn unknown_module_skipped() {
        let contents = vec![(
            PathBuf::from("src/ui/menu.rs"),
            "use crate::something_else::Foo;\n".to_string(),
        )];
        let mut issues = Vec::new();
        check(&contents, &Config::default(), &mut issues);
        assert!(issues.is_empty());
    }
}
