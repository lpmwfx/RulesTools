"""File size checks — from global/file-limits.md.

Limits by extension (code lines only — comments, blank lines, and
pure string/doc lines are excluded from the count):
  .slint          soft=200  hard=250
  .js / .ts       soft=200  hard=250
  .css / .scss    soft=120  hard=150
  .py             soft=200  hard=250
  .rs             soft=200  hard=250
  .cpp / .h       soft=280  hard=350

"Approaching" = within 20% of hard limit = soft limit.
Soft  → warning  ("plan the split")
Hard  → error    ("stop, split now")
"""

from __future__ import annotations
import re
from pathlib import Path
from typing import Generator

from .issue import Issue, Severity
from .standalone import is_standalone

# (soft_limit, hard_limit)
_LIMITS: dict[str, tuple[int, int]] = {
    ".slint": (200, 250),
    ".js":    (200, 250),
    ".ts":    (200, 250),
    ".mjs":   (200, 250),
    ".css":   (120, 150),
    ".scss":  (120, 150),
    ".py":    (200, 250),
    ".rs":    (200, 250),
    ".cpp":   (280, 350),
    ".cc":    (280, 350),
    ".cxx":   (280, 350),
    ".h":     (280, 350),
    ".hpp":   (280, 350),
}

_RULE = "global/file-limits"

# Patterns for non-code lines (per language family)
_LINE_COMMENT = re.compile(r"^\s*//")           # C-style line comment (// /// //!)
_BLOCK_OPEN   = re.compile(r"^\s*/\*")          # block comment open
_BLOCK_CLOSE  = re.compile(r"\*/\s*$")          # block comment close
_HASH_COMMENT = re.compile(r"^\s*#")            # Python/CSS comment
_BLANK        = re.compile(r"^\s*$")            # blank line
_PURE_STRING  = re.compile(r'^\s*"[^"]*"\s*;?\s*$')  # line that is only a string literal

_C_FAMILY = frozenset(
    {".rs", ".slint", ".js", ".ts", ".mjs", ".cpp", ".cc", ".cxx", ".h", ".hpp"},
)
_HASH_FAMILY = frozenset({".py"})
_CSS_FAMILY  = frozenset({".css", ".scss"})


def _count_code_lines(ext: str, lines: list[str]) -> int:
    """Count lines that are actual code — skip comments, blanks, pure strings."""
    count = 0
    in_block = False

    for raw in lines:
        # blank lines never count
        if _BLANK.match(raw):
            continue

        if ext in _C_FAMILY:
            if in_block:
                if _BLOCK_CLOSE.search(raw):
                    in_block = False
                continue
            if _BLOCK_OPEN.match(raw):
                if not _BLOCK_CLOSE.search(raw):
                    in_block = True
                continue
            if _LINE_COMMENT.match(raw):
                continue
            if _PURE_STRING.match(raw):
                continue

        elif ext in _HASH_FAMILY:
            if _HASH_COMMENT.match(raw):
                continue
            if _PURE_STRING.match(raw):
                continue

        elif ext in _CSS_FAMILY:
            if in_block:
                if _BLOCK_CLOSE.search(raw):
                    in_block = False
                continue
            if _BLOCK_OPEN.match(raw):
                if not _BLOCK_CLOSE.search(raw):
                    in_block = True
                continue

        count += 1

    return count


_STANDALONE_LIMITS = (800, 1000)   # tools/, scripts/, src/bin/


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    ext = path.suffix.lower()
    limits = _LIMITS.get(ext)
    if limits is None:
        return

    if is_standalone(path):
        soft, hard = _STANDALONE_LIMITS
    else:
        soft, hard = limits
    count = _count_code_lines(ext, lines)

    if count >= hard:
        yield Issue(
            file=path, line=len(lines), col=1,
            severity=Severity.ERROR,
            rule=f"{_RULE}/{ext.lstrip('.')}",
            message=(
                f"file has {count} code lines (of {len(lines)} total) — "
                f"hard limit is {hard}. "
                f"Split the module before adding anything."
            ),
        )
    elif count >= soft:
        yield Issue(
            file=path, line=len(lines), col=1,
            severity=Severity.ERROR,
            rule=f"{_RULE}/{ext.lstrip('.')}",
            message=(
                f"file has {count} code lines (of {len(lines)} total) — "
                f"approaching limit of {hard} "
                f"(soft={soft}). Plan the split now."
            ),
        )
