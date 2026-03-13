"""Slint string state checks — from slint/mother-child.md + uiux/state-flow.md.

Slint has no enums, so string comparisons for state routing are common.
The risk: the same magic strings scattered across many files with no
central definition — a rename breaks everything silently.

BANNED:
  - The same string literal (>=3 chars) used in == comparisons in 3+ files
    without a corresponding property definition (stringly-typed state)

WARNING:
  - String literal used in == comparison that also appears in root.x = "..."
    assignment (round-trip string state — should at minimum be a constant)

Note: This check is file-set aware — it scans ALL files then reports.
Single-file checks call check() per file; call check_project() for cross-file.
"""

from __future__ import annotations

from collections import defaultdict
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "slint/validation"

# == "some-value" or != "some-value"  (3+ char strings)
_STR_COMPARE = re.compile(r'[!=]=\s*"([^"]{3,})"')

# root.x = "some-value" or x: "some-value" state assignment
_STR_ASSIGN = re.compile(r'(?:root\.[\w-]+\s*=|[\w-]+\s*:)\s*"([^"]{3,})"')

# Skip token/globals/theme definition files
_TOKEN_FOLDERS = {"globals", "tokens", "theme"}

# Strings that are obviously not state (UI text, paths, etc.)
_SKIP_VALUES = re.compile(
    r"^(?:[A-Z]|.*\s.*|.*[/\\].*|.*\.slint|.*\.png|.*\.svg|v\d|\d)$"
)


def _is_token_file(path: Path) -> bool:
    return any(part in _TOKEN_FOLDERS for part in path.parts)


def _comment_start(raw: str) -> int:
    m = re.search(r"//", raw)
    return m.start() if m else len(raw)


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    """Per-file check: warn on string comparisons that look like state routing."""
    if _is_token_file(path):
        return

    for lineno, raw in enumerate(lines, start=1):
        if raw.lstrip().startswith("//"):
            continue
        comment_at = _comment_start(raw)
        segment = raw[:comment_at]

        for m in _STR_COMPARE.finditer(segment):
            val = m.group(1)
            if _SKIP_VALUES.match(val):
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/string-state-comparison",
                message=(
                    f'stringly-typed state == "{val}" — config-driven paradigm (uiux/tokens): '
                    f"all state values must be named constants defined in globals/ or moved to a Rust enum"
                ),
            )
