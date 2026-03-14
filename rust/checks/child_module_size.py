"""Child module extraction adviser — rust/modules.md

Detects large inline `mod name { ... }` blocks and advises extraction.
Works alongside modules.py which checks for nested mods.

Thresholds:
  - warn_at (100 lines): "Plan extraction"
  - error_at (150 lines): "Extract immediately"
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE = "rust/modules/extract-child"

# mod <name> { — inline module (not mod name; which is file ref)
_INLINE_MOD = re.compile(r"^\s*(?:pub(?:\([^)]+\))?\s+)?mod\s+(\w+)\s*\{")


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    """Scan for large inline child modules and advise extraction."""

    for start_idx, raw in enumerate(lines):
        match = _INLINE_MOD.match(raw)
        if not match:
            continue

        mod_name = match.group(1)

        # Find matching closing brace
        end_idx = _find_closing_brace(lines, start_idx)
        if end_idx is None:
            continue

        lines_count = end_idx - start_idx

        # Emit advisory if large
        if lines_count >= 150:
            yield Issue(
                file=path,
                line=start_idx + 1,
                col=1,
                severity=Severity.ERROR,
                rule=_RULE,
                message=(
                    f"inline module '{mod_name}' has {lines_count} lines — "
                    f"EXTRACT IMMEDIATELY to {mod_name}.rs.\n"
                    f"Single file = single responsibility. Structure:\n"
                    f"  mod {mod_name};\n"
                    f"Then create src/{mod_name}.rs with module contents."
                ),
            )
        elif lines_count >= 100:
            yield Issue(
                file=path,
                line=start_idx + 1,
                col=1,
                severity=Severity.WARNING,
                rule=_RULE,
                message=(
                    f"inline module '{mod_name}' has {lines_count} lines — "
                    f"plan extraction to {mod_name}.rs.\n"
                    f"Large modules hurt readability and AI context. Each file should do one thing well."
                ),
            )


def _find_closing_brace(lines: list[str], start_idx: int) -> int | None:
    """Find matching closing brace. Returns line index or None."""
    start_line = lines[start_idx]
    open_count = start_line.count("{")
    close_count = start_line.count("}")

    depth = open_count - close_count

    for idx in range(start_idx + 1, len(lines)):
        line = lines[idx]
        depth += line.count("{") - line.count("}")

        if depth == 0:
            return idx

        # Safety: don't scan forever
        if idx - start_idx > 500:
            return None

    return None
