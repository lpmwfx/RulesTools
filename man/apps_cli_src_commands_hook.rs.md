# apps/cli/src/commands/hook.rs

## `pub fn cmd_hook()`

*Line 7 · fn*

PostToolUse hook — scan file after Edit/Write (reads JSON from stdin).

Called by Claude Code `.claude/settings.json` PostToolUse hook.
Reads tool invocation JSON from stdin, extracts file_path,
scans the file, and prints violations to stderr (advisory).
Always exits 0 — never blocks edits.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=6" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
