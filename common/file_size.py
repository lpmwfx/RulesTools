"""File size checks — from global/file-limits.md.

Limits by extension (lines):
  .slint          soft=160  hard=200   (strictest — AI loses context)
  .js / .ts       soft=200  hard=250
  .css / .scss    soft=120  hard=150
  .py             soft=200  hard=250
  .rs             soft=240  hard=300
  .cpp / .h       soft=280  hard=350

"Approaching" = within 20% of hard limit = soft limit.
Soft  → warning  ("plan the split")
Hard  → error    ("stop, split now")
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator

from .issue import Issue, Severity

# (soft_limit, hard_limit)
_LIMITS: dict[str, tuple[int, int]] = {
    ".slint": (160, 200),
    ".js":    (200, 250),
    ".ts":    (200, 250),
    ".mjs":   (200, 250),
    ".css":   (120, 150),
    ".scss":  (120, 150),
    ".py":    (200, 250),
    ".rs":    (240, 300),
    ".cpp":   (280, 350),
    ".cc":    (280, 350),
    ".cxx":   (280, 350),
    ".h":     (280, 350),
    ".hpp":   (280, 350),
}

_RULE = "global/file-limits"


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    ext = path.suffix.lower()
    limits = _LIMITS.get(ext)
    if limits is None:
        return

    soft, hard = limits
    count = len(lines)

    if count >= hard:
        yield Issue(
            file=path, line=count, col=1,
            severity=Severity.ERROR,
            rule=f"{_RULE}/{ext.lstrip('.')}",
            message=(
                f"file has {count} lines — hard limit is {hard}. "
                f"Split the module before adding anything."
            ),
        )
    elif count >= soft:
        yield Issue(
            file=path, line=count, col=1,
            severity=Severity.ERROR,
            rule=f"{_RULE}/{ext.lstrip('.')}",
            message=(
                f"file has {count} lines — approaching limit of {hard} "
                f"(soft={soft}). Plan the split now."
            ),
        )
