"""JS module checks — from js/modules.md.

BANNED:
  - require() / module.exports  (must use ESM)
  - export let <name>           (mutable export)
  - _privateField convention    (use #privateField)
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "js/modules"

_REQUIRE        = re.compile(r"\brequire\s*\(")
_MODULE_EXPORTS = re.compile(r"\bmodule\.exports\b")
_EXPORT_LET     = re.compile(r"^export\s+let\s+(\w+)", re.MULTILINE)
_UNDERSCORE_PRIVATE = re.compile(r"\bthis\._(\w+)\b")


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        if m := _REQUIRE.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/esm-only",
                message="require() is CommonJS — use ESM import instead",
            )

        if m := _MODULE_EXPORTS.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/esm-only",
                message="module.exports is CommonJS — use export instead",
            )

        if m := _EXPORT_LET.match(stripped):
            yield Issue(
                file=path, line=lineno, col=raw.index("export") + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-mutable-export",
                message=(
                    f"'export let {m.group(1)}' — mutable exports are banned. "
                    f"Export a getter function or use a const."
                ),
            )

        for m in _UNDERSCORE_PRIVATE.finditer(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.WARNING,
                rule=f"{_RULE_BASE}/private-fields",
                message=(
                    f"this._{m.group(1)} — use ES2022 private field #{ m.group(1)} instead"
                ),
            )
