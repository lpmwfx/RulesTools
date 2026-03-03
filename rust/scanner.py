"""Rust scanner — orchestrates all Rust checks for a single file."""

from __future__ import annotations
from pathlib import Path
from typing import Generator

from common.issue import Issue
from common.file_size import check as check_file_size
from common.nesting import check as check_nesting
from common.debt import check as check_debt
from common.secrets import check as check_secrets
from rust.checks.errors import check as check_errors
from rust.checks.naming import check as check_naming
from rust.checks.modules import check as check_modules
from rust.checks.types import check as check_types
from rust.checks.threading import check as check_threading
from rust.checks.coupling import check as check_coupling
from rust.checks.clone import check as check_clone
from rust.checks.gateway import check as check_gateway
from rust.checks.adapter import check as check_adapter

# Rust: impl body = depth 2 (mod + impl), fn body = depth 1–2
# Flag at absolute brace depth >= 5 (impl + fn + 3 logic levels)
_NESTING_MAX_ABS = 5

EXTENSIONS = {".rs"}


def scan_file(path: Path) -> list[Issue]:
    try:
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return []

    issues: list[Issue] = []
    issues.extend(check_file_size(path, lines))
    issues.extend(check_nesting(path, lines, lang="rs", max_abs_depth=_NESTING_MAX_ABS))
    issues.extend(check_debt(path, lines))
    issues.extend(check_secrets(path, lines))
    issues.extend(check_errors(path, lines))
    issues.extend(check_naming(path, lines))
    issues.extend(check_modules(path, lines))
    issues.extend(check_types(path, lines))
    issues.extend(check_threading(path, lines))
    issues.extend(check_coupling(path, lines))
    issues.extend(check_clone(path, lines))
    issues.extend(check_gateway(path, lines))
    issues.extend(check_adapter(path, lines))
    return issues


def scan_tree(root: Path) -> Generator[Issue, None, None]:
    for path in root.rglob("*.rs"):
        # Skip target/ and generated files
        parts = path.parts
        if "target" in parts or ".cargo" in parts:
            continue
        yield from scan_file(path)
