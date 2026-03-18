# archive/mcp-rules/src/registry.rs

## `pub struct Edges`

*Line 7 · struct*

struct `Edges`.

---

## `pub struct RuleEntry`

*Line 22 · struct*

struct `RuleEntry`.

---

## `pub struct Registry`

*Line 56 · struct*

struct `Registry`.

---

## `pub fn load(repo: &Path) -> Result<Self, String>`

*Line 63 · fn*

Load registry from register.jsonl in the given repo path.

---

## `pub fn from_entries(entries: Vec<RuleEntry>) -> Self`

*Line 86 · fn*

Build registry from a vec of entries (used by load and tests).

---

## `pub fn len(&self) -> usize`

*Line 95 · fn*

fn `len`.

---

## `pub fn categories(&self) -> Vec<String>`

*Line 100 · fn*

Sorted unique categories.

---

## `pub fn rule_count(&self) -> usize`

*Line 113 · fn*

Total RULE: markers across all entries.

---

## `pub fn banned_count(&self) -> usize`

*Line 118 · fn*

Total BANNED: markers across all entries.

---

## `pub fn find_by_file(&self, file: &str) -> Option<&RuleEntry>`

*Line 123 · fn*

O(1) lookup by file path.

---

## `pub fn list(&self, category: Option<&str>) -> Vec<&RuleEntry>`

*Line 128 · fn*

List entries, optionally filtered by category.

---

## `pub fn search( &self, query: &str, category: Option<&str>, limit: usize, ) -> Vec<(&RuleEntry, usize)>`

*Line 136 · fn*

Weighted search. Returns (entry, score) pairs sorted descending.

---

## `pub fn learning_path( &self, languages: &[String], phase: Option<u8>, ) -> Vec<(u8, Vec<&RuleEntry>)>`

*Line 167 · fn*

Learning path: entries grouped by layer for given languages.
Returns vec of (layer_number, entries) sorted ascending.

---

