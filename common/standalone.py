"""Standalone directory detection — tools/, scripts/, src/bin/.

Standalone directories contain loose CLI utilities and scripts that are
not part of the layered architecture. They are exempt from topology and
import-direction checks, but project code must not import from them.
"""

from __future__ import annotations
import re
from pathlib import Path
from typing import Generator

from common.issue import Issue, Severity

# Directories that contain standalone tools — not part of the architecture
STANDALONE_DIRS = {"tools", "scripts", "bin"}

_COMMENT_STARTS = ("//", "#", "/*", "*", "--")

_IMPORT_RE = re.compile(
    r"""(?:use\s+(?:crate::)?|from\s+\.{0,3}|import\s+|from\s+['"]\.*/?)"""
    r"""(tools|scripts)\b"""
)


def is_standalone(path: Path) -> bool:
    """Return True if file lives in a standalone tools/scripts/bin directory."""
    parts = [p.lower() for p in path.parts]
    for dirname in STANDALONE_DIRS:
        if dirname in parts:
            return True
    for i, part in enumerate(parts):
        if part == "src" and i + 1 < len(parts) and parts[i + 1] == "bin":
            return True
    return False


def check_no_import(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    """Error if project code imports from tools/ or scripts/."""
    for lineno, raw in enumerate(lines, start=1):
        if raw.lstrip().startswith(_COMMENT_STARTS):
            continue
        m = _IMPORT_RE.search(raw)
        if m:
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule="global/topology/standalone-boundary",
                message=(
                    f"project code imports from {m.group(1)}/ "
                    f"— tools and scripts are standalone, not libraries"
                ),
            )
