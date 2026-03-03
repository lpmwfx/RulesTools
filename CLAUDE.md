# RulesTools — CLAUDE.md

Static code scanner that enforces the rules defined in the
[Rules MCP server](https://github.com/lpmwfx/Rules) (tools: `mcp__rules__*`).

## What this tool does

`rulestools scan` walks a project, checks source files against coding rules,
and writes violations to `proj/ISSUES` in the scanned project.

Every issue line follows this format:

```
path/to/file.rs:42:5: error rust/errors/no-unwrap: unwrap() in non-test code
```

## Rule ID → MCP lookup

The rule ID embedded in every issue (`rust/errors/no-unwrap`) maps directly
to a Rules MCP file. The mapping is deterministic:

```
Take the first two path segments of the rule ID and append .md

  rust/errors/no-unwrap          →  rust/errors.md
  rust/modules/no-sibling-coupling →  rust/modules.md
  global/nesting                 →  global/nesting.md
  global/file-limits/rs          →  global/file-limits.md
  uiux/state-flow/no-callback-logic → uiux/state-flow.md
  uiux/tokens/no-hardcoded-color →  uiux/tokens.md
```

To fetch the full rule and fix guidance, call:

```
mcp__rules__get_rule(file="rust/errors.md")
```

## When fixing issues

1. Open `proj/ISSUES` — issues marked `[NEW]` were introduced by the last change.
2. For each rule ID, derive the MCP file path (rule above) and call
   `mcp__rules__get_rule` to get the full RULE/BANNED context.
3. Fix the violation. Re-run `rulestools scan` to confirm it disappears.

## Scanners and their source rules

| Scanner file | Checks | Source rule (MCP) |
|---|---|---|
| `rust/checks/errors.py` | unwrap/expect/panic/Box<dyn Error> | `rust/errors.md` |
| `rust/checks/naming.py` | banned names, bool prefix, unsafe comment | `rust/naming.md` |
| `rust/checks/modules.py` | utils.rs, inline mod | `rust/modules.md` |
| `rust/checks/types.py` | &Vec, &String, println!, static mut | `rust/types.md` |
| `rust/checks/threading.py` | fire-and-forget, Arc/Rc comment | `rust/threading.md` + `rust/ownership.md` |
| `rust/checks/coupling.py` | use super::sibling, pub(super) | `rust/modules.md` |
| `slint/checks/tokens.py` | hardcoded colors/sizes | `uiux/tokens.md` |
| `slint/checks/structure.py` | multiple components per file | `global/module-tree.md` |
| `slint/checks/events.py` | callback logic, state mutations | `uiux/state-flow.md` |
| `js/checks/modules.py` | require/CJS, mutable exports | `js/modules.md` |
| `js/checks/safety.py` | eval, layer violation, promise | `js/safety.md` |
| `js/checks/validation.py` | JSON.parse/fetch without schema | `js/validation.md` |
| `css/checks/tokens.py` | hardcoded colors, !important, font px | `css/custom-properties.md` |
| `python/checks/types_check.py` | future annotations, Optional, bare except | `python/types.md` |
| `python/checks/nesting_check.py` | indent depth | `global/nesting.md` |
| `python/checks/validation_check.py` | json.loads/response.json without pydantic, raw dict params | `python/validation.md` |
| `kotlin/checks/safety.py` | !!, java.* imports, multi-class | `kotlin/encapsulation.md` |
| `common/nesting.py` | brace depth (all languages) | `global/nesting.md` |
| `common/file_size.py` | line count limits | `global/file-limits.md` |

## CLI

```bash
rulestools detect [PATH]          # auto-detect languages → proj/rulestools.toml
rulestools scan [PATH]            # one-shot scan, writes proj/ISSUES
rulestools scan --watch [PATH]    # daemon, re-scans on file change
rulestools init [PATH]            # installs VSCode task + pre-commit hook
```
