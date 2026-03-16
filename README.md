# RulesTools

Unified static code scanner for Rust and Slint projects. One workspace — scanner, documenter, CLI, and MCP servers.

## Install

```bash
# CLI + MCP servers
cargo install --git https://github.com/lpmwfx/RulesTools rulestools
cargo install --git https://github.com/lpmwfx/RulesTools mcp-tools
cargo install --git https://github.com/lpmwfx/RulesTools mcp-rules

# Documentation viewer (separate repo)
cargo install --git https://github.com/lpmwfx/ManViewer
```

## build.rs integration

Add to your `Cargo.toml`:

```toml
[build-dependencies]
rulestools-scanner    = { git = "https://github.com/lpmwfx/RulesTools" }
rulestools-documenter = { git = "https://github.com/lpmwfx/RulesTools" }
```

```rust
// build.rs
fn main() {
    rulestools_documenter::document_project();
    rulestools_scanner::scan_project();
}
```

Scanner runs at compile time — violations appear as `cargo:warning` lines.

## Project auto-detection

The scanner auto-detects project kind and applies the right check-set:

| Signal | Kind | Checks |
|---|---|---|
| `ui/*.slint` or `slint-build` | SlintApp | Full (gateway, mother-child, tokens) |
| `src/main.rs`, no slint | CliApp | No gateway/UI checks |
| Library crate | Library | Self-contained, no topology |
| Explicit `kind = "tool"` | Tool | Most relaxed |

Override in `proj/rulestools.toml`:

```toml
[project]
kind = "slint-app"
```

## 18 checks

| Category | Checks |
|---|---|
| Global | file-limits, nesting, tech-debt, secrets |
| Constants | magic-number, hardcoded-path, hardcoded-duration, hardcoded-url |
| Errors | no-unwrap, no-expect, no-panic, no-todo |
| Types | no-string-match, no-string-compare |
| Naming | no-noise-names, bool-prefix, unsafe-comment |
| Modules | no-utils, no-inline-mod, no-sibling-coupling |
| Threading | no-static-mut, arc-rc-comment |
| Ownership | clone-spam |
| Mother-child | mother-too-many-fns, child-owns-state |
| Cross-file | shared-candidate (duplicate pub fn detection) |

## CLI

```bash
rulestools scan .          # scan project, write proj/ISSUES
rulestools check .         # pre-commit mode (exit 1 on errors)
rulestools list .          # show checks and status
rulestools detect .        # show auto-detected project kind
rulestools gen .           # generate man/ documentation
```

## MCP servers

```bash
# Add to Claude Code (global)
claude mcp add --transport stdio --scope user rules -- mcp-rules
claude mcp add --transport stdio --scope user rulestools -- mcp-tools
```

**mcp-tools**: scan_file, scan_tree, check_staged, setup, security_scan, init_project

**mcp-rules**: help, get_rule, search_rules, list_rules, get_context

## Documentation viewer

Generate `man/` docs with `rulestools gen .`, then browse with [ManViewer](https://github.com/lpmwfx/ManViewer):

```bash
rulestools gen /path/to/project
man-viewer /path/to/project
```

## Workspace structure

```
RulesTools/
├── crates/scanner/       18 checks, 104 tests
├── crates/documenter/    parser + generator + docgen
├── apps/cli/             rulestools binary
├── apps/mcp-tools/       MCP server (scan, setup, init)
├── apps/mcp-rules/       MCP server (rule lookup)
```

## Ecosystem

| Component | Repository |
|---|---|
| **RulesTools** | This repo — scanner, documenter, CLI, MCP |
| **ManViewer** | [lpmwfx/ManViewer](https://github.com/lpmwfx/ManViewer) — GUI docs viewer |
| **Rules** | [lpmwfx/Rules](https://github.com/lpmwfx/Rules) — markdown rule definitions |
