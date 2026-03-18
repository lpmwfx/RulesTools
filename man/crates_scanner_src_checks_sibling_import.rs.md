# crates/scanner/src/checks/sibling_import.rs

## `pub fn check( contents: &[(PathBuf, String)], _cfg: &Config, issues: &mut Vec<Issue>, )`

*Line 11 · fn*

Check that child modules do not import sibling children directly.

Children must route through mother (mod.rs) or extract to shared/.
`use super::<sibling>` in a child file is an error.

---

