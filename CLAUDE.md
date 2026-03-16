# RulesTools — CLAUDE.md

Unified static code scanner — one Rust workspace replacing 7 separate repos.
Enforces coding rules from the [Rules repo](https://github.com/lpmwfx/Rules).

## Architecture

```
RulesTools/
├── crates/scanner/       rulestools-scanner lib (18 checks, 104 tests)
├── crates/documenter/    rulestools-documenter lib (stub)
├── apps/cli/             rulestools binary (scan/check/list/detect)
├── apps/mcp-rules/       MCP server for rule lookup (stub)
├── apps/mcp-tools/       MCP server for scan/init (stub)
└── Rules/                markdown rule data (separate repo)
```

## Project auto-detection

The scanner auto-detects project kind from filesystem structure:

| Signal | Kind | Check-set |
|---|---|---|
| `ui/*.slint` or `slint-build` in Cargo.toml | SlintApp | Full (gateway, mother-child, tokens, UI) |
| `src/main.rs` or `[[bin]]`, no slint | CliApp | No gateway/mother-child/UI |
| Library crate (no binary) | Library | Self-contained, no topology, allow println |
| Explicit `[project].kind = "tool"` | Tool | Most relaxed |

Override in `proj/rulestools.toml`:
```toml
[project]
kind = "cli"  # slint-app | cli | library | lib | tool
```

## Checks (18 active)

| Check ID | Type | Languages |
|---|---|---|
| `global/file-limits` | PerFile | all |
| `global/nesting` | PerFile | all |
| `global/tech-debt` | PerFile | all |
| `global/secrets` | PerFile | all |
| `rust/constants/no-magic-number` | PerFile | rust |
| `rust/constants/no-hardcoded-path` | PerFile | rust |
| `rust/constants/no-hardcoded-duration` | PerFile | rust |
| `rust/constants/no-hardcoded-url` | PerFile | rust |
| `rust/errors/no-unwrap` | PerFile | rust |
| `rust/docs/doc-required` | PerFile | rust |
| `rust/types/no-string-match` | PerFile | rust |
| `rust/naming/no-noise-names` | PerFile | rust |
| `rust/modules/no-utils` | PerFile | rust |
| `rust/modules/no-sibling-coupling` | PerFile | rust |
| `rust/ownership/clone-spam` | PerFile | rust |
| `rust/threading/no-static-mut` | PerFile | rust |
| `uiux/mother-child/mother-too-many-fns` | PerFile | rust |
| `rust/modules/shared-candidate` | CrossFile | rust |

## Key check: shared-candidate

Cross-file check that finds duplicate `pub fn` names across sibling files.
Forces AI DEV to discover existing code before creating duplicates.
Functions like `new`, `default`, `from` etc. are exempt (standard Rust patterns).

## CLI

```bash
rulestools scan .              # scan project, write proj/ISSUES
rulestools scan . --deny       # fail on errors (for CI)
rulestools check .             # same as scan --deny (pre-commit)
rulestools list .              # show all checks and status
rulestools detect .            # show auto-detected project kind + layout
```

## build.rs integration

```rust
fn main() {
    rulestools_scanner::scan_project();
}
```

## Rule ID → MCP lookup

```
rust/errors/no-unwrap          →  mcp__rules__get_rule("rust/errors.md")
global/nesting                 →  mcp__rules__get_rule("global/nesting.md")
uiux/mother-child/...         →  mcp__rules__get_rule("uiux/mother-child.md")
```
