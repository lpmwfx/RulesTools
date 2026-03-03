"""Slint token checks — from uiux/tokens.md.

BANNED in Slint components:
  - Hardcoded hex colors   (#rgb / #rrggbb / #rrggbbaa)
  - Hardcoded pixel sizes  (e.g. width: 42px  height: 100px)
  - Hardcoded font sizes   (font-size: 14px)
  - Hardcoded rgb/rgba()   (rgba(0,0,0,0.5))

All values must reference named tokens from the design token palette.
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "uiux/tokens"

_HEX_COLOR   = re.compile(r"#(?:[0-9a-fA-F]{3,4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})\b")
_PIXEL_VALUE = re.compile(r"\b(\d+(?:\.\d+)?)\s*px\b")
_RGB_FUNC    = re.compile(r"\brgb[a]?\s*\(")

# Properties where pixel values are expected to be tokens
_TOKEN_PROPS = re.compile(
    r"\b(width|height|min-width|min-height|max-width|max-height"
    r"|padding|margin|spacing|font-size|border-radius|border-width)\s*:",
    re.IGNORECASE,
)


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        # Skip comment lines
        if stripped.startswith("//"):
            continue

        # --- Hardcoded hex color ---
        for m in _HEX_COLOR.finditer(raw):
            # Skip if inside a comment
            before = raw[: m.start()]
            if "//" in before:
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-hardcoded-color",
                message=(
                    f"hardcoded color '{m.group()}' — "
                    f"use a named token (e.g. Theme.color-primary)"
                ),
            )

        # --- Hardcoded pixel sizes on token properties ---
        if _TOKEN_PROPS.search(raw):
            for m in _PIXEL_VALUE.finditer(raw):
                before = raw[: m.start()]
                if "//" in before:
                    continue
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/no-hardcoded-size",
                    message=(
                        f"hardcoded size '{m.group()}' — "
                        f"use a named token (e.g. Theme.spacing-md)"
                    ),
                )

        # --- rgb() / rgba() ---
        for m in _RGB_FUNC.finditer(raw):
            before = raw[: m.start()]
            if "//" in before:
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-hardcoded-color",
                message=(
                    f"hardcoded rgb/rgba() — "
                    f"use a named token (e.g. Theme.color-surface)"
                ),
            )
