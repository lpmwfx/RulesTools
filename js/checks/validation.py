"""JS validation checks — from js/validation.md.

BANNED / REQUIRED at system boundaries:
  - JSON.parse() without try/catch or .safeParse nearby
  - fetch() response used without schema validation
  - Response.json() used without schema validation
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "js/validation"

_JSON_PARSE   = re.compile(r"\bJSON\.parse\s*\(")
_FETCH_CALL   = re.compile(r"\bfetch\s*\(")
_RES_JSON     = re.compile(r"\.(json|text)\s*\(\s*\)")

# Schema validation indicators (Zod/Valibot)
_SCHEMA_PARSE = re.compile(r"\.(parse|safeParse|parseAsync)\s*\(")


def _nearby(lines: list[str], lineno: int, pattern: re.Pattern, window: int = 5) -> bool:
    """Check if pattern appears within ±window lines of lineno (1-indexed)."""
    start = max(0, lineno - window - 1)
    end   = min(len(lines), lineno + window)
    return any(pattern.search(l) for l in lines[start:end])


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        # --- JSON.parse() without schema validation nearby ---
        if m := _JSON_PARSE.search(raw):
            if not _nearby(lines, lineno, _SCHEMA_PARSE):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/schema-at-boundary",
                    message=(
                        "JSON.parse() without schema validation — "
                        "pipe through Schema.parse()/safeParse() at this boundary"
                    ),
                )

        # --- fetch().json() without schema validation nearby ---
        if _FETCH_CALL.search(raw) or _RES_JSON.search(raw):
            if _RES_JSON.search(raw) and not _nearby(lines, lineno, _SCHEMA_PARSE, window=8):
                yield Issue(
                    file=path, line=lineno, col=1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/schema-at-boundary",
                    message=(
                        ".json()/.text() response without schema validation — "
                        "validate with Schema.safeParse() before using the data"
                    ),
                )
