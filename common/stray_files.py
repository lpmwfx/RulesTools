"""Stray file check — source files must live in recognized project areas.

TREE-LEVEL check: call check_tree(root) once per scan, not per file.

.rs and .slint files must be inside recognized directories relative to
a Cargo.toml crate root:
  src/         — main source
  tests/       — integration tests
  benches/     — benchmarks
  examples/    — example programs
  build.rs     — build script (crate root only)
  tools/       — standalone (exempt from all checks)
  scripts/     — standalone (exempt from all checks)

Files found elsewhere are flagged as strays — likely AI-generated in
the wrong location.

Additionally, tools/ and scripts/ must not contain project-structure
subdirectories (src/, ui/, crate*/) — they hold flat standalone files.
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator

from common.issue import Issue, Severity
from common.standalone import STANDALONE_DIRS

_RULE_STRAY = "global/topology/stray-file"
_RULE_NESTED = "global/topology/standalone-nesting"

# Directories that are always skipped entirely
_SKIP_DIRS = {"target", ".git", ".cargo", "node_modules", "__pycache__"}

# Recognized source directories relative to a crate root
_SOURCE_DIRS = {"src", "tests", "benches", "examples"}

# Project-structure dirs that must not appear inside tools/ or scripts/
_PROJECT_STRUCTURE_DIRS = {"src", "ui", "adapter", "core", "pal", "gateway", "shared"}


def _find_crate_roots(root: Path) -> set[Path]:
    """Find all directories containing a Cargo.toml."""
    roots: set[Path] = set()
    for cargo in root.rglob("Cargo.toml"):
        if any(p in _SKIP_DIRS for p in cargo.parts):
            continue
        roots.add(cargo.parent)
    return roots


def _is_recognized(path: Path, crate_roots: set[Path]) -> bool:
    """Return True if a source file is in a recognized location."""
    parts_lower = [p.lower() for p in path.parts]

    # Skip dirs
    if any(p in _SKIP_DIRS for p in parts_lower):
        return True

    # Standalone dirs — exempt
    if any(p in STANDALONE_DIRS for p in parts_lower):
        return True

    # build.rs at a crate root
    if path.name == "build.rs":
        return path.parent in crate_roots

    # Inside a recognized source directory relative to any crate root
    for crate_root in crate_roots:
        try:
            rel = path.relative_to(crate_root)
        except ValueError:
            continue
        first_part = rel.parts[0].lower() if rel.parts else ""
        if first_part in _SOURCE_DIRS:
            return True

    return False


def _check_standalone_nesting(root: Path) -> Generator[Issue, None, None]:
    """Error if tools/ or scripts/ contain project-structure subdirectories."""
    for standalone_name in STANDALONE_DIRS:
        for standalone_dir in root.rglob(standalone_name):
            if not standalone_dir.is_dir():
                continue
            if any(p in _SKIP_DIRS for p in standalone_dir.parts):
                continue
            # Check for project-structure subdirs
            for child in standalone_dir.iterdir():
                if child.is_dir() and child.name.lower() in _PROJECT_STRUCTURE_DIRS:
                    yield Issue(
                        file=child / "(directory)",
                        line=1, col=1,
                        severity=Severity.ERROR,
                        rule=_RULE_NESTED,
                        message=(
                            f"{standalone_name}/{child.name}/ looks like project structure "
                            f"— standalone dirs must be flat, not contain {child.name}/"
                        ),
                    )


def check_tree(
    root: Path,
    extensions: set[str] | None = None,
) -> Generator[Issue, None, None]:
    """Flag source files outside recognized project areas.

    Args:
        root: Project root directory.
        extensions: File extensions to check (e.g. {".rs", ".slint"}).
                    Defaults to {".rs", ".slint"}.
    """
    if extensions is None:
        extensions = {".rs", ".slint"}

    crate_roots = _find_crate_roots(root)
    if not crate_roots:
        return  # Not a Cargo project

    for ext in extensions:
        pattern = f"*{ext}"
        for path in root.rglob(pattern):
            if not _is_recognized(path, crate_roots):
                yield Issue(
                    file=path, line=1, col=1,
                    severity=Severity.ERROR,
                    rule=_RULE_STRAY,
                    message=(
                        f"source file outside recognized project area "
                        f"— move to src/, tests/, benches/, or examples/"
                    ),
                )

    # Check standalone dirs don't contain project structure
    yield from _check_standalone_nesting(root)
