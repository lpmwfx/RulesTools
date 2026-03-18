# RulesTools — CLAUDE.md

Unified static code scanner — one Rust workspace, one binary.
Enforces coding rules from the [Rules repo](https://github.com/lpmwfx/Rules).

## Architecture

```
RulesTools/
├── crates/scanner/       rulestools-scanner lib (50 checks, 273 tests)
├── crates/documenter/    rulestools-documenter lib (stub)
├── apps/cli/             rulestools binary — CLI + MCP servers
│   └── src/
│       ├── main.rs           clap structs + dispatch
│       ├── commands/         CLI command handlers (scan, project, issue, generate, hook)
│       ├── scaffold.rs       project scaffolding (init/new/update/upgrade)
│       ├── publish.rs        publish/sync/check
│       └── mcp/              MCP servers (embedded, no subprocess)
│           ├── mod.rs         shared protocol (JSON-RPC stdio)
│           ├── tools/         rulestools mcp-tools (22 tools)
│           └── rules/         rulestools mcp-rules (7 tools)
├── archive/              old separate MCP binaries (reference only)
└── Rules/                markdown rule data (separate repo)
```

## One binary, three modes

```bash
rulestools scan .              # CLI: scan project
rulestools mcp-tools           # MCP server: scan, setup, init, publish (stdio)
rulestools mcp-rules           # MCP server: rule lookup, search (stdio)
```

MCP handlers call the SAME internal functions as CLI commands — no subprocess, no version drift.

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

## Checks (50 active)

| Check ID | Type | Languages |
|---|---|---|
| `global/file-limits` | PerFile | all |
| `global/nesting` | PerFile | all |
| `global/tech-debt` | PerFile | all |
| `global/secrets` | PerFile | all |
| `global/install-architecture/no-path-deps` | Tree | all |
| `rust/constants/no-magic-number` | PerFile | rust |
| `rust/constants/no-hardcoded-path` | PerFile | rust |
| `rust/constants/no-hardcoded-duration` | PerFile | rust |
| `rust/constants/no-hardcoded-url` | PerFile | rust |
| `rust/errors/no-unwrap` | PerFile | rust |
| `rust/docs/doc-required` | PerFile | rust |
| `rust/types/no-string-match` | PerFile | rust |
| `rust/types/no-borrowed-container` | PerFile | rust |
| `rust/naming/no-noise-names` | PerFile | rust |
| `rust/modules/no-utils` | PerFile | rust |
| `rust/modules/no-sibling-coupling` | PerFile | rust |
| `rust/modules/shared-guard` | PerFile | rust |
| `rust/modules/shared-candidate` | CrossFile | rust |
| `rust/modules/no-sibling-import` | CrossFile | rust |
| `rust/ownership/clone-spam` | PerFile | rust |
| `rust/safety/unsafe-needs-comment` | PerFile | rust |
| `rust/threading/no-static-mut` | PerFile | rust |
| `rust/threading/no-fire-and-forget` | PerFile | rust |
| `uiux/mother-child/mother-too-many-fns` | PerFile | rust |
| `topology/placement` | PerFile | all |
| `topology/layer-violation` | CrossFile | rust |
| `topology/naming` | Tree | all |
| `topology/suffix` | PerFile | rust |
| `slint/docs/doc-required` | PerFile | slint |
| `slint/tokens/zero-literal` | PerFile | slint |
| `slint/globals/structure` | PerFile | slint |
| `slint/strings/no-hardcoded-string` | PerFile | slint |
| `uiux/state-flow/single-gateway` | Tree | slint |
| `uiux/mother-child/child-has-state` | PerFile | slint |
| `python/types/missing-annotations` | PerFile | python |
| `python/naming/conventions` | PerFile | python |
| `python/validation/boundary-check` | PerFile | python |
| `js/safety/no-var` | PerFile | js/ts |
| `js/safety/no-console-log` | PerFile | js/ts |
| `js/safety/no-eval` | PerFile | js/ts |
| `js/jsdoc/type-required` | PerFile | js/ts |
| `js/modules/no-require` | PerFile | js/ts |
| `cpp/naming/conventions` | PerFile | cpp |
| `cpp/docs/doc-required` | PerFile | cpp |
| `cpp/safety/no-raw-memory` | PerFile | cpp |
| `kotlin/naming/conventions` | PerFile | kotlin |
| `kotlin/docs/doc-required` | PerFile | kotlin |
| `csharp/naming/conventions` | PerFile | csharp |
| `csharp/docs/doc-required` | PerFile | csharp |
| `css/tokens/zero-literal` | PerFile | css |

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
rulestools setup .             # install hooks + config
rulestools init . --kind tool  # scaffold new project
rulestools mcp-tools           # start MCP tools server (stdio)
rulestools mcp-rules           # start MCP rules server (stdio)
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
