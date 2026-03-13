"""Rust security checks — global/error-flow.md + rust/errors.md.

Checks (not duplicating errors.py which already handles unwrap/expect/panic):
  - Wildcard error arm _ => {} / _ => () — silent error swallow
  - Command::new + .arg(format!..) — potential argument injection
"""

from __future__ import annotations
import re
from pathlib import Path
from typing import Generator

from common.issue import Issue, Severity

_RULE_BASE = "rust/security"

# _ => {} or _ => () or _ => { /* anything trivial */ at end of arm
_WILDCARD_EMPTY = re.compile(r"\b_\s*=>\s*(?:\{\s*\}|\(\s*\))\s*,?")

# .arg(format!(...)) — user-controlled data passed as a command argument string
_CMD_FORMAT_ARG = re.compile(r"\.arg\s*\(\s*format\s*!")


def _is_test_context(lines: list[str], lineno: int) -> bool:
    for i in range(lineno - 2, max(lineno - 102, -1), -1):
        line = lines[i]
        if "#[test]" in line or "#[cfg(test)]" in line:
            return True
    return False


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        in_test = _is_test_context(lines, lineno)

        if not in_test:
            if m := _WILDCARD_EMPTY.search(raw):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-wildcard-swallow",
                    message=(
                        "wildcard arm _ => {} silently discards the error — "
                        "match each variant explicitly with a recovery action"
                    ),
                )

            if m := _CMD_FORMAT_ARG.search(raw):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/cmd-arg-injection",
                    message=(
                        ".arg(format!(...)) passes formatted string as command argument — "
                        "validate or sanitize user-supplied values before use"
                    ),
                )
