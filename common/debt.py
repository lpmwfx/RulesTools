"""Tech-debt marker check — from global/tech-debt.md.

Flags TODO/FIXME/HACK/NOCOMMIT/XXX in committed source code.
NOCOMMIT and HACK are errors (block commit); the rest are warnings.
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from .issue import Issue, Severity

_RULE = "global/tech-debt"

# Patterns and their severity
_MARKERS: list[tuple[re.Pattern, Severity]] = [
    (re.compile(r"\bNOCOMMIT\b",  re.IGNORECASE), Severity.ERROR),
    (re.compile(r"\bHACK\b",      re.IGNORECASE), Severity.ERROR),
    (re.compile(r"\bWORKAROUND\b",re.IGNORECASE), Severity.ERROR),
    (re.compile(r"\bTODO\b",      re.IGNORECASE), Severity.WARNING),
    (re.compile(r"\bFIXME\b",     re.IGNORECASE), Severity.WARNING),
    (re.compile(r"\bXXX\b"),                       Severity.WARNING),
]

# File types where these are documentation, not tech debt
_SKIP_SUFFIXES = {".md", ".txt", ".rst", ".toml", ".yaml", ".yml", ".json"}


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    if path.suffix.lower() in _SKIP_SUFFIXES:
        return

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        # Only flag inside comments/strings (don't flag code tokens named "TODO_handler")
        # Heuristic: marker appears after a comment character or is surrounded by spaces
        for pattern, severity in _MARKERS:
            for m in pattern.finditer(raw):
                col = m.start()
                before = raw[:col]
                # Must be in a comment or at a word boundary with surrounding whitespace
                in_comment = (
                    "//" in before or
                    "#" in before or
                    "/*" in before or
                    "--" in before or  # SQL / Slint comments
                    stripped.startswith(("//", "#", "*", "/*", "--"))
                )
                if not in_comment:
                    continue
                marker = m.group(0).upper()
                yield Issue(
                    file=path, line=lineno, col=col + 1,
                    severity=severity,
                    rule=_RULE,
                    message=(
                        f"{marker} in committed code — "
                        f"fix it or log it in proj/TODO before committing"
                    ),
                )
                break  # one issue per line max
