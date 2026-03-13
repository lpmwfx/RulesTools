"""Rust coupling checks — module boundary violations.

BANNED:
  - `use super::sibling` in non-mod.rs files
    (siblings should communicate through the folder's mod.rs interface,
     not import each other directly via super::)

  - `pub(super)` in leaf files
    (suggests a parent–child implementation relationship instead of
     clean encapsulated modules)

RULE: mod.rs exports the public interface of a folder.
RULE: Sibling files must not know about each other — only mod.rs may re-export.
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "rust/modules"

# use super::something  (in a file that is NOT mod.rs)
# catches:
#   use super::helpers::foo;
#   use super::{foo, bar};
_SUPER_IMPORT = re.compile(r"^\s*use\s+super\s*::\s*(\w+)", re.MULTILINE)

# pub(super) fn / struct / enum / type
_PUB_SUPER = re.compile(r"\bpub\s*\(\s*super\s*\)")


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    is_mod = path.name in ("mod.rs", "lib.rs")

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        # ── 1. use super::sibling in non-mod files ───────────────────────
        if not is_mod:
            if m := _SUPER_IMPORT.match(raw):
                sibling = m.group(1)
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-sibling-coupling",
                    message=(
                        f"'use super::{sibling}' — sibling import couples this file "
                        f"to its neighbour. Re-export '{sibling}' from mod.rs and "
                        f"import via 'crate::...' instead."
                    ),
                )

        # ── 2. pub(super) outside mod.rs ─────────────────────────────────
        if not is_mod:
            for m in _PUB_SUPER.finditer(raw):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-parent-child-visibility",
                    message=(
                        "pub(super) in a leaf file — suggests parent–child coupling. "
                        "Use pub(crate) and encapsulate behind mod.rs interface instead."
                    ),
                )
