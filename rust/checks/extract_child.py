"""Rust extract-child checks — cause-based signals for missing children.

E201 single-caller-extract:
    A function with >=20 lines that is called from exactly one place
    is a logical child — it belongs in its own module.

E202 fan-out-extract:
    A function that calls >=4 distinct other functions is an orchestrator
    and belongs in its own module.

These rules measure the *cause* of large files, not the consequence.
When a file grows to 300 lines, these rules tell you *which* functions
to extract — not just that you should split.
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity
from rust.checks.mother_child import _is_test_file, _is_test_context

_RULE_BASE = "uiux/mother-child"

# ── Thresholds ───────────────────────────────────────────────────────────────
_SINGLE_CALLER_MIN_LINES = 40  # E201: fn must be at least this many lines
_FAN_OUT_MIN_CALLS = 5         # E202: fn must call at least this many others
_FAN_OUT_MIN_LINES = 25        # E202: ignore small glue functions

# ── Patterns ─────────────────────────────────────────────────────────────────
_FN_DEF = re.compile(
    r"^\s*(?:pub(?:\([^)]+\))?\s+)?(?:async\s+)?fn\s+(\w+)\s*[<(]"
)
# Only match standalone function calls — NOT method calls (foo.bar())
# Negative lookbehind for `.` to exclude method invocations.
_CALL_IDENT = re.compile(r"(?<!\.)\b(\w+)\s*\(")

_KEYWORDS_RS = frozenset({
    "if", "while", "for", "match", "loop", "return", "fn", "pub",
    "let", "use", "mod", "impl", "trait", "struct", "enum", "type",
    "where", "async", "await", "move", "unsafe", "extern", "const",
    "static", "self", "super", "crate",
})

_COMMON_MACROS = frozenset({
    "vec", "assert", "assert_eq", "assert_ne", "panic", "todo",
    "unimplemented", "unreachable", "dbg", "println", "eprintln",
    "format", "print", "eprint", "write", "writeln",
    "log", "info", "warn", "error", "debug", "trace", "tracing",
})


def _clean_line(raw: str) -> str:
    """Strip string literals and line comments for brace counting."""
    s = re.sub(r'"(?:[^"\\]|\\.)*"', '""', raw)
    return re.sub(r"//.*", "", s)


def _parse_fn_extents(lines: list[str]) -> list[tuple[str, int, int]]:
    """Parse fn definitions → (name, start_1idx, end_1idx).

    start = line of `fn` keyword; end = line of closing `}`.
    Trait signatures with no body (ending in `;`) are skipped.
    """
    results: list[tuple[str, int, int]] = []
    n = len(lines)
    i = 0
    while i < n:
        raw = lines[i]
        if raw.lstrip().startswith("//"):
            i += 1
            continue
        m = _FN_DEF.match(raw)
        if not m:
            i += 1
            continue

        name = m.group(1)
        start = i + 1  # 1-indexed
        depth = 0
        found_open = False
        j = i

        while j < n:
            clean = _clean_line(lines[j])
            opens = clean.count("{")
            closes = clean.count("}")
            if opens > 0:
                found_open = True
            depth += opens - closes
            if found_open and depth <= 0:
                results.append((name, start, j + 1))  # j+1 = 1-indexed end
                i = j + 1
                break
            j += 1
        else:
            i += 1  # no body found (trait signature) — skip

    return results


def _count_call_sites_outside(
    name: str, lines: list[str], fn_start: int, fn_end: int
) -> int:
    """Count lines with `name(` outside [fn_start..fn_end] (1-indexed)."""
    pattern = re.compile(rf"\b{re.escape(name)}\s*\(")
    count = 0
    for lineno, raw in enumerate(lines, start=1):
        if fn_start <= lineno <= fn_end:
            continue
        if raw.lstrip().startswith("//"):
            continue
        if pattern.search(raw):
            count += 1
    return count


def _count_unique_callees(
    fn_name: str, lines: list[str], fn_start: int, fn_end: int
) -> int:
    """Count distinct non-keyword function names called in [fn_start..fn_end]."""
    callees: set[str] = set()
    for lineno in range(fn_start, fn_end + 1):
        raw = lines[lineno - 1]
        if raw.lstrip().startswith("//"):
            continue
        for hit in _CALL_IDENT.finditer(raw):
            callee = hit.group(1)
            if callee == fn_name:
                continue  # skip recursion
            if callee in _KEYWORDS_RS or callee in _COMMON_MACROS:
                continue
            if callee[0].isupper():
                continue  # constructors / type names
            callees.add(callee)
    return len(callees)


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    if _is_test_file(path):
        return

    for name, start, end in _parse_fn_extents(lines):
        if _is_test_context(lines, start):
            continue

        body_lines = end - start + 1

        # E201 — single caller + large body → extract to child module
        if body_lines >= _SINGLE_CALLER_MIN_LINES:
            callers = _count_call_sites_outside(name, lines, start, end)
            if callers == 1:
                yield Issue(
                    file=path, line=start, col=1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/extract-single-caller",
                    message=(
                        f"fn `{name}` ({body_lines} lines) has exactly 1 caller "
                        f"— it is a logical child module. Extract to its own file "
                        f"and import it."
                    ),
                )

        # E202 — fan-out orchestrator → extract to child module
        if body_lines >= _FAN_OUT_MIN_LINES:
            callees = _count_unique_callees(name, lines, start, end)
            if callees >= _FAN_OUT_MIN_CALLS:
                yield Issue(
                    file=path, line=start, col=1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/extract-orchestrator",
                    message=(
                        f"fn `{name}` calls {callees} distinct functions — "
                        f"it is an orchestrator. Extract to its own child module."
                    ),
                )
