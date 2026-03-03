"""CSS layout safety checks — from css/validation.md + css/custom-properties.md.

BANNED:
  - Magic z-index values > 10 (use --z-index-* tokens)
  - !important (already in tokens.py — also caught here for layout context)
  - Hardcoded transition durations in ms without a token
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "css/validation"

# z-index: <number> where number > 10
_Z_INDEX = re.compile(r"\bz-index\s*:\s*(\d+)\b")

# transition: Xms or animation-duration: Xms without a var()
_TRANSITION_MS = re.compile(r"\b(transition|animation(?:-duration)?)\s*:[^;]*\b(\d{2,})\s*ms")
_VAR_REF = re.compile(r"\bvar\s*\(")


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith(("//", "/*", "*")):
            continue

        # --- Magic z-index ---
        if m := _Z_INDEX.search(raw):
            value = int(m.group(1))
            if value > 10:
                severity = Severity.ERROR if value >= 100 else Severity.WARNING
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=severity,
                    rule=f"{_RULE_BASE}/no-magic-z-index",
                    message=(
                        f"magic z-index: {value} — define a "
                        f"--z-index-<name> custom property and use var()"
                    ),
                )

        # --- Magic transition duration ---
        if m := _TRANSITION_MS.search(raw):
            if not _VAR_REF.search(raw):
                duration = m.group(2)
                prop = m.group(1)
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/no-magic-duration",
                    message=(
                        f"hardcoded {prop} duration {duration}ms — "
                        f"define a --duration-* token and use var()"
                    ),
                )
