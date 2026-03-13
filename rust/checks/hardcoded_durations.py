"""Rust hardcoded duration checks — from rust/constants.md.

Zero-literal architecture: Duration values must be named constants
from state/ modules or _cfg struct fields.

BANNED:
  - Duration::from_secs(N) with literal N
  - Duration::from_millis(N) with literal N
  - Duration::from_nanos(N) with literal N
  - Duration::from_micros(N) with literal N
  - Duration::new(N, N) with literal N
  - tokio::time::sleep(Duration::from_*) with literal

Exempt:
  - const / static definition lines
  - Test files and #[cfg(test)] blocks
  - Comments
  - Duration::from_secs(0) — zero is exempt everywhere
"""

from __future__ import annotations

from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "rust/constants"

# Duration constructors with literal numeric arguments
_DURATION_LITERAL = re.compile(
    r"Duration::(?:from_secs|from_millis|from_nanos|from_micros|new)\s*\(\s*(\d+)"
)

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

        for m in _DURATION_LITERAL.finditer(raw):
            val = m.group(1)
            # Duration::from_secs(0) is exempt (zero is universal)
            if val == "0":
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-hardcoded-duration",
                message=(
                    f"hardcoded duration literal {val} — "
                    f"use named const from state/ module, "
                    f"e.g. Duration::from_secs(TIMEOUT_SECS)"
                ),
            )
