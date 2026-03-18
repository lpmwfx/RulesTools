use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Deserialize, Default, Debug)]
/// struct `Edges`.
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
/// struct `RuleEntry`.
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

/// struct `Registry`.
pub struct Registry {
    entries: Vec<RuleEntry>,
    by_file: HashMap<String, usize>,
}

/// Lazy-loaded global registry.
static REGISTRY: OnceLock<Registry> = OnceLock::new();

/// fn `get_registry`.
pub fn get_registry(repo: &Path) -> Result<&'static Registry, String> {
    if let Some(reg) = REGISTRY.get() {
        return Ok(reg);
    }
    let reg = Registry::load(repo)?;
    let _ = REGISTRY.set(reg);
    Ok(REGISTRY.get().unwrap())
}

impl Registry {
    /// fn `load`.
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

    /// fn `from_entries`.
    pub fn from_entries(entries: Vec<RuleEntry>) -> Self {
        let mut by_file = HashMap::with_capacity(entries.len());
        for (i, entry) in entries.iter().enumerate() {
            by_file.insert(entry.file.clone(), i);
        }
        Self { entries, by_file }
    }

    /// fn `len`.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// fn `categories`.
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

    /// fn `rule_count`.
    pub fn rule_count(&self) -> usize {
        self.entries.iter().map(|e| e.rules.len()).sum()
    }

    /// fn `banned_count`.
    pub fn banned_count(&self) -> usize {
        self.entries.iter().map(|e| e.banned.len()).sum()
    }

    /// fn `find_by_file`.
    pub fn find_by_file(&self, file: &str) -> Option<&RuleEntry> {
        self.by_file.get(file).map(|&i| &self.entries[i])
    }

    /// fn `list`.
    pub fn list(&self, category: Option<&str>) -> Vec<&RuleEntry> {
        self.entries
            .iter()
            .filter(|e| category.map_or(true, |cat| e.category == cat))
            .collect()
    }

    /// fn `search`.
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

    /// fn `learning_path`.
    pub fn learning_path(
        &self,
        languages: &[String],
        phase: Option<u8>,
    ) -> Vec<(u8, Vec<&RuleEntry>)> {
        let lang_set: std::collections::HashSet<String> =
            languages.iter().map(|l| l.to_lowercase()).collect();

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

        let relevant: Vec<&RuleEntry> = self
            .entries
            .iter()
            .filter(|e| include_cats.contains(&e.category.to_lowercase()))
            .collect();

        if relevant.is_empty() {
            return Vec::new();
        }

        let mut layer_groups: HashMap<u8, Vec<&RuleEntry>> = HashMap::new();
        for entry in relevant {
            layer_groups.entry(entry.layer).or_default().push(entry);
        }

        let mut layers: Vec<(u8, Vec<&RuleEntry>)> = layer_groups.into_iter().collect();
        layers.sort_by_key(|(layer, _)| *layer);
        for (_, entries) in &mut layers {
            entries.sort_by(|a, b| a.file.cmp(&b.file));
        }

        if let Some(p) = phase {
            if let Some(layer) = layers.into_iter().find(|(l, _)| *l == p) {
                return vec![layer];
            }
            return Vec::new();
        }

        layers
    }
}

// --- Scoring ---

fn bidi_match(token: &str, field: &str) -> bool {
    if token.is_empty() || field.is_empty() {
        return false;
    }
    token.contains(field) || field.contains(token)
}

fn weighted_fields(entry: &RuleEntry) -> Vec<(&str, usize)> {
    let mut fields: Vec<(&str, usize)> = Vec::new();
    fields.push((&entry.file, 3));
    fields.push((&entry.title, 3));
    fields.push((&entry.subtitle, 1));
    for tag in &entry.tags {
        fields.push((tag, 2));
    }
    for concept in &entry.concepts {
        fields.push((concept, 2));
    }
    for kw in &entry.keywords {
        fields.push((kw, 1));
    }
    for axiom in &entry.axioms {
        fields.push((axiom, 2));
    }
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

    if score > 0 && entry.binding {
        score += 10;
    }

    score
}

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
