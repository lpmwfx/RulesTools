use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Deserialize, Default, Debug)]
pub struct Edges {
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub required_by: Vec<String>,
    #[serde(default)]
    pub feeds: Vec<String>,
    #[serde(default)]
    pub fed_by: Vec<String>,
    #[serde(default)]
    pub related: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct RuleEntry {
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default = "default_layer")]
    pub layer: u8,
    #[serde(default)]
    pub binding: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub concepts: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub axioms: Vec<String>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub banned: Vec<String>,
    #[serde(default)]
    pub edges: Edges,
}

fn default_layer() -> u8 {
    4
}

pub struct Registry {
    entries: Vec<RuleEntry>,
    by_file: HashMap<String, usize>,
}

impl Registry {
    /// Load registry from register.jsonl in the given repo path.
    pub fn load(repo: &Path) -> Result<Self, String> {
        let jsonl_path = repo.join("register.jsonl");
        let content = std::fs::read_to_string(&jsonl_path)
            .map_err(|e| format!("Cannot read register.jsonl: {e}"))?;

        let mut entries = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<RuleEntry>(line) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    eprintln!("registry: skipping bad line: {e}");
                }
            }
        }

        Ok(Self::from_entries(entries))
    }

    /// Build registry from a vec of entries (used by load and tests).
    pub fn from_entries(entries: Vec<RuleEntry>) -> Self {
        let mut by_file = HashMap::with_capacity(entries.len());
        for (i, entry) in entries.iter().enumerate() {
            by_file.insert(entry.file.clone(), i);
        }
        Self { entries, by_file }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Sorted unique categories.
    pub fn categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self
            .entries
            .iter()
            .map(|e| e.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        cats.sort();
        cats
    }

    /// Total RULE: markers across all entries.
    pub fn rule_count(&self) -> usize {
        self.entries.iter().map(|e| e.rules.len()).sum()
    }

    /// Total BANNED: markers across all entries.
    pub fn banned_count(&self) -> usize {
        self.entries.iter().map(|e| e.banned.len()).sum()
    }

    /// O(1) lookup by file path.
    pub fn find_by_file(&self, file: &str) -> Option<&RuleEntry> {
        self.by_file.get(file).map(|&i| &self.entries[i])
    }

    /// List entries, optionally filtered by category.
    pub fn list(&self, category: Option<&str>) -> Vec<&RuleEntry> {
        self.entries
            .iter()
            .filter(|e| category.map_or(true, |cat| e.category == cat))
            .collect()
    }

    /// Weighted search. Returns (entry, score) pairs sorted descending.
    pub fn search(
        &self,
        query: &str,
        category: Option<&str>,
        limit: usize,
    ) -> Vec<(&RuleEntry, usize)> {
        let tokens: Vec<String> = query.lower_tokens();
        if tokens.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(&RuleEntry, usize)> = Vec::new();
        for entry in &self.entries {
            if let Some(cat) = category {
                if entry.category != cat {
                    continue;
                }
            }
            let score = score_entry(entry, &tokens);
            if score > 0 {
                scored.push((entry, score));
            }
        }

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.truncate(limit);
        scored
    }

    /// Learning path: entries grouped by layer for given languages.
    /// Returns vec of (layer_number, entries) sorted ascending.
    pub fn learning_path(
        &self,
        languages: &[String],
        phase: Option<u8>,
    ) -> Vec<(u8, Vec<&RuleEntry>)> {
        let lang_set: std::collections::HashSet<String> =
            languages.iter().map(|l| l.to_lowercase()).collect();

        // Always include foundational categories
        let mut include_cats = lang_set;
        for cat in &[
            "global",
            "project-files",
            "gateway",
            "adapter",
            "core",
            "pal",
        ] {
            include_cats.insert(cat.to_string());
        }

        // Collect relevant entries
        let relevant: Vec<&RuleEntry> = self
            .entries
            .iter()
            .filter(|e| include_cats.contains(&e.category.to_lowercase()))
            .collect();

        if relevant.is_empty() {
            return Vec::new();
        }

        // Group by layer
        let mut layer_groups: HashMap<u8, Vec<&RuleEntry>> = HashMap::new();
        for entry in relevant {
            layer_groups.entry(entry.layer).or_default().push(entry);
        }

        // Sort layers ascending, entries within layer by file
        let mut layers: Vec<(u8, Vec<&RuleEntry>)> = layer_groups.into_iter().collect();
        layers.sort_by_key(|(layer, _)| *layer);
        for (_, entries) in &mut layers {
            entries.sort_by(|a, b| a.file.cmp(&b.file));
        }

        if let Some(p) = phase {
            // Find the layer matching this phase number
            if let Some(layer) = layers.into_iter().find(|(l, _)| *l == p) {
                return vec![layer];
            }
            return Vec::new();
        }

        layers
    }
}

