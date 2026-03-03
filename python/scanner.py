"""Python scanner — orchestrates all Python checks."""

from __future__ import annotations
from pathlib import Path
from typing import Generator

from common.issue import Issue
from common.file_size import check as check_file_size
from common.debt import check as check_debt
from common.secrets import check as check_secrets
from python.checks.types_check import check as check_types
from python.checks.nesting_check import check as check_nesting
from python.checks.validation_check import check as check_validation
from python.checks.antipatterns import check as check_antipatterns

EXTENSIONS = {".py"}

_SKIP_DIRS = {".git", "__pycache__", ".venv", "venv", ".tox", "dist", "build", "target"}


def scan_file(path: Path) -> list[Issue]:
    try:
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return []

    issues: list[Issue] = []
    issues.extend(check_file_size(path, lines))
    issues.extend(check_debt(path, lines))
    issues.extend(check_secrets(path, lines))
    issues.extend(check_types(path, lines))
    issues.extend(check_nesting(path, lines))
    issues.extend(check_validation(path, lines))
    issues.extend(check_antipatterns(path, lines))
    return issues


def scan_tree(root: Path) -> Generator[Issue, None, None]:
    for path in root.rglob("*.py"):
        if any(skip in path.parts for skip in _SKIP_DIRS):
            continue
        yield from scan_file(path)
