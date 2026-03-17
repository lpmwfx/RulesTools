use std::collections::HashSet;
use std::path::PathBuf;

use crate::config::Config;
use crate::issue::{Issue, Severity};

/// Built-in Slint objects and common types that are NOT gateway receivers.
/// This list is intentionally broad — it's better to skip a false positive
/// than to flag a non-gateway type as a topology error.
const BUILTIN_RECEIVERS: &[&str] = &[
    // Slint builtins
    "Math", "Colors", "Palette", "Theme", "StyleMetrics",
    "self", "root", "parent", "TextInputInterface",
    // Slint enums / structs (commonly used in callbacks)
    "PointerEventKind", "PointerEventButton", "PointerScrollEvent",
    "KeyEvent", "KeyboardModifiers", "StandardButtonKind",
    "TextHorizontalAlignment", "TextVerticalAlignment",
    "ImageFit", "ImageRendering", "FillRule",
    "InputType", "TextWrap", "TextOverflow", "EventResult",
    // Settings/config globals
    "Settings", "EditorSettings", "AppSettings", "UserSettings",
    // Design tokens / theming
    "FluentIcons", "MaterialIcons", "MaterialSymbols",
    "Spacing", "Sizes", "Radius", "Elevation",
    "Type", "Variants", "State", "States",
    "MaterialColors", "FluentColors", "ColorScheme", "AppearanceMode",
    // Navigation
    "ViewId", "NavId", "PageId", "TabId",
];

/// Check that all Slint UI callbacks delegate to exactly ONE gateway object.
///
/// Multiple different gateway receivers across .slint files = error.
pub fn check(
    paths: &[PathBuf],
    _cfg: &Config,
    issues: &mut Vec<Issue>,
) {
    let mut receivers: HashSet<String> = HashSet::new();
    let mut first_receiver_path: Option<(PathBuf, usize, String)> = None;

    for path in paths {
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("slint") {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Look for callback assignments: receiver.method(...)
            // Pattern: something like `root.gateway.do_thing()` or `AppAdapter.save()`
            // In Slint callbacks: `clicked => { Gateway.action(); }`
            if !trimmed.contains("=>") && !trimmed.contains("changed =>") {
                continue;
            }

            // Find receiver objects in callback bodies
            // Look for lines after => { that contain Object.method()
            // Simple heuristic: find Capitalized.method( patterns
            for receiver in extract_receivers(trimmed) {
                if BUILTIN_RECEIVERS.contains(&receiver.as_str()) {
                    continue;
                }
                if receivers.is_empty() {
                    first_receiver_path = Some((path.clone(), i + 1, receiver.clone()));
                }
                receivers.insert(receiver);
            }
        }

        // Also scan non-callback lines for receiver patterns inside callback blocks
        let mut in_callback = false;
        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("=>") && trimmed.contains('{') {
                in_callback = true;
                continue;
            }
            if in_callback {
                if trimmed.contains('}') {
                    in_callback = false;
                }
                for receiver in extract_receivers(trimmed) {
                    if BUILTIN_RECEIVERS.contains(&receiver.as_str()) {
                        continue;
                    }
                    if receivers.is_empty() {
                        first_receiver_path = Some((path.clone(), i + 1, receiver.clone()));
                    }
                    receivers.insert(receiver);
                }
            }
        }
    }

    if receivers.len() > 1 {
        let receiver_list: Vec<&str> = receivers.iter().map(|s| s.as_str()).collect();
        if let Some((ref path, line, ref _first)) = first_receiver_path {
            issues.push(Issue::new(
                path,
                line,
                1,
                Severity::Error,
                "uiux/state-flow/single-gateway",
                &format!(
                    "multiple gateway receivers: {} — all callbacks must delegate to one gateway object",
                    receiver_list.join(", "),
                ),
            ));
        }
    }
}

/// Extract capitalized receiver names from a line (e.g., `Gateway.save()` → "Gateway").
fn extract_receivers(line: &str) -> Vec<String> {
    let mut receivers = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();

    let mut i = 0;
    while i < len {
        // Look for UpperCase followed by letters, then a dot
        if chars[i].is_uppercase() {
            let start = i;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            if i < len && chars[i] == '.' {
                let name: String = chars[start..i].iter().collect();
                // Must be followed by a method call (lowercase letter)
                if i + 1 < len && chars[i + 1].is_lowercase() {
                    receivers.push(name);
                }
            }
        } else {
            i += 1;
        }
    }

    receivers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_receivers_basic() {
        let receivers = extract_receivers("Gateway.save()");
        assert_eq!(receivers, vec!["Gateway"]);
    }

    #[test]
    fn extract_receivers_builtin_filtered() {
        let receivers = extract_receivers("Math.round(x)");
        assert_eq!(receivers, vec!["Math"]); // extracted but filtered in check()
    }

    #[test]
    fn extract_receivers_multiple() {
        let receivers = extract_receivers("Gateway.save(); Backend.load()");
        assert_eq!(receivers, vec!["Gateway", "Backend"]);
    }

    #[test]
    fn extract_receivers_none() {
        let receivers = extract_receivers("let x = 42;");
        assert!(receivers.is_empty());
    }

    #[test]
    fn extract_receivers_lowercase_ignored() {
        let receivers = extract_receivers("self.value = 10;");
        assert!(receivers.is_empty()); // self starts with lowercase
    }
}
