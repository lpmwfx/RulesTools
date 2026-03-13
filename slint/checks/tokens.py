"""Slint token checks — from uiux/tokens.md + slint/states.md.

Stateless, data-driven, no-literals architecture:
ANY literal value in a component file is hardcoding.

BANNED in Slint components:
  - Hardcoded hex colors   (#rgb / #rrggbb / #rrggbbaa)
  - Hardcoded rgb/rgba()   (rgba(0,0,0,0.5))
  - Hardcoded pixel sizes  (ALL values including 0px, 1px)
  - Hardcoded percentages  (100%, 50%, etc.)
  - Hardcoded durations    (200ms, 1s, etc.)
  - Hardcoded integers     (0, 1, 2, 400, 700, etc.)
  - Hardcoded floats       (0.5, 1.5, etc.)

Exempt (definition files):
  - Folders: globals/, tokens/, theme/, state/
  - Lines inside // comments
  - true/false (boolean keywords)

Slint syntax exceptions (compiler requires literals):
  - GridLayout row: / col:  (compile-time integer constants)
  - @image-url("...")       (compile-time string literal)
  - @tr("...") template     (compile-time string literal)
"""

from __future__ import annotations

from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "slint/states"
_RULE_COLORS = "uiux/tokens"

# ── Patterns ─────────────────────────────────────────────────────────────────

_HEX_COLOR = re.compile(r"#(?:[0-9a-fA-F]{3,4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})\b")
_RGB_FUNC = re.compile(r"\brgb[a]?\s*\(")

# ALL px values (including 0px, 1px)
_PIXEL_VALUE = re.compile(r"\b\d+(?:\.\d+)?\s*px\b")

# ALL percentage values (100%, 50%, 0%)
_PERCENT_VALUE = re.compile(r"\b\d+(?:\.\d+)?\s*%")

# ALL duration values (200ms, 1s, 0ms)
_DURATION_VALUE = re.compile(r"\b\d+(?:\.\d+)?\s*(?:ms|s)\b")

# Bare float literals (0.5, 1.0, 0.0, etc.)
_FLOAT_VALUE = re.compile(r"(?<![#\w])\b\d+\.\d+\b(?!\s*(?:px|%|ms|s)\b)")

# Bare integer literals (0, 1, 2, 400, etc.)
_INT_VALUE = re.compile(r"(?<![#\w\.\-])\b(\d+)\b(?!\s*(?:px|%|ms|s)\b)(?![.\w])")

# ── Syntax exceptions (Slint compiler requires literals) ─────────────────────

_GRID_ROW_COL = re.compile(r"\b(?:row|col)\s*:\s*\d+")
_IMAGE_URL = re.compile(r"@image-url\s*\(")
_TR_MACRO = re.compile(r"@tr\s*\(")

# ── Definition file folders — exempt from all checks ─────────────────────────

_DEFINITION_FOLDERS = {"globals", "tokens", "theme", "state"}


def _is_definition_file(path: Path) -> bool:
    return any(part in _DEFINITION_FOLDERS for part in path.parts)


def _is_property_decl(raw: str, match_start: int) -> bool:
    before = raw[:match_start]
    return bool(re.search(r"\bproperty\s*<", before))


def _comment_start(raw: str) -> int:
    m = re.search(r"//", raw)
    return m.start() if m else len(raw)


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    if _is_definition_file(path):
        return

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        comment_at = _comment_start(raw)
        segment = raw[:comment_at]

        # Skip lines with @image-url() or @tr() — syntax exceptions
        if _IMAGE_URL.search(segment) or _TR_MACRO.search(segment):
            continue

        # ── Hardcoded hex color ──────────────────────────────────────────
        for m in _HEX_COLOR.finditer(segment):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_COLORS}/no-hardcoded-color",
                message=f"hardcoded color '{m.group()}' — use Colors.* token",
            )

        # ── rgb() / rgba() ───────────────────────────────────────────────
        for m in _RGB_FUNC.finditer(segment):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_COLORS}/no-hardcoded-color",
                message="hardcoded rgb/rgba() — use Colors.* token",
            )

        # ── Hardcoded pixel sizes (ALL, including 0px) ───────────────────
        for m in _PIXEL_VALUE.finditer(segment):
            if _is_property_decl(raw, m.start()):
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-hardcoded-size",
                message=f"hardcoded size '{m.group().strip()}' — use Sizes.* or Spacing.* variable",
            )

        # ── Hardcoded percentages (ALL, including 100%) ──────────────────
        for m in _PERCENT_VALUE.finditer(segment):
            if _is_property_decl(raw, m.start()):
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-hardcoded-size",
                message=f"hardcoded percentage '{m.group().strip()}' — use Sizes.full / Sizes.half variable",
            )

        # ── Hardcoded durations (ALL, including 0ms) ─────────────────────
        for m in _DURATION_VALUE.finditer(segment):
            if _is_property_decl(raw, m.start()):
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-hardcoded-duration",
                message=f"hardcoded duration '{m.group().strip()}' — use Durations.* variable",
            )

        # ── Hardcoded float literals ─────────────────────────────────────
        for m in _FLOAT_VALUE.finditer(segment):
            if _is_property_decl(raw, m.start()):
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-hardcoded-number",
                message=f"hardcoded float '{m.group()}' — use state variable or theme token",
            )

        # ── Hardcoded integer literals ───────────────────────────────────
        for m in _INT_VALUE.finditer(segment):
            if _is_property_decl(raw, m.start()):
                continue
            if _GRID_ROW_COL.search(raw):
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-hardcoded-number",
                message=f"hardcoded integer '{m.group(1)}' — use state variable (ViewStates.*, Sizes.*)",
            )
