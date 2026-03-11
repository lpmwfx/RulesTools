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
| `common/topology.py` | type suffix matches layer folder — `_adp`/`_core`/`_gtw` etc. (all languages) | `global/topology.md` |
| `common/import_direction.py` | import DAG violations — folder-path + type-suffix (_adp/_core etc.) in import lines | `global/topology.md` |
| `common/debt.py` | TODO/FIXME/HACK/NOCOMMIT (all languages) | `global/tech-debt.md` |
| `common/secrets.py` | hardcoded passwords, API keys, PEM keys (all languages) | `global/secrets.md` |
| `common/nesting.py` | brace depth (all languages) | `global/nesting.md` |
| `common/file_size.py` | line count limits | `global/file-limits.md` |
| `rust/checks/errors.py` | unwrap/expect/panic/Box<dyn Error> | `rust/errors.md` |
| `rust/checks/naming.py` | banned names, bool prefix, unsafe comment | `rust/naming.md` |
| `rust/checks/modules.py` | utils.rs, inline mod | `rust/modules.md` |
| `rust/checks/types.py` | &Vec, &String, println!, static mut | `rust/types.md` |
| `rust/checks/threading.py` | fire-and-forget, Arc/Rc comment | `rust/threading.md` |
| `rust/checks/coupling.py` | use super::sibling, pub(super) | `rust/modules.md` |
| `rust/checks/clone.py` | .clone() spam (>3 per function) | `rust/ownership.md` |
| `rust/checks/mother_child.py` | mother-too-many-fns (>3 warn, >6 error in mod.rs/main.rs), child-owns-state (static/lazy_static/thread_local/OnceLock in leaf files) | `uiux/mother-child.md` |
| `slint/checks/tokens.py` | hardcoded colors/sizes | `uiux/tokens.md` |
| `slint/checks/structure.py` | multiple components per file | `global/module-tree.md` |
| `slint/checks/events.py` | callback logic, state mutations | `uiux/state-flow.md` |
| `slint/checks/strings.py` | hardcoded string literals in components | `slint/validation.md` |
| `slint/checks/mother_child.py` | child-has-state (in-out property without <=> in non-Window component), sibling-import (view importing sibling view) | `uiux/mother-child.md` |
| `slint/checks/architecture.py` | multiple gateway objects across tree | `uiux/state-flow.md` |
| `js/checks/modules.py` | require/CJS, mutable exports | `js/modules.md` |
| `js/checks/safety.py` | eval, layer violation, promise, console.log | `js/safety.md` |
| `js/checks/validation.py` | JSON.parse/fetch without schema | `js/validation.md` |
| `js/checks/typescript.py` | any, @ts-ignore, non-null assertion | `js/safety.md` |
| `css/checks/tokens.py` | hardcoded colors, !important, font px | `css/custom-properties.md` |
| `css/checks/layout.py` | magic z-index, hardcoded transition durations | `css/validation.md` |
| `python/checks/types_check.py` | future annotations, Optional, bare except | `python/types.md` |
| `python/checks/nesting_check.py` | indent depth | `global/nesting.md` |
| `python/checks/validation_check.py` | json.loads/response.json without pydantic | `python/validation.md` |
| `python/checks/antipatterns.py` | mutable defaults, global keyword, eval | `python/types.md` |
| `kotlin/checks/safety.py` | !!, java.* imports, multi-class | `kotlin/encapsulation.md` |
| `kotlin/checks/coroutines.py` | Thread.sleep, runBlocking, untracked launch | `kotlin/coroutines.md` |
| `csharp/checks/types.py` | #nullable disable, dynamic, object param | `csharp/types.md` |
| `csharp/checks/errors.py` | catch (Exception), naked catch, throw new Exception, empty wildcard arm, log-without-rethrow | `csharp/errors.md` |
| `csharp/checks/naming.py` | banned names, bool prefix, interface I-prefix, _camelCase fields | `csharp/naming.md` |
| `csharp/checks/threading.py` | .Result, .Wait(), GetAwaiter().GetResult(), Thread.Sleep, async void, fire-and-forget | `csharp/threading.md` |
| `csharp/checks/linq.py` | First() without OrDefault, Count()>0, Select+Where order, side effects, ToList in loop | `csharp/linq.md` |
| `csharp/checks/security.py` | SQL injection, Process.Start injection, Environment.Exit, hardcoded passwords, BinaryFormatter, XmlDocument XXE, path traversal, Regex ReDoS | `csharp/errors.md` |
| `csharp/checks/project_file.py` | `<Nullable>enable</Nullable>` missing, `<TreatWarningsAsErrors>true` missing | `csharp/modules.md` |

## CLI

```bash
rulestools detect [PATH]          # auto-detect languages → proj/rulestools.toml
rulestools scan [PATH]            # one-shot scan, writes proj/ISSUES
rulestools scan --watch [PATH]    # daemon, re-scans on file change
rulestools init [PATH]            # installs VSCode task + pre-commit hook
```
