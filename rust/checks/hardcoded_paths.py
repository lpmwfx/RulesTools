"""Rust hardcoded file path checks — from rust/constants.md.

File names and paths embedded as string literals are scattered across
the codebase with no central definition. A rename requires hunting
every occurrence.

BANNED:
  - String literals ending in .json/.toml/.yaml/.txt/.png/.svg/.wasm
    that are NOT on a const/static definition line

Exempt:
  - const / static definitions (these ARE the source of truth)
  - Test files and #[cfg(test)] blocks
  - Lines that are comments
"""

from __future__ import annotations

from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "rust/constants"

# String literal that looks like a filename with an extension
_FILE_STRING = re.compile(
    r'"([^"]*\.(?:json|toml|yaml|yml|txt|png|svg|wasm|ron))"',
    re.IGNORECASE,
)

# const / static definition — the source of truth, not a violation
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

        for m in _FILE_STRING.finditer(raw):
            filename = m.group(1)
            # Skip paths that look like full paths (contain / or \)
            # — those are test fixtures or doc examples
            if "/" in filename or "\\" in filename:
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-hardcoded-path",
                message=(
                    f'hardcoded path "{filename}" — config-driven paradigm (rust/types): '
                    f"all filenames must be named constants in paths module, "
                    f"e.g. const NAME: &str = \"{filename}\";"
                ),
            )
