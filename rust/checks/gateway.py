"""Rust gateway layer checks — from gateway/io.md.

VITAL: Gateway is the ONLY layer that performs IO.
Files outside src/gateway/ must not use std::fs, reqwest, or similar IO crates.

BANNED outside gateway/:
  - std::fs::read / write / remove / create
  - reqwest::get / post / Client::new
  - std::process::Command
  - tokio::fs
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE = "gateway/io/layer-violation"

# IO patterns that belong ONLY in the gateway layer
_FS_DIRECT    = re.compile(r"\bstd\s*::\s*fs\s*::\s*(read|write|remove|create|copy|rename|metadata)")
_REQWEST      = re.compile(r"\breqwest\s*::\s*(get|post|put|delete|patch|Client)")
_TOKIO_FS     = re.compile(r"\btokio\s*::\s*fs\s*::")
_PROCESS_CMD  = re.compile(r"\bstd\s*::\s*process\s*::\s*Command")

_IO_PATTERNS: list[tuple[re.Pattern, str]] = [
    (_FS_DIRECT,   "std::fs IO"),
    (_REQWEST,     "reqwest HTTP"),
    (_TOKIO_FS,    "tokio::fs IO"),
    (_PROCESS_CMD, "std::process::Command"),
]


def _is_in_gateway(path: Path) -> bool:
    """True if the file lives inside a gateway/ directory."""
    parts = [p.lower() for p in path.parts]
    return "gateway" in parts


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    if _is_in_gateway(path):
        return  # IO is allowed here

    # Also skip PAL (Platform Abstraction Layer) — it wraps the actual IO
    parts_lower = [p.lower() for p in path.parts]
    if "pal" in parts_lower or "platform" in parts_lower:
        return

    # Skip test files
    if "test" in parts_lower or path.stem.endswith("_test") or path.stem == "tests":
        return

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        for pattern, label in _IO_PATTERNS:
            if m := pattern.search(raw):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=_RULE,
                    message=(
                        f"{label} outside gateway/ — "
                        f"only Gateway and PAL may perform IO; move this call there"
                    ),
                )
                break  # one IO violation per line
