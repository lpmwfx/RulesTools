"""TypeScript safety checks — from js/safety.md.

BANNED:
  - `: any` / `as any` — defeats the type system entirely
  - `@ts-ignore` / `@ts-nocheck` — suppresses type errors instead of fixing them
  - Non-null assertion `!` on member access (foo!.bar) — hides null bugs
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "js/safety"

# : any or as any
_TYPE_ANY = re.compile(r"(?::\s*any\b|as\s+any\b)")

# @ts-ignore or @ts-nocheck
_TS_SUPPRESS = re.compile(r"@ts-(ignore|nocheck)\b")

# foo!.bar or foo![ — non-null assertion on access
_NON_NULL = re.compile(r"\w!\s*[.\[]")


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    # Only apply to TypeScript files
    if path.suffix not in {".ts", ".tsx"}:
        return

    is_test = "test" in path.parts or path.stem.endswith((".test", ".spec"))

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            # ts-suppress directives ARE in comments — still flag them
            if m := _TS_SUPPRESS.search(raw):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-ts-suppress",
                    message=(
                        f"@ts-{m.group(1)} suppresses type errors — "
                        f"fix the underlying type issue instead"
                    ),
                )
            continue

        # --- : any / as any ---
        for m in _TYPE_ANY.finditer(raw):
            before = raw[: m.start()]
            if before.count('"') % 2 or before.count("'") % 2:
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-any",
                message=(
                    "'any' removes TypeScript's type safety — "
                    "use 'unknown' + type guard, or define a proper interface"
                ),
            )

        # --- Non-null assertion ---
        if not is_test:
            for m in _NON_NULL.finditer(raw):
                before = raw[: m.start()]
                if before.count('"') % 2 or before.count("'") % 2:
                    continue
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/no-non-null-assertion",
                    message=(
                        "non-null assertion '!' hides a potential null/undefined — "
                        "use optional chaining '?.' or add an explicit null check"
                    ),
                )
