# crates/documenter/src/manifest.rs

## `pub enum ItemKind`

*Line 5 · enum*

Kind of documented item.

---

## `pub fn label(&self) -> &'static str`

*Line 20 · fn*

Display label.

---

## `pub struct DocItem`

*Line 38 · struct*

A single documented item extracted from source.

---

## `pub fn is_documented(&self) -> bool`

*Line 48 · fn*

Whether this item has documentation.

---

## `pub struct SourceDoc`

*Line 55 · struct*

All documented items from one source file.

---

## `pub struct Manifest`

*Line 62 · struct*

Aggregated manifest for the entire project.

---

## `pub struct ManifestEntry`

*Line 69 · struct*

Per-file entry in the manifest.

---

