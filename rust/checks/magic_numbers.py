"""Rust magic number checks — from rust/types.md.

Unnamed numeric literals embedded in logic are hard to understand and
maintain. All non-trivial numbers should be named constants.

BANNED:
  - f32/f64 literals >= 2.0 that are not in a const/static definition
  - Integer literals >= 10 used in arithmetic/comparisons outside const

Exempt:
  - const / static definition lines (these ARE the named constant)
  - 0, 1, 2 (universal idioms)
  - 0.0, 1.0, 0.5 (normalised values)
  - -1 (sentinel)
  - Test files and #[cfg(test)] blocks
  - Format strings and log messages (inside a string literal)
"""

from __future__ import annotations

from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "rust/types"

# f32/f64 literal: digits.digits (not 0.0, 1.0, 0.5, 0.25)
_FLOAT_LIT = re.compile(r"\b(\d+\.\d+)\b")
_TRIVIAL_FLOATS = {"0.0", "1.0", "0.5", "0.25", "2.0", "-1.0", "0.1"}

# Integer literal >= 10 used in comparison or arithmetic (not in string)
# Matches:  >= 500   > 100   + 80   * 24   % 60
_INT_IN_EXPR = re.compile(r"(?:[><=!+\-*/%&|,\s])(\d{2,})\b")
_TRIVIAL_INTS = {
    "10", "16", "32", "64", "100", "128", "255", "256",
    "1000", "1024",
}  # powers-of-two and round numbers kept if obvious

# const / static line — these are definitions, not violations
_CONST_DEF = re.compile(r"^\s*(?:pub\s+)?(?:const|static)\s+")

# Inside a string literal — skip
_STRING_LIT = re.compile(r'"[^"]*"')


def _strip_strings(raw: str) -> str:
    """Remove string literal contents to avoid matching numbers inside strings."""
    return _STRING_LIT.sub('""', raw)


def _is_test_file(path: Path) -> bool:
    parts = path.parts
    return (
        "tests" in parts
        or "test" in parts
        or path.stem.endswith("_test")
        or path.stem.startswith("test_")
    )


def _is_test_context(lines: list[str], lineno: int) -> bool:
    for i in range(lineno - 2, max(lineno - 60, -1), -1):
        l = lines[i].strip()
        if "#[test]" in l or "#[cfg(test)]" in l:
            return True
        if l.startswith("mod tests") or l.startswith("mod test"):
            return True
    return False


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    if _is_test_file(path):
        return

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue
        if _CONST_DEF.match(raw):
            continue
        if _is_test_context(lines, lineno):
            continue

        clean = _strip_strings(raw)

        # ── Unnamed float literal ────────────────────────────────────────────
        for m in _FLOAT_LIT.finditer(clean):
            val = m.group(1)
            if val in _TRIVIAL_FLOATS:
                continue
            try:
                if float(val) < 2.0:
                    continue
            except ValueError:
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-magic-number",
                message=(
                    f"magic number {val} — config-driven paradigm (rust/types): "
                    f"all values must be named, e.g. const NAME: f32 = {val};"
                ),
            )

        # ── Unnamed large integer in expression ──────────────────────────────
        # Collect float positions so we don't double-report e.g. 80 in "80.0"
        float_positions = {m.start() for m in _FLOAT_LIT.finditer(clean)}

        for m in _INT_IN_EXPR.finditer(clean):
            val = m.group(1)
            # Skip if this integer is the integer part of a float already reported
            if any(abs(m.start(1) - fp) <= len(val) + 1 for fp in float_positions):
                continue
            if val in _TRIVIAL_INTS:
                continue
            try:
                n = int(val)
            except ValueError:
                continue
            if n < 10:
                continue
            # Skip powers of two (common bit masks / sizes)
            if n > 0 and (n & (n - 1)) == 0:
                continue
            yield Issue(
                file=path, line=lineno, col=m.start(1) + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-magic-number",
                message=(
                    f"magic number {val} — config-driven paradigm (rust/types): "
                    f"all values must be named, e.g. const NAME: u32 = {val};"
                ),
            )
