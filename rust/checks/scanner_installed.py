"""Rust scanner-installation check — verifies rustscanners is wired in.

A Rust project MUST have `rustscanners::scan_project()` called from at
least one `build.rs`.  If an AI removes the call (or the dependency),
this check emits an ERROR that blocks the pre-commit hook.

TREE-LEVEL check: call check_tree(root) once per scan, not per file.
Rule: rust/build/scanner-required
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Generator

from common.issue import Issue, Severity

_RULE = "rust/build/scanner-required"

_CALL_RE = re.compile(r"\brustscanners\s*::\s*scan_project\s*\(")
_DEP_RE  = re.compile(r"\brusstscanners\b")          # in Cargo.toml

_SKIP_DIRS = {"target", ".git", ".cargo"}


def _skip(path: Path) -> bool:
    return any(p in _SKIP_DIRS for p in path.parts)


def check_tree(root: Path) -> Generator[Issue, None, None]:
    """Emit ERROR if no build.rs in the project calls rustscanners::scan_project()."""

    # Collect all build.rs files (workspace members + root)
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
        return  # at least one build.rs calls it — project is compliant

    # ── No caller found — emit an error ──────────────────────────────────────
    # Attach the issue to the root Cargo.toml if it exists, otherwise to the
    # first build.rs that's missing the call, otherwise to the root itself.
    root_cargo = root / "Cargo.toml"
    if root_cargo.exists():
        target = root_cargo
    elif build_scripts:
        target = build_scripts[0]
    else:
        # No build.rs at all — still an error
        target = root / "build.rs"

    if build_scripts:
        detail = (
            f"build.rs exists but does not call rustscanners::scan_project() — "
            f"add it or the pre-commit hook cannot verify Rust code quality"
        )
    else:
        detail = (
            "no build.rs found — rustscanners must be added: "
            "add [build-dependencies] rustscanners = { git = \"...\" } "
            "and call rustscanners::scan_project() from build.rs"
        )

    yield Issue(
        file=target, line=1, col=1,
        severity=Severity.ERROR,
        rule=_RULE,
        message=detail,
    )
