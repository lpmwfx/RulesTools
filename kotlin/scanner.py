"""Kotlin scanner — orchestrates all Kotlin checks."""

from __future__ import annotations
from pathlib import Path
from typing import Generator

from common.issue import Issue
from common.file_size import check as check_file_size
from common.nesting import check as check_nesting
from kotlin.checks.safety import check as check_safety

# Kotlin: class(1) + fun(2) + 3 logic = 5 → flag at 6
_NESTING_MAX_ABS = 6

EXTENSIONS = {".kt", ".kts"}

_SKIP_DIRS = {".git", "build", ".gradle", ".idea", "target"}


def scan_file(path: Path) -> list[Issue]:
    try:
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return []

    issues: list[Issue] = []
    issues.extend(check_file_size(path, lines))
    issues.extend(check_nesting(path, lines, lang="kotlin", max_abs_depth=_NESTING_MAX_ABS))
    issues.extend(check_safety(path, lines))
    return issues


def scan_tree(root: Path) -> Generator[Issue, None, None]:
    for ext in EXTENSIONS:
        for path in root.rglob(f"*{ext}"):
            if any(skip in path.parts for skip in _SKIP_DIRS):
                continue
            yield from scan_file(path)
