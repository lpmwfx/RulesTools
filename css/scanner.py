"""CSS scanner — file size + token + cascade checks."""

from __future__ import annotations
from pathlib import Path
from typing import Generator

from common.issue import Issue
from common.file_size import check as check_file_size
from common.nesting import check as check_nesting
from common.debt import check as check_debt
from css.checks.tokens import check as check_tokens
from css.checks.layout import check as check_layout

# CSS uses { } for rule blocks — flag at depth 4+ (nested rules are unusual)
_NESTING_MAX_ABS = 4

EXTENSIONS = {".css", ".scss", ".sass"}

_SKIP_DIRS = {"node_modules", ".git", "dist", "build", "target"}


def scan_file(path: Path) -> list[Issue]:
    try:
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return []

    issues: list[Issue] = []
    issues.extend(check_file_size(path, lines))
    issues.extend(check_nesting(path, lines, lang="css", max_abs_depth=_NESTING_MAX_ABS))
    issues.extend(check_debt(path, lines))
    issues.extend(check_tokens(path, lines))
    issues.extend(check_layout(path, lines))
    return issues


def scan_tree(root: Path) -> Generator[Issue, None, None]:
    for ext in EXTENSIONS:
        for path in root.rglob(f"*{ext}"):
            if any(skip in path.parts for skip in _SKIP_DIRS):
                continue
            yield from scan_file(path)
