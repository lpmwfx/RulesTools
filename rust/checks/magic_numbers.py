"""Rust magic number checks — from rust/constants.md + global/config-driven.md.

Zero-literal architecture: ALL numeric values in function bodies must be
named constants (const/static) or config fields (_cfg).

BANNED in function bodies:
  - ALL integer literals >= 2  (use named const or cfg.field)
  - ALL float literals except 0.0 and 1.0  (use named const or cfg.field)

Exempt:
  - 0 and 1 (universal indexing, ranges, arithmetic)
  - 0.0 and 1.0 (normalised values)
  - const / static definition lines (these ARE the named constant)
  - Enum variant discriminant lines (Variant = 3)
  - Test files and #[cfg(test)] blocks
  - Format/log/assert macro lines (inside string templates)
  - Comments
"""

from __future__ import annotations

from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "rust/constants"

# Float literal: digits.digits (not 0.0, 1.0)
_FLOAT_LIT = re.compile(r"\b(\d+\.\d+)\b")
_TRIVIAL_FLOATS = {"0.0", "1.0"}

# Integer literal — any bare number
_INT_LIT = re.compile(r"(?<![#\w.\-])\b(\d+)\b(?![.\w])")

# const / static line — these are definitions, not violations
_CONST_DEF = re.compile(r"^\s*(?:pub\s+)?(?:const|static)\s+")

# Enum variant with discriminant:  Variant = 3,
_ENUM_VARIANT = re.compile(r"^\s*\w+\s*=\s*\d+")

# Inside a string literal — skip
_STRING_LIT = re.compile(r'"[^"]*"')

# Macro invocations that take format strings (skip entire line)
_FORMAT_MACRO = re.compile(
    r"\b(?:format|println|eprintln|print|eprint|write|writeln|"
    r"tracing::info|tracing::debug|tracing::warn|tracing::error|tracing::trace|"
    r"info|debug|warn|error|trace|log::info|log::debug|log::warn|log::error|"
    r"panic|todo|unimplemented|unreachable|assert|assert_eq|assert_ne|"
    r"anyhow::bail|anyhow::anyhow|bail)!\s*\("
)


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


def _is_format_macro_line(raw: str) -> bool:
    """Check if the line is inside a format/log/assert macro."""
    return bool(_FORMAT_MACRO.search(raw))


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    if _is_test_file(path):
        return

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue
        if _CONST_DEF.match(raw):
            continue
        if _ENUM_VARIANT.match(stripped):
            continue
        if _is_test_context(lines, lineno):
            continue
        if _is_format_macro_line(raw):
            continue

        clean = _strip_strings(raw)

        # ── Unnamed float literal ────────────────────────────────────────────
        for m in _FLOAT_LIT.finditer(clean):
            val = m.group(1)
            if val in _TRIVIAL_FLOATS:
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-magic-number",
                message=(
                    f"magic number {val} — use named const or _cfg field, "
                    f"e.g. const NAME: f64 = {val};"
                ),
            )

        # ── Unnamed integer literal >= 2 ─────────────────────────────────────
        float_positions = {m.start() for m in _FLOAT_LIT.finditer(clean)}

        for m in _INT_LIT.finditer(clean):
            val = m.group(1)
            # Skip if part of a float
            if any(abs(m.start(1) - fp) <= len(val) + 1 for fp in float_positions):
                continue
            try:
                n = int(val)
            except ValueError:
                continue
            # 0 and 1 are universal idioms — exempt
            if n <= 1:
                continue
            yield Issue(
                file=path, line=lineno, col=m.start(1) + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-magic-number",
                message=(
                    f"magic number {val} — use named const or _cfg field, "
                    f"e.g. const NAME: usize = {val};"
                ),
            )
