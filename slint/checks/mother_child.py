"""Slint mother-child architecture checks.

Rules (from uiux/mother-child.md):

  VITAL: Only the mother component (inherits Window) may own state
  VITAL: Children are stateless — only `in property` + `callback`
  RULE:  Siblings never import each other — all communication through mother
  BANNED: `in-out property` in child components (except <=> delegation)
  BANNED: View importing a sibling view (views/ folder)
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "uiux/mother-child"

# ── Detection patterns ──────────────────────────────────────────────────────

# Mother = the file containing `inherits Window`
_INHERITS_WINDOW = re.compile(r"\binherits\s+Window\b")

# Global definition file (not a component — exempt from child rules)
_EXPORT_GLOBAL = re.compile(r"^\s*export\s+global\s+")

# in-out property declaration (state ownership)
_IN_OUT_PROP = re.compile(r"^\s*in-out\s+property\b")

# Delegation binding — the only valid reason for in-out in a child
_DELEGATION = re.compile(r"<=>")

# Import statement: import { Foo } from "path.slint"
_IMPORT = re.compile(r'import\s+\{[^}]*\}\s+from\s+"([^"]+)"')

# Standard library import (always OK)
_STD_IMPORT = re.compile(r"^std-widgets\.slint$")


def _is_mother(lines: list[str]) -> bool:
    """True if this file defines the root Window component."""
    for line in lines:
        if _INHERITS_WINDOW.search(line):
            return True
    return False


def _is_global_file(lines: list[str]) -> bool:
    """True if this file defines a global (tokens, types, bridge)."""
    for line in lines[:40]:
        if _EXPORT_GLOBAL.match(line):
            return True
    return False


def _is_views_folder(path: Path) -> bool:
    """True if this file lives in a views/ directory."""
    return "views" in path.parts


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    # Skip the mother file — she IS allowed to own state
    if _is_mother(lines):
        return

    # Skip global definition files (Theme, Types, AppBridge)
    if _is_global_file(lines):
        return

    # ── 1. Child has state (in-out property without <=>) ────────────────
    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()

        if stripped.startswith("//"):
            continue

        if _IN_OUT_PROP.match(stripped) and not _DELEGATION.search(raw):
            yield Issue(
                file=path, line=lineno, col=1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/child-has-state",
                message=(
                    "in-out property in child component — children must be "
                    "stateless. Use 'in property' to receive state from mother, "
                    "'callback' to emit events up. "
                    "Only the mother (inherits Window) may own state."
                ),
            )

    # ── 2. Sibling import in views/ ─────────────────────────────────────
    #
    # Views are all direct children of mother — they must not know about
    # each other.  Valid imports from views/ are:
    #   "../globals/..."    (cross-folder — OK)
    #   "../panels/..."     (cross-folder — OK)
    #   "std-widgets.slint" (standard library — OK)
    #
    # An import without "../" prefix (same-folder) in views/ = sibling.
    if _is_views_folder(path):
        for lineno, raw in enumerate(lines, start=1):
            m = _IMPORT.search(raw)
            if not m:
                continue
            import_path = m.group(1)

            # Standard library — always OK
            if _STD_IMPORT.match(import_path):
                continue

            # Cross-folder import (../) — OK
            if import_path.startswith("../") or import_path.startswith("./"):
                continue

            # Same-folder import in views/ = sibling coupling
            yield Issue(
                file=path, line=lineno, col=m.start(1) + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/sibling-import",
                message=(
                    f"view imports sibling view '{import_path}' — views are "
                    f"children of mother and must not know about each other. "
                    f"Route shared state through mother instead."
                ),
            )
