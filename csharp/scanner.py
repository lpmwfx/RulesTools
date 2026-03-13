"""C# scanner — orchestrates all C# / .NET checks."""

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
from csharp.checks.types import check as check_types
from csharp.checks.errors import check as check_errors
from csharp.checks.naming import check as check_naming
from csharp.checks.threading import check as check_threading
from csharp.checks.linq import check as check_linq
from csharp.checks.security import check as check_security
from csharp.checks.project_file import check as check_project_file

# C# nesting: namespace(1) + class(2) + method(3) + 3 logic levels = 6 → flag at 7
_NESTING_MAX_ABS = 7

EXTENSIONS = {".cs", ".csx"}
PROJ_EXTENSIONS = {".csproj", ".props"}

_SKIP_DIRS = {".git", "bin", "obj", ".vs", "packages", "TestResults", "target"}


def scan_file(path: Path) -> list[Issue]:
    try:
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return []

    if path.suffix in PROJ_EXTENSIONS:
        return list(check_project_file(path, lines))

    issues: list[Issue] = []
    issues.extend(check_file_size(path, lines))
    issues.extend(check_nesting(path, lines, lang="csharp", max_abs_depth=_NESTING_MAX_ABS))
    issues.extend(check_debt(path, lines))
    issues.extend(check_secrets(path, lines))
    issues.extend(check_types(path, lines))
    issues.extend(check_errors(path, lines))
    issues.extend(check_naming(path, lines))
    issues.extend(check_threading(path, lines))
    issues.extend(check_linq(path, lines))
    issues.extend(check_security(path, lines))
    issues.extend(check_topology(path, lines))
    issues.extend(check_imports(path, lines))
    return issues


def scan_tree(root: Path) -> Generator[Issue, None, None]:
    for ext in EXTENSIONS:
        for path in root.rglob(f"*{ext}"):
            if any(skip in path.parts for skip in _SKIP_DIRS):
                continue
            yield from scan_file(path)

    for ext in PROJ_EXTENSIONS:
        for path in root.rglob(f"*{ext}"):
            if any(skip in path.parts for skip in _SKIP_DIRS):
                continue
            yield from scan_file(path)