// --- Scoring ---

/// Bidirectional substring match.
fn bidi_match(token: &str, field: &str) -> bool {
    if token.is_empty() || field.is_empty() {
        return false;
    }
    token.contains(field) || field.contains(token)
}

/// Build weighted fields from an entry.
fn weighted_fields(entry: &RuleEntry) -> Vec<(&str, usize)> {
    let mut fields: Vec<(&str, usize)> = Vec::new();

    // File path (weight 3)
    fields.push((&entry.file, 3));
    // Title (weight 3)
    fields.push((&entry.title, 3));
    // Subtitle (weight 1)
    fields.push((&entry.subtitle, 1));
    // Tags (weight 2 each)
    for tag in &entry.tags {
        fields.push((tag, 2));
    }
    // Concepts (weight 2 each)
    for concept in &entry.concepts {
        fields.push((concept, 2));
    }
    // Keywords (weight 1 each)
    for kw in &entry.keywords {
        fields.push((kw, 1));
    }
    // Axioms (weight 2 each)
    for axiom in &entry.axioms {
        fields.push((axiom, 2));
    }
    // Category (weight 1)
    fields.push((&entry.category, 1));

    fields
}

fn score_entry(entry: &RuleEntry, tokens: &[String]) -> usize {
    let fields = weighted_fields(entry);
    let mut score = 0usize;

    for token in tokens {
        for &(field_text, weight) in &fields {
            let field_lower = field_text.to_lowercase();
            if bidi_match(token, &field_lower) {
                score += weight;
            }
        }
    }

    // Binding bonus
    if score > 0 && entry.binding {
        score += 10;
    }

    score
}

/// Helper trait for splitting query into lowercase tokens.
trait LowerTokens {
    fn lower_tokens(&self) -> Vec<String>;
}

