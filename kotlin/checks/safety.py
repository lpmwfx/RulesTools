"""Kotlin safety checks — from kotlin/encapsulation.md + kotlin/result-pattern.md.

BANNED:
  - !! operator (unsafe null assertion) — use safe-call + elvis or Result
  - import java.*  outside platform/ layer
  - import javax.* outside platform/
  - import java.awt.* outside platform/
  - Multiple class/object/interface definitions per file
  - throw Exception() / throw RuntimeException() — use sealed Result
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "kotlin"

_NOT_NULL    = re.compile(r"!!")
_JAVA_IMPORT = re.compile(r"^import\s+(java|javax|android\.os)\.")
_THROW_BARE  = re.compile(r"\bthrow\s+(Exception|RuntimeException|IllegalStateException|IllegalArgumentException)\s*\(")

# Class/object/interface/sealed class definitions (not inside companion objects)
_TYPE_DEF    = re.compile(r"^(?:(?:public|private|internal|protected|abstract|sealed|open|data|enum)\s+)*(?:class|object|interface)\s+(\w+)", re.MULTILINE)


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    is_platform = "platform" in path.parts

    text = "\n".join(lines)

    # --- Multiple type definitions per file ---
    type_defs = list(_TYPE_DEF.finditer(text))
    if len(type_defs) >= 2:
        # Find line number of second definition
        char_count = 0
        line_starts = [0]
        for line in lines:
            char_count += len(line) + 1
            line_starts.append(char_count)

        def pos_to_line(pos: int) -> int:
            for i, start in enumerate(line_starts):
                if start > pos:
                    return i
            return len(lines)

        names = [m.group(1) for m in type_defs]
        for m in type_defs[1:]:
            lineno = pos_to_line(m.start())
            yield Issue(
                file=path, line=lineno, col=1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/encapsulation/one-class-per-file",
                message=(
                    f"'{m.group(1)}' — multiple types in one file. "
                    f"Extract to '{m.group(1)}.kt' (primary: '{names[0]}')"
                ),
            )

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        # --- !! operator ---
        for m in _NOT_NULL.finditer(raw):
            # Skip string literals (simplified)
            before = raw[: m.start()]
            if before.count('"') % 2:
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/safety/no-not-null-assertion",
                message=(
                    "!! unsafe null assertion — use safe-call (?.) + "
                    "elvis (?:) or return Result.Error"
                ),
            )

        # --- java.* imports outside platform/ ---
        if not is_platform:
            if m := _JAVA_IMPORT.match(stripped):
                yield Issue(
                    file=path, line=lineno, col=1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/encapsulation/java-in-platform-only",
                    message=(
                        f"'{raw.strip()}' — Java/Android imports only allowed in platform/ layer. "
                        f"Wrap behind a Kotlin interface."
                    ),
                )

        # --- bare throw Exception ---
        if m := _THROW_BARE.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/result-pattern/no-bare-throw",
                message=(
                    f"throw {m.group(1)}() — use sealed Result.Error for expected failures"
                ),
            )
