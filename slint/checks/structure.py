"""Slint structural checks — from global/module-tree.md.

RULE: Every named Slint component in its own file.
BANNED: Multiple independent component definitions in one .slint file.

Exception: small helper structs/enums defined alongside a primary component
are allowed — flag only when there are 2+ *export component* definitions.
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE = "global/module-tree/one-component-per-file"

# Matches:  export component Foo  or  component Foo
_COMPONENT_DEF = re.compile(r"^\s*(?:export\s+)?component\s+(\w+)", re.MULTILINE)


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    text = "\n".join(lines)
    matches = list(_COMPONENT_DEF.finditer(text))

    if len(matches) < 2:
        return

    # Find line numbers for each extra component definition
    line_offsets = [0]
    for line in lines:
        line_offsets.append(line_offsets[-1] + len(line) + 1)

    def pos_to_line(pos: int) -> int:
        for i, offset in enumerate(line_offsets):
            if offset > pos:
                return i
        return len(lines)

    names = [m.group(1) for m in matches]
    # First component is "primary" — report all subsequent ones
    for m in matches[1:]:
        lineno = pos_to_line(m.start())
        yield Issue(
            file=path, line=lineno, col=1,
            severity=Severity.ERROR,
            rule=_RULE,
            message=(
                f"component '{m.group(1)}' — multiple components in one file. "
                f"Extract to '{m.group(1)}.slint' "
                f"(primary: '{names[0]}')"
            ),
        )
