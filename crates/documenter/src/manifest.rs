use serde::{Deserialize, Serialize};

/// Kind of documented item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemKind {
    Fn,
    Struct,
    Enum,
    Trait,
    Type,
    Mod,
    Const,
    Component,
    Property,
    Callback,
}

impl ItemKind {
    /// Display label.
    pub fn label(&self) -> &'static str {
        match self {
            ItemKind::Fn => "fn",
            ItemKind::Struct => "struct",
            ItemKind::Enum => "enum",
            ItemKind::Trait => "trait",
            ItemKind::Type => "type",
            ItemKind::Mod => "mod",
            ItemKind::Const => "const",
            ItemKind::Component => "component",
            ItemKind::Property => "property",
            ItemKind::Callback => "callback",
        }
    }
}

/// A single documented item extracted from source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocItem {
    pub name: String,
    pub kind: ItemKind,
    pub signature: String,
    pub line: usize,
    pub doc: String,
}

impl DocItem {
    /// Whether this item has documentation.
    pub fn is_documented(&self) -> bool {
        !self.doc.is_empty()
    }
}

/// All documented items from one source file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDoc {
    pub source: String,
    pub items: Vec<DocItem>,
}

/// Aggregated manifest for the entire project.
#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub project: String,
    pub files: Vec<ManifestEntry>,
}

/// Per-file entry in the manifest.
#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub source: String,
    pub item_count: usize,
    pub undocumented: usize,
}
