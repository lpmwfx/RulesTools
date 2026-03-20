# crates/scanner/src/checks/mod.rs

## `pub mod file_size;`

*Line 8 · mod*

mod `file_size`.

---

## `pub mod nesting;`

*Line 10 · mod*

mod `nesting`.

---

## `pub mod debt;`

*Line 12 · mod*

mod `debt`.

---

## `pub mod secrets;`

*Line 14 · mod*

mod `secrets`.

---

## `pub mod magic_numbers;`

*Line 16 · mod*

mod `magic_numbers`.

---

## `pub mod errors;`

*Line 18 · mod*

mod `errors`.

---

## `pub mod doc_required;`

*Line 20 · mod*

mod `doc_required`.

---

## `pub mod string_states;`

*Line 22 · mod*

mod `string_states`.

---

## `pub mod naming;`

*Line 24 · mod*

mod `naming`.

---

## `pub mod modules;`

*Line 26 · mod*

mod `modules`.

---

## `pub mod hardcoded_paths;`

*Line 28 · mod*

mod `hardcoded_paths`.

---

## `pub mod hardcoded_durations;`

*Line 30 · mod*

mod `hardcoded_durations`.

---

## `pub mod hardcoded_urls;`

*Line 32 · mod*

mod `hardcoded_urls`.

---

## `pub mod clone_spam;`

*Line 34 · mod*

mod `clone_spam`.

---

## `pub mod coupling;`

*Line 36 · mod*

mod `coupling`.

---

## `pub mod threading;`

*Line 38 · mod*

mod `threading`.

---

## `pub mod mother_child;`

*Line 40 · mod*

mod `mother_child`.

---

## `pub mod shared_discovery;`

*Line 42 · mod*

mod `shared_discovery`.

---

## `pub mod placement;`

*Line 44 · mod*

mod `placement`.

---

## `pub mod shared_guard;`

*Line 46 · mod*

mod `shared_guard`.

---

## `pub mod unsafe_comment;`

*Line 48 · mod*

mod `unsafe_comment`.

---

## `pub mod layer_violation;`

*Line 50 · mod*

mod `layer_violation`.

---

## `pub mod sibling_import;`

*Line 52 · mod*

mod `sibling_import`.

---

## `pub mod slint_gateway;`

*Line 54 · mod*

mod `slint_gateway`.

---

## `pub mod slint_mother_child;`

*Line 56 · mod*

mod `slint_mother_child`.

---

## `pub mod topology_naming;`

*Line 58 · mod*

mod `topology_naming`.

---

## `pub mod topology_suffix;`

*Line 60 · mod*

mod `topology_suffix`.

---

## `pub mod js_safety;`

*Line 62 · mod*

mod `js_safety`.

---

## `pub mod borrowed_containers;`

*Line 64 · mod*

mod `borrowed_containers`.

---

## `pub mod slint_checks;`

*Line 66 · mod*

mod `slint_checks`.

---

## `pub mod python_checks;`

*Line 68 · mod*

mod `python_checks`.

---

## `pub mod js_modules;`

*Line 70 · mod*

mod `js_modules`.

---

## `pub mod path_deps;`

*Line 72 · mod*

mod `path_deps`.

---

## `pub mod cpp_checks;`

*Line 74 · mod*

mod `cpp_checks`.

---

## `pub mod kotlin_checks;`

*Line 76 · mod*

mod `kotlin_checks`.

---

## `pub mod csharp_checks;`

*Line 78 · mod*

mod `csharp_checks`.

---

## `pub mod css_checks;`

*Line 80 · mod*

mod `css_checks`.

---

## `pub type PerFileCheckFn = fn( file_ctx: &FileContext, lines: &[&str], cfg: &Config, issues: &mut Vec<Issue>, path: &std::path::Path, );`

*Line 83 · type*

Per-file check function signature.

---

## `pub type CrossFileCheckFn = fn( contents: &[(PathBuf, String)], cfg: &Config, issues: &mut Vec<Issue>, );`

*Line 92 · type*

Cross-file check function signature.

---

## `pub type TreeCheckFn = fn( paths: &[PathBuf], cfg: &Config, issues: &mut Vec<Issue>, );`

*Line 99 · type*

Tree-level check function signature.

---

## `pub enum CheckKind`

*Line 106 · enum*

The kind of check — determines how it is dispatched.

---

## `pub struct CheckEntry`

*Line 113 · struct*

A registered check entry.

---

## `pub fn per_file(id: impl Into<String>, languages: Vec<Language>, func: PerFileCheckFn) -> Self`

*Line 121 · fn*

Create a per-file check for specific languages.

---

## `pub fn cross_file(id: impl Into<String>, languages: Vec<Language>, func: CrossFileCheckFn) -> Self`

*Line 126 · fn*

Create a cross-file check.

---

## `pub fn tree(id: impl Into<String>, languages: Vec<Language>, func: TreeCheckFn) -> Self`

*Line 131 · fn*

Create a tree-level check.

---

## `pub fn applies_to(&self, lang: Language) -> bool`

*Line 136 · fn*

Check if this entry applies to the given language.

---

## `pub fn registry() -> Vec<CheckEntry>`

*Line 142 · fn*

Return all registered checks.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
