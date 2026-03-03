"""Rust error-handling checks — from rust/errors.md.

BANNED:
  - unwrap() / expect()  in non-test code
  - panic!()             in non-test code
  - Box<dyn Error>       (use concrete error types)
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "rust/errors"

_UNWRAP     = re.compile(r"\.\s*unwrap\s*\(\s*\)")
_EXPECT     = re.compile(r"\.\s*expect\s*\(")
_PANIC      = re.compile(r"\bpanic!\s*\(")
_BOX_DYN    = re.compile(r"Box\s*<\s*dyn\s+Error")


def _is_test_context(lines: list[str], lineno: int) -> bool:
    """Heuristic: are we inside a #[test] or #[cfg(test)] block?"""
    # Look back up to 50 lines for #[test] or #[cfg(test)]
    for i in range(lineno - 2, max(lineno - 52, -1), -1):
        l = lines[i].strip()
        if "#[test]" in l or "#[cfg(test)]" in l:
            return True
        # Stop at fn definitions that are NOT test fns
        if re.match(r"(pub\s+)?(async\s+)?fn\s+\w+", l) and "#[test]" not in l:
            return False
    return False


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    for lineno, raw in enumerate(lines, start=1):
        # Skip comment lines
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        in_test = _is_test_context(lines, lineno)

        if not in_test:
            for m in _UNWRAP.finditer(raw):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-unwrap",
                    message="unwrap() in non-test code — use ? or match",
                )
            for m in _EXPECT.finditer(raw):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-unwrap",
                    message="expect() in non-test code — use ? or map_err",
                )
            for m in _PANIC.finditer(raw):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-panic",
                    message="panic!() for recoverable error — return Err(...) instead",
                )

        for m in _BOX_DYN.finditer(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.WARNING,
                rule=f"{_RULE_BASE}/no-box-dyn-error",
                message="Box<dyn Error> — use a concrete error type (thiserror)",
            )
