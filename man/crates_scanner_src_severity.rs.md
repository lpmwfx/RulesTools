# crates/scanner/src/severity.rs

## `pub struct SeverityResolver`

*Line 10 · struct*

Maps check IDs to severity based on ProjectKind.

Same checks, different enforcement per project kind.
Tool = most relaxed, SlintApp = full enforcement.

---

## `pub fn for_kind(kind: ProjectKind) -> Self`

*Line 16 · fn*

Build resolver for a given ProjectKind.

---

## `pub fn resolve(&self, rule_id: &str, default: Severity) -> Severity`

*Line 48 · fn*

Resolve the final severity for a check.

Matches on exact rule_id first, then tries category prefix
(everything up to the last `/`). This handles checks that emit
sub-IDs like `rust/errors/no-expect` from the `rust/errors/no-unwrap` check.

---

