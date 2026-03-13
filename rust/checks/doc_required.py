"""Enforce /// doc comments on public Rust items — rust/docs.md

REQUIRED on:
  pub fn, pub struct, pub enum, pub trait, pub type, pub mod, pub const/static
  (including pub(crate) and pub(super))

EXEMPT:
  - test files (path contains /tests/ or /test/)
  - pub use re-exports
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Generator

from rulestools.issue import Issue, Severity

_RULE = "rust/docs/doc-required"

_PUB_ITEM = re.compile(
    r"^\s*pub(?:\([^)]+\))?\s+(?:async\s+)?(?:fn|struct|enum|trait|type|mod|const|static)\s+(\w+)"
)
_ATTRIBUTE = re.compile(r"^\s*#\[")
_PUB_USE   = re.compile(r"^\s*pub\s+use\b")


def _is_test_file(path: Path) -> bool:
    parts = path.parts
    return "tests" in parts or "test" in parts


def _has_doc_comment(lines: list[str], item_idx: int) -> bool:
    """Walk backwards skipping attributes; return True if /// is found."""
    i = item_idx
    while i > 0:
        i -= 1
        trimmed = lines[i].strip()
        if not trimmed:
            return False
        if trimmed.startswith("///"):
            return True
        if _ATTRIBUTE.match(lines[i]):
            continue  # skip attribute lines
        return False
    return False


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    if _is_test_file(path):
        return

    for idx, line in enumerate(lines):
        if _PUB_USE.match(line):
            continue
        m = _PUB_ITEM.match(line)
        if not m:
            continue
        name = m.group(1)
        if not _has_doc_comment(lines, idx):
            yield Issue(
                file=path,
                line=idx + 1,
                col=1,
                severity=Severity.ERROR,
                rule=_RULE,
                message=f"pub item `{name}` is missing a `///` doc comment — add one above the declaration",
            )