impl LowerTokens for str {
    fn lower_tokens(&self) -> Vec<String> {
        self.to_lowercase()
            .split_whitespace()
            .map(String::from)
            .collect()
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(file: &str, category: &str, title: &str) -> RuleEntry {
        RuleEntry {
            file: file.into(),
            category: category.into(),
            title: title.into(),
            subtitle: String::new(),
            layer: 4,
            binding: false,
            tags: Vec::new(),
            concepts: Vec::new(),
            keywords: Vec::new(),
            axioms: Vec::new(),
            rules: Vec::new(),
            banned: Vec::new(),
            edges: Edges::default(),
        }
    }

    #[test]
    fn load_handles_missing_fields() {
        let json = r#"{"file": "test/minimal.md"}"#;
        let e: RuleEntry = serde_json::from_str(json).unwrap();
        assert_eq!(e.file, "test/minimal.md");
        assert_eq!(e.category, "");
        assert_eq!(e.layer, 4); // default
        assert!(!e.binding);
        assert!(e.tags.is_empty());
    }

    #[test]
    fn from_entries_builds_index() {
        let entries = vec![
            entry("global/types.md", "global", "Types"),
            entry("rust/errors.md", "rust", "Errors"),
        ];
        let reg = Registry::from_entries(entries);
        assert_eq!(reg.len(), 2);
        assert!(reg.find_by_file("global/types.md").is_some());
        assert!(reg.find_by_file("rust/errors.md").is_some());
        assert!(reg.find_by_file("nonexistent.md").is_none());
    }

    #[test]
    fn categories_sorted() {
        let entries = vec![
            entry("rust/a.md", "rust", "A"),
            entry("global/b.md", "global", "B"),
            entry("python/c.md", "python", "C"),
            entry("global/d.md", "global", "D"),
        ];
        let reg = Registry::from_entries(entries);
        assert_eq!(reg.categories(), vec!["global", "python", "rust"]);
    }

    #[test]
    fn stats_count_rules_and_banned() {
        let mut e1 = entry("a.md", "global", "A");
        e1.rules = vec!["R1".into(), "R2".into()];
        e1.banned = vec!["B1".into()];
        let mut e2 = entry("b.md", "global", "B");
        e2.rules = vec!["R3".into()];
        e2.banned = vec!["B2".into(), "B3".into()];

        let reg = Registry::from_entries(vec![e1, e2]);
        assert_eq!(reg.rule_count(), 3);
        assert_eq!(reg.banned_count(), 3);
    }

    #[test]
    fn matches_bidirectional() {
        assert!(bidi_match("type", "types")); // token in field
        assert!(bidi_match("types", "type")); // field in token
        assert!(!bidi_match("type", "error"));
        assert!(!bidi_match("type", ""));     // empty field
        assert!(!bidi_match("", "types"));    // empty token
    }

    #[test]
    fn score_weights_correct() {
        let mut e = entry("rust/types.md", "rust", "Type System");
        e.tags = vec!["types".into()];
        e.keywords = vec!["typecheck".into()];

        let tokens = vec!["type".into()];
        let score = score_entry(&e, &tokens);

        // file "rust/types.md" contains "type" → +3
        // title "type system" contains "type" → +3
        // tag "types" contains "type" → +2
        // keyword "typecheck" contains "type" → +1
        // category "rust" does not match → 0
        assert_eq!(score, 9);
    }

    #[test]
    fn score_binding_bonus() {
        let mut e = entry("global/binding.md", "global", "Binding Rule");
        e.binding = true;
        e.tags = vec!["test".into()];

        let tokens = vec!["test".into()];
        let score = score_entry(&e, &tokens);
        // tag match → +2, binding bonus → +10 = 12
        assert!(score >= 12);
    }

    #[test]
    fn search_respects_category() {
        let entries = vec![
            entry("rust/a.md", "rust", "Rust Types"),
            entry("python/a.md", "python", "Python Types"),
        ];
        let reg = Registry::from_entries(entries);

        let results = reg.search("types", Some("rust"), 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.category, "rust");
    }

    #[test]
    fn search_limit() {
        let entries = vec![
            entry("a/1.md", "a", "Types one"),
            entry("a/2.md", "a", "Types two"),
            entry("a/3.md", "a", "Types three"),
            entry("a/4.md", "a", "Types four"),
        ];
        let reg = Registry::from_entries(entries);
        let results = reg.search("types", None, 3);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn learning_path_groups_by_layer() {
        let mut e1 = entry("global/a.md", "global", "A");
        e1.layer = 1;
        let mut e2 = entry("global/b.md", "global", "B");
        e2.layer = 2;
        let mut e3 = entry("global/c.md", "global", "C");
        e3.layer = 3;

        let reg = Registry::from_entries(vec![e1, e2, e3]);
        let path = reg.learning_path(&["rust".into()], None);
        assert_eq!(path.len(), 3);
        assert_eq!(path[0].0, 1);
        assert_eq!(path[1].0, 2);
        assert_eq!(path[2].0, 3);
    }

    #[test]
    fn learning_path_phase_filter() {
        let mut e1 = entry("global/a.md", "global", "A");
        e1.layer = 1;
        let mut e2 = entry("global/b.md", "global", "B");
        e2.layer = 2;
        let mut e3 = entry("global/c.md", "global", "C");
        e3.layer = 3;

        let reg = Registry::from_entries(vec![e1, e2, e3]);
        let path = reg.learning_path(&["rust".into()], Some(2));
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].0, 2);
    }

    #[test]
    fn learning_path_includes_global() {
        let mut e1 = entry("global/a.md", "global", "A");
        e1.layer = 1;
        let mut e2 = entry("rust/b.md", "rust", "B");
        e2.layer = 2;
        let mut e3 = entry("python/c.md", "python", "C");
        e3.layer = 1;

        let reg = Registry::from_entries(vec![e1, e2, e3]);
        let path = reg.learning_path(&["rust".into()], None);

        // Should include global/a.md and rust/b.md, NOT python/c.md
        let all_files: Vec<&str> = path
            .iter()
            .flat_map(|(_, entries)| entries.iter().map(|e| e.file.as_str()))
            .collect();
        assert!(all_files.contains(&"global/a.md"));
        assert!(all_files.contains(&"rust/b.md"));
        assert!(!all_files.contains(&"python/c.md"));
    }
}
