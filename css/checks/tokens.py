"""CSS token checks — from css/custom-properties.md.

BANNED:
  - Hardcoded hex colors        (#rgb / #rrggbb) — use var(--color-*)
  - Hardcoded rgb/rgba()        — use var(--color-*)
  - Hardcoded px for typography — use rem or var(--*)
  - !important                  — signals broken cascade
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "css/custom-properties"

_HEX_COLOR   = re.compile(r"(?<![\w-])#(?:[0-9a-fA-F]{3,4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})\b")
_RGB_FUNC    = re.compile(r"\brgba?\s*\(")
_IMPORTANT   = re.compile(r"!\s*important")

# font-size / line-height with raw px values (should use rem or var)
_FONT_PX     = re.compile(r"\b(font-size|line-height)\s*:\s*[\d.]+px\b")

# Selector or value context — skip @keyframes color stops etc.
_COMMENT     = re.compile(r"/\*.*?\*/", re.DOTALL)


def _strip_comments(text: str) -> str:
    return _COMMENT.sub("", text)


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        # Skip line comments (// not valid CSS but sometimes used in SCSS)
        if stripped.startswith("//"):
            continue
        # Skip lines that are inside block comments (simplified — per-line check)
        if stripped.startswith("*") or stripped.startswith("/*"):
            continue

        # --- Hardcoded hex color ---
        for m in _HEX_COLOR.finditer(raw):
            before = raw[: m.start()]
            if "/*" in before or "//" in before:
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-hardcoded-color",
                message=(
                    f"hardcoded color '{m.group()}' — "
                    f"use a custom property: var(--color-...)"
                ),
            )

        # --- rgb/rgba() ---
        for m in _RGB_FUNC.finditer(raw):
            before = raw[: m.start()]
            if "/*" in before or "//" in before:
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-hardcoded-color",
                message="rgb/rgba() — use a custom property: var(--color-...)",
            )

        # --- !important ---
        if m := _IMPORTANT.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule="css/cascade/no-important",
                message=(
                    "!important — signals broken cascade. "
                    "Fix specificity or use custom properties instead."
                ),
            )

        # --- font-size / line-height in px ---
        if m := _FONT_PX.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/use-rem",
                message=(
                    f"{m.group(1)} in px — use rem or var(--*) "
                    f"so the user's browser font settings are respected"
                ),
            )
