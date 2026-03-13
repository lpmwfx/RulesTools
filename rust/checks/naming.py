"""Rust naming checks — from rust/naming.md.

Checks:
  - Banned bare variable names (data, info, value, etc.)
  - Boolean variables must start with is_/has_/can_/should_
  - Unsafe block must have an explaining comment on the same or previous line
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "rust/naming"

# Banned names when used as `let <name>` or `let mut <name>` without domain suffix
_BANNED_BARE = {
    "data", "info", "value", "item", "object",
    "temp", "state", "ctx", "result", "res", "var",
}

# let [mut] <name>[: type] = ...
_LET = re.compile(r"\blet\s+(?:mut\s+)?(\w+)")

# Variable declarations typed as bool
_BOOL_TYPED = re.compile(r"\blet\s+(?:mut\s+)?(\w+)\s*:\s*bool\b")

# bool-returning patterns (heuristic: = true/false literal)
_BOOL_ASSIGN = re.compile(r"\blet\s+(?:mut\s+)?(\w+)\s*=\s*(?:true|false)\b")

_BOOL_PREFIXES = ("is_", "has_", "can_", "should_")

_UNSAFE = re.compile(r"\bunsafe\s*\{")


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        # --- Banned bare names ---
        for m in _LET.finditer(raw):
            name = m.group(1)
            if name in _BANNED_BARE:
                yield Issue(
                    file=path, line=lineno, col=m.start(1) + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-noise-names",
                    message=(
                        f"'{name}' is a banned bare name — add a domain suffix "
                        f"(e.g. '{name}_parsed', '{name}_input')"
                    ),
                )

        # --- Boolean naming ---
        for pattern in (_BOOL_TYPED, _BOOL_ASSIGN):
            for m in pattern.finditer(raw):
                name = m.group(1)
                if not any(name.startswith(p) for p in _BOOL_PREFIXES):
                    yield Issue(
                        file=path, line=lineno, col=m.start(1) + 1,
                        severity=Severity.ERROR,
                        rule=f"{_RULE_BASE}/bool-prefix",
                        message=(
                            f"boolean '{name}' must start with "
                            f"is_/has_/can_/should_"
                        ),
                    )

        # --- unsafe without comment ---
        for m in _UNSAFE.finditer(raw):
            # Check same line or the line immediately before
            same_line_comment = "//" in raw[: m.start()]
            prev_line_comment = (
                lineno >= 2 and "//" in lines[lineno - 2]
            )
            if not same_line_comment and not prev_line_comment:
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/unsafe-comment",
                    message=(
                        "unsafe block without explaining comment — "
                        "document the invariant and why unsafe is needed"
                    ),
                )
