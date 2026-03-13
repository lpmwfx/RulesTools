"""Rust hardcoded URL checks — from rust/constants.md.

Zero-literal architecture: URL strings must be named constants
from state/ modules or _cfg struct fields.

BANNED:
  - "http://..." or "https://..." string literals outside const/static

Exempt:
  - const / static definition lines (these ARE the named constant)
  - Test files and #[cfg(test)] blocks
  - Comments
"""

from __future__ import annotations

from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "rust/constants"

# URL string literal
_URL_STRING = re.compile(r'"(https?://[^"]+)"')

# const / static line — exempt
_CONST_DEF = re.compile(r"^\s*(?:pub\s+)?(?:const|static)\s+")


def _is_test_file(path: Path) -> bool:
    parts = path.parts
    return (
        "tests" in parts
        or "test" in parts
        or path.stem.endswith("_test")
        or path.stem.startswith("test_")
    )


def _is_test_context(lines: list[str], lineno: int) -> bool:
    for i in range(lineno - 2, max(lineno - 60, -1), -1):
        l = lines[i].strip()
        if "#[test]" in l or "#[cfg(test)]" in l:
            return True
        if l.startswith("mod tests") or l.startswith("mod test"):
            return True
    return False


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    if _is_test_file(path):
        return

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue
        if _CONST_DEF.match(raw):
            continue
        if _is_test_context(lines, lineno):
            continue

        for m in _URL_STRING.finditer(raw):
            url = m.group(1)
            # Truncate for display
            display = url if len(url) <= 40 else url[:37] + "..."
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-hardcoded-url",
                message=(
                    f'hardcoded URL "{display}" — '
                    f"use named const from state/ module or _cfg field"
                ),
            )
