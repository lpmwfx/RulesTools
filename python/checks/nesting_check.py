"""Python nesting check — from global/nesting.md + python/nesting.md.

Python uses indentation directly as nesting depth.
Rule: max 3 logic levels inside a function/method.

With 4-space indentation:
  level 0 = module scope
  level 1 = class or function body       (4 spaces)
  level 2 = method body inside class     (8 spaces)
  level 3 = logic level 1               (12 spaces)
  level 4 = logic level 2               (16 spaces)
  level 5 = logic level 3  ← max OK    (20 spaces)
  level 6 = BANNED                      (24 spaces)

We flag at >= 24 spaces indent (6+ levels) as error,
and >= 20 spaces (5+ levels) as warning for non-class code.
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE = "global/nesting"

# We only flag lines that are control-flow openers (not just deeply indented data)
_CONTROL = re.compile(r"^\s*(if|elif|else|for|while|with|try|except|finally)\b")


def _indent_level(line: str, indent_size: int = 4) -> int:
    spaces = len(line) - len(line.lstrip())
    return spaces // indent_size


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    for lineno, raw in enumerate(lines, start=1):
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue

        # Only flag control-flow lines to avoid false positives on deep data structures
        if not _CONTROL.match(raw):
            continue

        level = _indent_level(raw)

        if level >= 6:
            yield Issue(
                file=path, line=lineno, col=len(raw) - len(raw.lstrip()) + 1,
                severity=Severity.ERROR,
                rule=_RULE,
                message=(
                    f"nesting depth {level} — banned (max 3 logic levels). "
                    f"Extract a helper function."
                ),
            )
        elif level >= 5:
            yield Issue(
                file=path, line=lineno, col=len(raw) - len(raw.lstrip()) + 1,
                severity=Severity.ERROR,
                rule=_RULE,
                message=(
                    f"nesting depth {level} — approaching limit of 3 logic levels. "
                    f"Consider extracting a helper."
                ),
            )
