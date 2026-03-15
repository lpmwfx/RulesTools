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
# Regex::new(...).unwrap() inside LazyLock — panics on invalid pattern (developer error, caught at startup)
_REGEX_INIT = re.compile(r"\bRegex(?:Set)?::new\s*\(")


def _is_test_context(lines: list[str], lineno: int) -> bool:
    """Heuristic: are we inside a #[test] or #[cfg(test)] block?"""
    # Look back up to 100 lines for #[test] or #[cfg(test)]
    for i in range(lineno - 2, max(lineno - 102, -1), -1):
        l = lines[i].strip()
        if "#[test]" in l or "#[cfg(test)]" in l:
            return True
        # Stop at fn definitions that are NOT test fns
        if re.match(r"(pub\s+)?(async\s+)?fn\s+\w+", l):
            # Check if #[test] is on the preceding line (common Rust pattern)
            prev = lines[i - 1].strip() if i > 0 else ""
            if "#[test]" in prev or "#[cfg(test)]" in prev:
                return True
            # Not a direct test fn — but may be a helper inside a #[cfg(test)] mod.
            # Continue scanning backward: if we find #[cfg(test)] before another
            # fn boundary, this helper lives inside a test module.
            for j in range(i - 1, max(i - 60, -1), -1):
                lj = lines[j].strip()
                if "#[cfg(test)]" in lj or "#[test]" in lj:
                    return True
                # Another fn definition before finding a test marker → not test code
                if re.match(r"(pub\s+)?(async\s+)?fn\s+\w+", lj):
                    return False
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
                if _REGEX_INIT.search(raw):
                    continue  # Regex::new(...).unwrap() — invalid pattern is a developer error caught at startup
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
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-box-dyn-error",
                message="Box<dyn Error> — use a concrete error type (thiserror)",
            )
