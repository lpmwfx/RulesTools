# apps/cli/src/mcp/rules/registry.rs

## `pub struct Edges`

*Line 8 · struct*

struct `Edges`.

---

## `pub struct RuleEntry`

*Line 23 · struct*

struct `RuleEntry`.

---

## `pub struct Registry`

*Line 57 · struct*

struct `Registry`.

---

## `pub fn get_registry(repo: &Path) -> Result<&'static Registry, String>`

*Line 66 · fn*

fn `get_registry`.

---

## `pub fn load(repo: &Path) -> Result<Self, String>`

*Line 77 · fn*

fn `load`.

---

## `pub fn from_entries(entries: Vec<RuleEntry>) -> Self`

*Line 100 · fn*

fn `from_entries`.

---

## `pub fn len(&self) -> usize`

*Line 109 · fn*

fn `len`.

---

## `pub fn categories(&self) -> Vec<String>`

*Line 114 · fn*

fn `categories`.

---

## `pub fn rule_count(&self) -> usize`

*Line 127 · fn*

fn `rule_count`.

---

## `pub fn banned_count(&self) -> usize`

*Line 132 · fn*

fn `banned_count`.

---

## `pub fn find_by_file(&self, file: &str) -> Option<&RuleEntry>`

*Line 137 · fn*

fn `find_by_file`.

---

## `pub fn list(&self, category: Option<&str>) -> Vec<&RuleEntry>`

*Line 142 · fn*

fn `list`.

---

## `pub fn search( &self, query: &str, category: Option<&str>, limit: usize, ) -> Vec<(&RuleEntry, usize)>`

*Line 150 · fn*

fn `search`.

---

## `pub fn learning_path( &self, languages: &[String], phase: Option<u8>, ) -> Vec<(u8, Vec<&RuleEntry>)>`

*Line 180 · fn*

fn `learning_path`.

---

