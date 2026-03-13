"""Rust type-usage checks — from rust/types.md.

BANNED:
  - &Vec<T> as parameter  (use &[T] instead)
  - &String as parameter  (use &str instead)
  - println!/eprintln!    in library code (not main.rs / tests)
  - static mut            (global mutable state)
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "rust/types"

_AMP_VEC    = re.compile(r"&\s*Vec\s*<")
_AMP_STRING = re.compile(r"&\s*String\b")
_PRINTLN    = re.compile(r"\b(e?println)!\s*\(")
_STATIC_MUT = re.compile(r"\bstatic\s+mut\b")


def _is_fn_param(raw: str, match: re.Match) -> bool:
    """Heuristic: is the match inside a function signature line?"""
    before = raw[: match.start()]
    return "fn " in before or "(" in before


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    is_main = path.name == "main.rs"
    in_test = False

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        if "#[cfg(test)]" in raw or "#[test]" in raw:
            in_test = True
        # Leave test context at closing brace at depth 0 is tricky —
        # simple heuristic: exit test mode when we see a top-level fn
        if re.match(r"^pub\s+fn\s+|^fn\s+", stripped) and "test" not in stripped:
            in_test = False

        # &Vec<T> parameter
        for m in _AMP_VEC.finditer(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/use-slice",
                message="&Vec<T> parameter — use &[T] instead (more general, zero cost)",
            )

        # &String parameter
        for m in _AMP_STRING.finditer(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/use-str",
                message="&String parameter — use &str instead (accepts both String and &str)",
            )

        # println!/eprintln! in library code
        if not is_main and not in_test:
            for m in _PRINTLN.finditer(raw):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-println",
                    message=(
                        f"{m.group(1)}!() in library code — "
                        f"use tracing/log macros instead"
                    ),
                )

        # static mut
        for m in _STATIC_MUT.finditer(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-global-mut",
                message=(
                    "static mut — global mutable state is banned. "
                    "Use Arc<Mutex<T>> or thread_local! instead."
                ),
            )
