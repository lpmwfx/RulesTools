"""C# type safety checks — from csharp/types.md.

BANNED:
  - #nullable disable  — nullable reference types must stay on
  - dynamic            — use generics or interfaces
  - object as parameter or return type — use generics or interfaces
"""

from __future__ import annotations
import re
from pathlib import Path
from typing import Generator

from common.issue import Issue, Severity

_RULE_BASE = "csharp/types"

_NULLABLE_DISABLE = re.compile(r"#nullable\s+disable")
_DYNAMIC_USE      = re.compile(r"\bdynamic\b")
# object as param or return: "object " at word boundary, not inside string/comment
_OBJECT_PARAM     = re.compile(r"\bobject\s+\w")


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//") or stripped.startswith("*"):
            continue

        if m := _NULLABLE_DISABLE.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-nullable-disable",
                message=(
                    "#nullable disable — nullable reference types must be enabled. "
                    "Use T? for intentionally nullable values."
                ),
            )

        if m := _DYNAMIC_USE.search(raw):
            # Skip comments and string literals (simple heuristic)
            before = raw[: m.start()]
            if before.count('"') % 2 == 0:
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-dynamic",
                    message=(
                        "`dynamic` — use proper generics or interfaces instead"
                    ),
                )

        if m := _OBJECT_PARAM.search(raw):
            before = raw[: m.start()]
            if before.count('"') % 2 == 0:
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-object-param",
                    message=(
                        "`object` as parameter or variable — "
                        "use generics (<T>) or a concrete interface instead"
                    ),
                )
