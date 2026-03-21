use std::path::Path;
use std::sync::LazyLock;
use regex::Regex;

use crate::manifest::{DocItem, ItemKind, SourceDoc};

struct ItemPattern {
    regex: Regex,
    kind: ItemKind,
}

static RS_PATTERNS: LazyLock<Vec<ItemPattern>> = LazyLock::new(|| vec![
    ItemPattern { regex: Regex::new(r"^\s*pub(?:\([^)]+\))?\s+(?:async\s+)?fn\s+(\w+)").unwrap(), kind: ItemKind::Fn },
    ItemPattern { regex: Regex::new(r"^\s*pub(?:\([^)]+\))?\s+struct\s+(\w+)").unwrap(), kind: ItemKind::Struct },
    ItemPattern { regex: Regex::new(r"^\s*pub(?:\([^)]+\))?\s+enum\s+(\w+)").unwrap(), kind: ItemKind::Enum },
    ItemPattern { regex: Regex::new(r"^\s*pub(?:\([^)]+\))?\s+trait\s+(\w+)").unwrap(), kind: ItemKind::Trait },
    ItemPattern { regex: Regex::new(r"^\s*pub(?:\([^)]+\))?\s+type\s+(\w+)").unwrap(), kind: ItemKind::Type },
    ItemPattern { regex: Regex::new(r"^\s*pub(?:\([^)]+\))?\s+mod\s+(\w+)").unwrap(), kind: ItemKind::Mod },
    ItemPattern { regex: Regex::new(r"^\s*pub(?:\([^)]+\))?\s+(?:const|static)\s+(\w+)").unwrap(), kind: ItemKind::Const },
]);

static SLINT_PATTERNS: LazyLock<Vec<ItemPattern>> = LazyLock::new(|| vec![
    ItemPattern { regex: Regex::new(r"^\s*export\s+component\s+(\w+)").unwrap(), kind: ItemKind::Component },
    ItemPattern { regex: Regex::new(r"^\s*export\s+struct\s+(\w+)").unwrap(), kind: ItemKind::Struct },
    ItemPattern { regex: Regex::new(r"^\s*export\s+enum\s+(\w+)").unwrap(), kind: ItemKind::Enum },
    ItemPattern { regex: Regex::new(r"^\s*(?:pure\s+)?callback\s+([\w-]+)").unwrap(), kind: ItemKind::Callback },
    ItemPattern { regex: Regex::new(r"^\s*(?:in|out|in-out|private)\s+property\s+<[^>]+>\s+([\w-]+)").unwrap(), kind: ItemKind::Property },
]);

static ATTR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*#\[").unwrap());
static PUB_USE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*pub(?:\([^)]+\))?\s+use\s+").unwrap());

/// Parse a Rust source file for public items and their doc comments.
pub fn parse_rs(_path: &Path, content: &str) -> Vec<DocItem> {
    let lines: Vec<&str> = content.lines().collect();
    let mut items = Vec::new();
    let mut doc_buf: Vec<String> = Vec::new();

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("/// ") || trimmed == "///" {
            let doc_line = trimmed.strip_prefix("/// ").unwrap_or("");
            doc_buf.push(doc_line.to_string());
            continue;
        }
        if ATTR_RE.is_match(trimmed) {
            continue;
        }
        if trimmed.is_empty() {
            doc_buf.clear();
            continue;
        }
        if PUB_USE_RE.is_match(trimmed) {
            doc_buf.clear();
            continue;
        }

        for pat in RS_PATTERNS.iter() {
            if let Some(caps) = pat.regex.captures(line) {
                let name = caps[1].to_string();
                let sig = extract_signature(&lines, idx);
                items.push(DocItem {
                    name,
                    kind: pat.kind.clone(),
                    signature: sig,
                    line: idx + 1,
                    doc: doc_buf.join("\n"),
                });
                break;
            }
        }
        doc_buf.clear();
    }

    items
}

