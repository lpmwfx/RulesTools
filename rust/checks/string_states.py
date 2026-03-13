"""Rust magic string checks — from rust/types.md.

Strings used as discriminators (node types, operation kinds, tab IDs)
should be enums. Raw string comparisons and match arms scatter state
definitions across the codebase.

BANNED:
  - match expr { "foo" => ... }  — match on string literal (use enum)
  - if x == "foo"  where the string looks like an identifier/kind value

WARNING:
  - String literals ending in .json/.toml/.png used outside a const/static
    definition — centralise in a paths module (see rust/hardcoded_paths.py)

Exempt:
  - Test files and #[cfg(test)] blocks
  - Error messages and log strings (contain spaces or sentence structure)
  - Single-char strings
  - Lines that ARE const/static definitions
"""

from __future__ import annotations

from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "rust/types"

# match arm on a string literal:  "foo" =>
_MATCH_STR_ARM = re.compile(r'"([^"]{2,}?)"\s*=>')

# == "identifier-like"  or  != "identifier-like"
_EQ_STR = re.compile(r'[!=]=\s*"([^"]{2,}?)"')

# const / static definition line — these ARE the source of truth
_CONST_DEF = re.compile(r"^\s*(?:pub\s+)?(?:const|static)\s+")

# Strings that are obviously messages/paths/sentences (not identifiers)
_SKIP_VALUES = re.compile(
    r"(?:\s|[/\\.]|[A-Z]{2}|[?!,])"  # spaces, path chars, SCREAMING, punctuation
)


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

        # ── match "foo" => ──────────────────────────────────────────────────
        for m in _MATCH_STR_ARM.finditer(raw):
            val = m.group(1)
            if _SKIP_VALUES.search(val):
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-string-match",
                message=(
                    f'stringly-typed match "{val}" — type-safety paradigm (rust/types): '
                    f"define an enum variant instead of a raw string literal in match arms"
                ),
            )

        # ── == "identifier" ─────────────────────────────────────────────────
        for m in _EQ_STR.finditer(raw):
            val = m.group(1)
            if _SKIP_VALUES.search(val):
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-string-compare",
                message=(
                    f'stringly-typed comparison == "{val}" — type-safety paradigm (rust/types): '
                    f"discriminators must be enums or named consts, not raw string literals"
                ),
            )
