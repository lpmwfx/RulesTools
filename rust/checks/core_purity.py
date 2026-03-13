"""Rust core-purity check — from core/design.md.

Core must import ZERO UI, IO, or platform code.

BANNED in src/core/:
  use slint / use gtk / use winit     — UI toolkits
  use std::fs / use tokio::fs         — filesystem IO
  use reqwest                          — network IO
  use tokio (general)                  — async runtime (WARNING — may be sync primitives)
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE = "core/design/no-platform-import"

# Patterns banned unconditionally in core/
_BANNED: list[tuple[re.Pattern, str]] = [
    (re.compile(r"^\s*use\s+slint\s*(?:::|;|\s)"),       "slint (UI toolkit)"),
    (re.compile(r"^\s*use\s+gtk\s*(?:::|;|\s)"),          "gtk (UI toolkit)"),
    (re.compile(r"^\s*use\s+winit\s*(?:::|;|\s)"),         "winit (UI toolkit)"),
    (re.compile(r"^\s*use\s+std\s*::\s*fs\s*::"),          "std::fs (filesystem IO)"),
    (re.compile(r"^\s*use\s+tokio\s*::\s*fs\s*::"),        "tokio::fs (async IO)"),
    (re.compile(r"^\s*use\s+reqwest\s*(?:::|;|\s)"),       "reqwest (HTTP)"),
    (re.compile(r"^\s*use\s+std\s*::\s*process\s*::"),     "std::process (subprocess)"),
]

# tokio in general is WARNING — sync primitives are ok, but async IO is not
_TOKIO_GENERAL = re.compile(r"^\s*use\s+tokio\s*(?:::|;|\s)")


def _is_in_core(path: Path) -> bool:
    return "core" in [p.lower() for p in path.parts]


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    if not _is_in_core(path):
        return

    # Skip test files
    parts_lower = [p.lower() for p in path.parts]
    if (
        "test" in parts_lower
        or "tests" in parts_lower
        or path.stem.endswith("_test")
        or path.stem.startswith("test_")
    ):
        return

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        for pattern, label in _BANNED:
            if pattern.match(raw):
                yield Issue(
                    file=path, line=lineno, col=1,
                    severity=Severity.ERROR,
                    rule=_RULE,
                    message=(
                        f"{label} in core/ - Core must be pure; "
                        f"move IO to Gateway, platform calls to PAL"
                    ),
                )
                break
        else:
            # tokio (general) — warning only, sync primitives may be ok
            if _TOKIO_GENERAL.match(raw):
                yield Issue(
                    file=path, line=lineno, col=1,
                    severity=Severity.ERROR,
                    rule=_RULE,
                    message=(
                        "tokio in core/ - use only tokio::sync primitives here; "
                        "async IO belongs in Gateway"
                    ),
                )
