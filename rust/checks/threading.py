"""Rust threading checks — from rust/threading.md + rust/ownership.md.

BANNED:
  - tokio::spawn() / thread::spawn() result discarded (fire-and-forget)
  - Arc/Rc without documenting why (heuristic: no comment on same or previous line)
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "rust/threading"

# tokio::spawn(...) or std::thread::spawn(...) where return value is NOT bound
# Pattern: line starts with expression (not let _ = / let handle =)
_SPAWN = re.compile(r"\b(tokio::spawn|thread::spawn)\s*\(")
_LET_BIND = re.compile(r"^\s*(let\s+\w+|let\s+_)\s*=")

# Rc::new / Arc::new without a comment explaining why
_RC_ARC = re.compile(r"\b(Rc|Arc)::new\s*\(")


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        # --- Fire-and-forget spawn ---
        for m in _SPAWN.finditer(raw):
            # If the line binds the result (let handle = / let _ =) it's intentional
            if _LET_BIND.match(raw):
                continue
            # If result is used in an expression context (e.g. handles.push(...))
            if "=" in raw[: m.start()]:
                continue
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-fire-and-forget",
                message=(
                    f"{m.group(1)}() result discarded — "
                    f"store the JoinHandle or use a structured shutdown mechanism"
                ),
            )

        # --- Arc/Rc without comment ---
        for m in _RC_ARC.finditer(raw):
            same_line_comment = "//" in raw[m.end():]
            prev_line_comment = lineno >= 2 and "//" in lines[lineno - 2]
            if not same_line_comment and not prev_line_comment:
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/arc-rc-comment",
                    message=(
                        f"{m.group(1)}::new() without explaining comment — "
                        f"document WHY shared ownership is needed here"
                    ),
                )
