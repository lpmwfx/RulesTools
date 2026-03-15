"""Nesting depth checker — from global/nesting.md.

Rule: Max 3 nesting levels — extract to helper if deeper.
Banned: 4+ levels.

Strategy: brace-depth tracking with string/comment stripping.
Works for Rust, JS, Slint (all use { } blocks).

We count depth RELATIVE to the enclosing function/closure/component body,
so top-level struct/impl/component definitions do not inflate the count.

Depth accounting:
  - Opening  `{` increments depth
  - Closing  `}` decrements depth
  - We report lines where the *code content* is at relative depth > WARN_AT
    after subtracting the "base depth" (outermost function/closure we entered)

For simplicity we track absolute brace depth and flag anything beyond
MAX_ABS_DEPTH, which is calibrated per language:

  Rust:  fn body = depth 1 (top-level fn) or 2 (impl method)
         Match arm `=> {` excluded via ignore_open_patterns (syntactic, not logical)
         → flag at absolute depth >= 8  (impl + fn + 6 logic levels)
  JS:    fn body = depth 1
         → flag at absolute depth >= 4  (fn + 3 logic levels)
  Slint: component body = depth 1
         → flag at absolute depth >= 4  (component + 3 levels)

Callers pass `max_abs_depth` appropriate for the language.
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from .issue import Issue, Severity

_RULE = "global/nesting"

# Patterns to strip before counting braces
_SINGLE_LINE_COMMENT_RS = re.compile(r"//.*")
_SINGLE_LINE_COMMENT_JS = re.compile(r"//.*")
_STRING_DQ = re.compile(r'"(?:[^"\\]|\\.)*"')
_STRING_SQ = re.compile(r"'(?:[^'\\]|\\.)*'")
_STRING_BACKTICK = re.compile(r"`(?:[^`\\]|\\.)*`")


def _strip_strings_and_comments(line: str, lang: str) -> str:
    """Remove string literals and line comments so braces inside them are ignored."""
    line = _STRING_DQ.sub('""', line)
    line = _STRING_SQ.sub("''", line)
    if lang == "js":
        line = _STRING_BACKTICK.sub("``", line)
    # Strip line comment
    line = re.sub(r"//.*", "", line)
    return line


def check(
    path: Path,
    lines: list[str],
    lang: str,
    max_abs_depth: int,
    ignore_open_patterns: list[re.Pattern] | None = None,
) -> Generator[Issue, None, None]:
    """Yield issues for lines that exceed max_abs_depth brace nesting.

    ignore_open_patterns — lines matching any of these patterns still update
    depth tracking but do NOT trigger an error.  Used to exempt Rust match
    arm openers (`=> {`) which are syntactically required, not logical nesting.
    """
    depth = 0
    in_block_comment = False

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.rstrip()

        # Block comment tracking (/* ... */)
        if in_block_comment:
            if "*/" in stripped:
                idx = stripped.index("*/")
                stripped = stripped[idx + 2:]
                in_block_comment = False
            else:
                continue  # entire line is inside block comment

        # Remove block comment openers on this line
        while "/*" in stripped:
            start = stripped.index("/*")
            if "*/" in stripped[start:]:
                end = stripped.index("*/", start) + 2
                stripped = stripped[:start] + stripped[end:]
            else:
                stripped = stripped[:start]
                in_block_comment = True
                break

        clean = _strip_strings_and_comments(stripped, lang)

        opens = clean.count("{")
        closes = clean.count("}")

        # Single-line blocks  `=> { stmt; }`  or  `{ }` open+close on same line.
        # These are property-style assignments (Slint callbacks, Rust closures)
        # and do NOT represent real nesting depth — skip them.
        net = opens - closes
        if opens > 0 and net == 0:
            continue

        depth_before = depth
        depth += net
        depth = max(depth, 0)  # guard against malformed files

        # Flag when this line's opening brace pushes depth to/past the limit
        if opens > 0 and depth >= max_abs_depth:
            # Skip lines matching ignore patterns (e.g. match arm `=> {`)
            if ignore_open_patterns and any(p.search(clean) for p in ignore_open_patterns):
                continue
            col = raw.index("{") + 1 if "{" in raw else 1
            yield Issue(
                file=path, line=lineno, col=col,
                severity=Severity.ERROR,
                rule=_RULE,
                message=(
                    f"nesting depth {depth} exceeds limit of "
                    f"{max_abs_depth - 1} logic levels — extract a helper function"
                ),
            )
