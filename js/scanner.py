"""JS scanner — orchestrates all JS checks for a single file."""

from __future__ import annotations
from pathlib import Path
from typing import Generator

from common.issue import Issue
from common.file_size import check as check_file_size
from common.nesting import check as check_nesting
from common.debt import check as check_debt
from common.secrets import check as check_secrets
from common.topology import check as check_topology
from common.import_direction import check as check_imports
from js.checks.modules import check as check_modules
from js.checks.safety import check as check_safety
from js.checks.validation import check as check_validation
from js.checks.typescript import check as check_typescript

# JS: fn body = depth 1, flag at >= 4 (fn + 3 logic levels)
_NESTING_MAX_ABS = 4

EXTENSIONS = {".js", ".mjs", ".ts"}

_SKIP_DIRS = {"node_modules", ".git", "dist", "build", ".cache"}


def scan_file(path: Path) -> list[Issue]:
    try:
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return []

    issues: list[Issue] = []
    issues.extend(check_file_size(path, lines))
    issues.extend(check_nesting(path, lines, lang="js", max_abs_depth=_NESTING_MAX_ABS))
    issues.extend(check_debt(path, lines))
    issues.extend(check_secrets(path, lines))
    issues.extend(check_modules(path, lines))
    issues.extend(check_safety(path, lines))
    issues.extend(check_validation(path, lines))
    issues.extend(check_typescript(path, lines))
    issues.extend(check_topology(path, lines))
    issues.extend(check_imports(path, lines))
    return issues


def scan_tree(root: Path) -> Generator[Issue, None, None]:
    for ext in EXTENSIONS:
        for path in root.rglob(f"*{ext}"):
            if any(skip in path.parts for skip in _SKIP_DIRS):
                continue
            yield from scan_file(path)
