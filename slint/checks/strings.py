"""Slint hardcoded string check — from slint/validation.md.

UI string literals in Slint components should come from typed properties,
not be hardcoded inline. Hardcoded strings cannot be configured, tested,
or localized.

BANNED:
  - String literals used directly as property values in component body
    (text: "Save", title: "Error", placeholder-text: "Type here...")
  - Exception: single-character strings and empty strings (icons, separators)
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE = "slint/validation/no-hardcoded-string"

# property: "literal string" — but not in property definitions or translations
# Matches lines like: text: "Save"  or  title: "Hello World"
_HARDCODED_STR = re.compile(
    r'^\s*([\w-]+)\s*:\s*"([^"]{2,})"'  # prop: "value" (2+ chars)
)

# Properties that legitimately use string literals
_ALLOWED_PROPS = {
    "accessible-label",   # accessibility — OK to hardcode for now
    "icon",               # icon name / path
    "source",             # image source
    "background-image",   # image path
}

# String values that look like icon names, paths, or format strings
_ALLOWED_PATTERNS = re.compile(r"^(@|#|/|\\|\d|\{|\[)")


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        m = _HARDCODED_STR.match(raw)
        if not m:
            continue

        prop = m.group(1).lower()
        value = m.group(2)

        if prop in _ALLOWED_PROPS:
            continue
        if _ALLOWED_PATTERNS.match(value):
            continue
        # Skip property declarations (property <string> foo: "default")
        if "property" in raw[:raw.index('"')]:
            continue

        yield Issue(
            file=path, line=lineno, col=raw.index('"') + 1,
            severity=Severity.ERROR,
            rule=_RULE,
            message=(
                f"hardcoded string '{value}' in component — "
                f"expose as an `in property <string> {prop}` so callers can configure it"
            ),
        )
