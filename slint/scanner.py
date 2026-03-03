"""Slint scanner — orchestrates all Slint checks for a single file.

Slint is the STRICTEST file type:
  hard limit = 200 lines (AI loses the property graph above this)
  soft limit = 160 lines
  nesting    = max 3 levels inside a component
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator

from common.issue import Issue
from common.file_size import check as check_file_size
from common.nesting import check as check_nesting
from slint.checks.tokens import check as check_tokens
from slint.checks.structure import check as check_structure
from slint.checks.events import check as check_events

# Slint: component (1) + layout (2) + nested layout (3) + if/for (4) + widget (5) = normal
# Flag at 6: warning at exactly 6, error at 7+
# Single-line { } blocks (callbacks, closures) are excluded by nesting.py
_NESTING_MAX_ABS = 6

EXTENSIONS = {".slint"}

_SKIP_DIRS = {".git", "target"}


def scan_file(path: Path) -> list[Issue]:
    try:
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return []

    issues: list[Issue] = []
    issues.extend(check_file_size(path, lines))
    issues.extend(check_nesting(path, lines, lang="slint", max_abs_depth=_NESTING_MAX_ABS))
    issues.extend(check_tokens(path, lines))
    issues.extend(check_structure(path, lines))
    issues.extend(check_events(path, lines))
    return issues


def scan_tree(root: Path) -> Generator[Issue, None, None]:
    for path in root.rglob("*.slint"):
        if any(skip in path.parts for skip in _SKIP_DIRS):
            continue
        yield from scan_file(path)
