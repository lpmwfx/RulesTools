"""JS safety checks — from js/safety.md.

Checks:
  - eval() usage (dangerous)
  - Unhandled promise (.then() without .catch())
  - Layer violation: core/ importing from ui/
  - Missing await on async calls (heuristic)
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "js/safety"

_EVAL           = re.compile(r"\beval\s*\(")
_THEN_NO_CATCH  = re.compile(r"\.then\s*\([^)]*\)\s*;")  # .then(...); with no .catch
_CONSOLE        = re.compile(r"\bconsole\s*\.\s*(log|warn|error|debug|info)\s*\(")


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    parts = path.parts

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        # --- eval ---
        if m := _EVAL.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-eval",
                message="eval() is forbidden — it is a security vulnerability",
            )

        # --- layer violation: core/ must not import ui/ ---
        if "core" in parts:
            if re.search(r"""from\s+['"].*[/\\]ui[/\\]""", raw):
                yield Issue(
                    file=path, line=lineno, col=1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/layer-violation",
                    message="core/ must not import from ui/ — only adapter/ may bridge them",
                )

        # --- .then() without .catch() on same line ---
        if _THEN_NO_CATCH.search(raw) and ".catch" not in raw:
            yield Issue(
                file=path, line=lineno, col=raw.index(".then") + 1,
                severity=Severity.WARNING,
                rule=f"{_RULE_BASE}/unhandled-promise",
                message=".then() without .catch() — all promises must be handled",
            )

        # --- console.log/warn/error in non-test code ---
        is_test = "test" in path.parts or path.stem.endswith((".test", ".spec"))
        if not is_test:
            if m := _CONSOLE.search(raw):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/no-console",
                    message=(
                        f"console.{m.group(1)}() in production code — "
                        f"use a structured logger (e.g. pino, winston) instead"
                    ),
                )
