"""Rust clone-spam check — from rust/ownership.md.

Flags functions where .clone() is called many times — a sign that
ownership has not been thought through (common AI pattern).
Also flags Arc/Rc::clone() aliases that hide the clone count.
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE = "rust/ownership/clone-spam"

_CLONE_CALL  = re.compile(r"\.clone\s*\(\s*\)")
_FN_START    = re.compile(r"^\s*(pub\s+)?(async\s+)?fn\s+\w+")
_BRACE_OPEN  = re.compile(r"\{")
_BRACE_CLOSE = re.compile(r"\}")

_WARN_THRESHOLD  = 3   # >= 3 clones in one function → warning
_ERROR_THRESHOLD = 6   # >= 6 clones → error


def _extract_functions(lines: list[str]) -> list[tuple[int, int, str]]:
    """Return list of (start_line, end_line, body_text) for each fn body."""
    fns: list[tuple[int, int, str]] = []
    i = 0
    while i < len(lines):
        raw = lines[i]
        if _FN_START.match(raw):
            # Find the opening brace
            depth = 0
            start = i
            body_lines: list[str] = []
            found_open = False
            while i < len(lines):
                line = lines[i]
                opens  = len(_BRACE_OPEN.findall(line))
                closes = len(_BRACE_CLOSE.findall(line))
                if opens and not found_open:
                    found_open = True
                depth += opens - closes
                body_lines.append(line)
                if found_open and depth <= 0:
                    fns.append((start + 1, i + 1, "\n".join(body_lines)))
                    i += 1
                    break
                i += 1
            continue
        i += 1
    return fns


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    # Skip test files
    if path.stem.startswith("test") or path.name == "tests.rs":
        return

    for start_ln, end_ln, body in _extract_functions(lines):
        # Skip test functions
        fn_context = "\n".join(lines[max(0, start_ln - 3):start_ln])
        if "#[test]" in fn_context or "#[cfg(test)]" in fn_context:
            continue

        count = len(_CLONE_CALL.findall(body))
        if count >= _ERROR_THRESHOLD:
            yield Issue(
                file=path, line=start_ln, col=1,
                severity=Severity.ERROR,
                rule=_RULE,
                message=(
                    f"{count} .clone() calls in this function — "
                    f"rethink ownership: use references, lifetimes, or Cow instead"
                ),
            )
        elif count >= _WARN_THRESHOLD:
            yield Issue(
                file=path, line=start_ln, col=1,
                severity=Severity.ERROR,
                rule=_RULE,
                message=(
                    f"{count} .clone() calls in this function — "
                    f"consider passing references or restructuring ownership"
                ),
            )
