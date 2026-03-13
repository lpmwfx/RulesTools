"""Rust module-structure checks — from rust/modules.md.

Checks:
  - Filename is utils.rs / helpers.rs / common.rs (without domain prefix)
  - Nested `mod` blocks (mod inside mod — should be separate files)
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "rust/modules"

_BANNED_FILENAMES = {"utils", "helpers", "common"}

# mod <name> { — inline module declaration (not `mod name;` which is a file ref)
_INLINE_MOD = re.compile(r"^\s*(?:pub(?:\([^)]+\))?\s+)?mod\s+(\w+)\s*\{")


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    stem = path.stem.lower()

    # --- Banned filename ---
    if stem in _BANNED_FILENAMES:
        yield Issue(
            file=path, line=1, col=1,
            severity=Severity.ERROR,
            rule=f"{_RULE_BASE}/no-generic-filenames",
            message=(
                f"'{path.name}' is a banned generic module name — "
                f"add a domain prefix (e.g. 'config_helpers.rs')"
            ),
        )

    # --- Nested inline mod blocks ---
    mod_depth = 0
    brace_depth = 0

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        if _INLINE_MOD.match(raw):
            if brace_depth > 0:
                # We're inside something — this is a nested mod
                yield Issue(
                    file=path, line=lineno, col=1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/one-module-per-file",
                    message=(
                        "nested inline mod block — extract to a separate file "
                        "(use 'mod name;' to reference it)"
                    ),
                )
            mod_depth = brace_depth  # remember where we opened

        # Track brace depth (simplified — ignores strings/comments)
        clean = re.sub(r"//.*", "", raw)
        clean = re.sub(r'"[^"]*"', '""', clean)
        brace_depth += clean.count("{") - clean.count("}")
        brace_depth = max(brace_depth, 0)