/// Parse a Slint source file for exported items and their doc comments.
pub fn parse_slint(_path: &Path, content: &str) -> Vec<DocItem> {
    let lines: Vec<&str> = content.lines().collect();
    let mut items = Vec::new();
    let mut doc_buf: Vec<String> = Vec::new();

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Recognize both /// (inserted by documenter) and // (hand-written) as doc comments
        if trimmed.starts_with("/// ") || trimmed == "///" {
            let doc_line = trimmed.strip_prefix("/// ").unwrap_or("");
            doc_buf.push(doc_line.to_string());
            continue;
        }
        if trimmed.starts_with("// ") {
            let doc_line = trimmed.strip_prefix("// ").unwrap_or("");
            doc_buf.push(doc_line.to_string());
            continue;
        }
        if trimmed.is_empty() {
            doc_buf.clear();
            continue;
        }

        for pat in SLINT_PATTERNS.iter() {
            if let Some(caps) = pat.regex.captures(line) {
                let name = caps[1].to_string();
                items.push(DocItem {
                    name,
                    kind: pat.kind.clone(),
                    signature: trimmed.to_string(),
                    line: idx + 1,
                    doc: doc_buf.join("\n"),
                });
                break;
            }
        }
        doc_buf.clear();
    }

    items
}

/// Extract multi-line signature starting at `start`.
fn extract_signature(lines: &[&str], start: usize) -> String {
    let mut sig = String::new();
    let limit = (start + 8).min(lines.len());
    for i in start..limit {
        let line = lines[i].trim();
        sig.push_str(line);
        if line.contains('{') || line.ends_with(';') {
            break;
        }
        sig.push(' ');
    }
    sig.trim_end_matches('{').trim().to_string()
}

/// Collect all source docs from a project root.
pub fn collect_docs(root: &Path) -> Vec<SourceDoc> {
    let skip_dirs = ["target", ".git", ".cargo", "man", "node_modules"];
    let mut docs = Vec::new();

    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            if !e.file_type().is_dir() { return true; }
            let name = e.file_name().to_string_lossy();
            !skip_dirs.iter().any(|s| *s == name.as_ref())
        })
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() { continue; }
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let items = match ext {
            "rs" => parse_rs(path, &content),
            "slint" => parse_slint(path, &content),
            _ => continue,
        };

        if items.is_empty() { continue; }

        let source = path.strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        docs.push(SourceDoc { source, items });
    }

    docs
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parse_documented_fn() {
        let content = "/// Does something.\npub fn foo() {}";
        let items = parse_rs(&PathBuf::from("test.rs"), content);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "foo");
        assert_eq!(items[0].doc, "Does something.");
        assert!(items[0].is_documented());
    }

    #[test]
    fn parse_undocumented_fn() {
        let content = "pub fn bar() {}";
        let items = parse_rs(&PathBuf::from("test.rs"), content);
        assert_eq!(items.len(), 1);
        assert!(!items[0].is_documented());
    }

    #[test]
    fn skips_pub_use() {
        let content = "pub use crate::other::Thing;";
        let items = parse_rs(&PathBuf::from("test.rs"), content);
        assert!(items.is_empty());
    }

    #[test]
    fn parse_slint_component() {
        let content = "// A button.\nexport component MyButton { }";
        let items = parse_slint(&PathBuf::from("test.slint"), content);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "MyButton");
        assert_eq!(items[0].doc, "A button.");
    }

    #[test]
    fn parse_slint_triple_slash_doc() {
        let content = "/// A button component.\nexport component MyButton { }";
        let items = parse_slint(&PathBuf::from("test.slint"), content);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "MyButton");
        assert_eq!(items[0].doc, "A button component.");
        assert!(items[0].is_documented());
    }

    #[test]
    fn parse_slint_double_slash_still_works() {
        let content = "// A panel.\nexport component Panel { }";
        let items = parse_slint(&PathBuf::from("test.slint"), content);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].doc, "A panel.");
        assert!(items[0].is_documented());
    }

    #[test]
    fn parse_slint_generated_stub_recognized() {
        // Simulates what the inserter generates: `/// component 'Foo'.`
        let content = "/// component `MyButton`.\nexport component MyButton { }";
        let items = parse_slint(&PathBuf::from("test.slint"), content);
        assert_eq!(items.len(), 1);
        assert!(items[0].is_documented(), "inserter-generated /// must be recognized");
    }
}
