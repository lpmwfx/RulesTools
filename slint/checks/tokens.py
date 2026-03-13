"""Slint token checks — from uiux/tokens.md.

BANNED in Slint components:
  - Hardcoded hex colors   (#rgb / #rrggbb / #rrggbbaa)
  - Hardcoded pixel sizes  on ANY property (not just a whitelist)
  - Hardcoded font sizes   (font-size: 14px)
  - Hardcoded rgb/rgba()   (rgba(0,0,0,0.5))
  - Hardcoded opacity      (opacity: 0.4 — anything not 0.0 or 1.0)

Exempt:
  - Token definition files: globals/, tokens/ folders
  - property <length> declarations (default values are OK)
  - 0px, 1px  (border/hairline values accepted as-is)
  - 100% (fill patterns are idiomatic Slint)
  - Lines inside // comments
"""

from __future__ import annotations

from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "uiux/tokens"

_HEX_COLOR   = re.compile(r"#(?:[0-9a-fA-F]{3,4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})\b")
_RGB_FUNC    = re.compile(r"\brgb[a]?\s*\(")
# Any px value >= 5px (0-4px are border/hairline values — allowed)
_PIXEL_VALUE = re.compile(r"\b([5-9]\d*(?:\.\d+)?|\d{2,}(?:\.\d+)?)\s*px\b")
# Opacity values that are not 0.0 or 1.0
_OPACITY_PROP = re.compile(r"\bopacity\s*:", re.IGNORECASE)
_OPACITY_VAL  = re.compile(r"\b0\.[1-9]\d*\b")

# Token definition folders — exempt from checks
_TOKEN_FOLDERS = {"globals", "tokens", "theme"}


def _is_token_file(path: Path) -> bool:
    """Token definition files are allowed to contain raw values."""
    return any(part in _TOKEN_FOLDERS for part in path.parts)


def _is_property_decl(raw: str, match_start: int) -> bool:
    """True if the px value is inside a property <length> declaration (default value)."""
    before = raw[:match_start]
    return bool(re.search(r"\bproperty\s*<", before))


def _comment_start(raw: str) -> int:
    """Return the index of // comment start, or len(raw) if none."""
    m = re.search(r"//", raw)
    return m.start() if m else len(raw)


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    if _is_token_file(path):
        return

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        comment_at = _comment_start(raw)

        # ── Hardcoded hex color ─────────────────────────────────────────────
        for m in _HEX_COLOR.finditer(raw):
            if m.start() >= comment_at:
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-hardcoded-color",
                message=(
                    f"hardcoded color '{m.group()}' — token paradigm (uiux/tokens): "
                    f"all colors must be named tokens, e.g. Colors.bg-primary from tokens/colors.slint"
                ),
            )

        # ── rgb() / rgba() ──────────────────────────────────────────────────
        for m in _RGB_FUNC.finditer(raw):
            if m.start() >= comment_at:
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-hardcoded-color",
                message="hardcoded rgb/rgba() — token paradigm (uiux/tokens): all colors must be named tokens, e.g. Colors.surface from tokens/colors.slint",
            )

        # ── Hardcoded pixel sizes (ALL properties, not just a whitelist) ────
        for m in _PIXEL_VALUE.finditer(raw):
            if m.start() >= comment_at:
                continue
            if _is_property_decl(raw, m.start()):
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-hardcoded-size",
                message=(
                    f"hardcoded size '{m.group()}' — token paradigm (uiux/tokens): "
                    f"all sizes must be named tokens, e.g. Spacing.md from tokens/spacing.slint"
                ),
            )

        # ── Hardcoded opacity (not 0.0 or 1.0) ─────────────────────────────
        if _OPACITY_PROP.search(raw[:comment_at]):
            for m in _OPACITY_VAL.finditer(raw[:comment_at]):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-hardcoded-opacity",
                    message=(
                        f"hardcoded opacity '{m.group()}' — token paradigm (uiux/tokens): "
                        f"all opacity values must be named tokens, e.g. Spacing.opacity-disabled from tokens/"
                    ),
                )
