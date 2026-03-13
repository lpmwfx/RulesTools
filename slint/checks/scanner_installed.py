"""Slint scanner-installation check — verifies slintscanners is wired in.

A project with .slint files MUST have `slintscanners::scan_project()` called
from at least one `build.rs`.  If an AI removes the call (or the dependency),
this check emits an ERROR that blocks the pre-commit hook.

TREE-LEVEL check: call check_tree(root) once per scan, not per file.
Rule: slint/build/scanner-required
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Generator

from common.issue import Issue, Severity

_RULE = "slint/build/scanner-required"

_CALL_RE = re.compile(r"\bslintscanners\s*::\s*scan_project\s*\(")

_SKIP_DIRS = {"target", ".git", ".cargo"}


def _skip(path: Path) -> bool:
    return any(p in _SKIP_DIRS for p in path.parts)


def check_tree(root: Path) -> Generator[Issue, None, None]:
    """Emit ERROR if no build.rs in the project calls slintscanners::scan_project()."""

    # Collect all build.rs files
    build_scripts = [
        p for p in root.rglob("build.rs")
        if not _skip(p)
    ]

    # Check each build.rs for the scan_project() call
    callers: list[Path] = []
    for bs in build_scripts:
        try:
            text = bs.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        if _CALL_RE.search(text):
            callers.append(bs)

    if callers:
        return  # compliant

    # ── No caller found ───────────────────────────────────────────────────────
    root_cargo = root / "Cargo.toml"
    if root_cargo.exists():
        target = root_cargo
    elif build_scripts:
        target = build_scripts[0]
    else:
        target = root / "build.rs"

    if build_scripts:
        detail = (
            "build.rs exists but does not call slintscanners::scan_project() — "
            "add it or the pre-commit hook cannot verify Slint UI quality"
        )
    else:
        detail = (
            "no build.rs found — slintscanners must be added: "
            "add [build-dependencies] slintscanners = { git = \"...\" } "
            "and call slintscanners::scan_project() from build.rs"
        )

    yield Issue(
        file=target, line=1, col=1,
        severity=Severity.ERROR,
        rule=_RULE,
        message=detail,
    )
