# RulesTools — CLAUDE.md

## MANDATORY — read proj/ first

Before any work, READ the superprojekt design files:
1. `../proj/PROJECT` — full architecture, scaffold design, topology, layer rules, type suffixes
2. `../proj/TODO` — current tasks
3. `../proj/RULES` — active rules

The scaffold design (project kinds, topology variants, proj/ files, layer import rules) is in `proj/PROJECT` § "Scaffold design". That is the source of truth — not this file, not memory.

## Architecture

```
RulesTools/
├── crates/scanner/       rulestools-scanner lib (50 checks, 275 tests)
├── crates/documenter/    rulestools-documenter lib (parser + man/ generator)
├── apps/cli/             rulestools binary — CLI + embedded MCP servers
│   └── src/
│       ├── main.rs           clap structs + dispatch
│       ├── commands/         CLI handlers (scan, project, issue, generate, hook)
│       ├── scaffold.rs       project scaffolding (init/new/update/upgrade)
│       └── mcp/              MCP servers (embedded, no subprocess)
│           ├── tools/        22 tools
│           └── rules/        7 tools
```

One binary, three modes: `rulestools scan .` / `rulestools mcp-tools` / `rulestools mcp-rules`

## build.rs integration

Every scaffolded project gets this build.rs:
```rust
fn main() {
    rulestools_scanner::scan_project();
    rulestools_documenter::document_project();
}
```

Scanner: validates code against 50 checks.
Documenter: inserts stub `///` docs, generates `man/` directory, fails build if undocumented pub items remain.

## Project auto-detection

| Signal | Kind | Check-set |
|---|---|---|
| `ui/*.slint` or `slint-build` in Cargo.toml | SlintApp | Full enforcement |
| `src/main.rs` or `[[bin]]`, no slint | CliApp | No UI checks |
| Library crate (no binary) | Library | Self-contained |
| `[project].kind = "tool"` | Tool | Most relaxed |

Override in `proj/rulestools.toml`: `[project] kind = "cli"`


---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
