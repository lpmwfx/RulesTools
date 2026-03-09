"""C# project file checks — csharp/verification.md + csharp/modules.md.

Checks:
  - <Nullable> not set to enable
  - <TreatWarningsAsErrors> not set to true

Both checks are skipped if Directory.Build.props in a parent directory (up to 3 levels)
already contains the relevant setting.
"""

from __future__ import annotations
import re
from pathlib import Path
from typing import Generator

from common.issue import Issue, Severity

_RULE_BASE = "csharp/project"

_NULLABLE_RE = re.compile(r"<Nullable>\s*enable\s*</Nullable>", re.IGNORECASE)
_WARNINGS_RE = re.compile(r"<TreatWarningsAsErrors>\s*true\s*</TreatWarningsAsErrors>", re.IGNORECASE)


def _build_props_content(start: Path, levels: int = 3) -> str:
    """Walk up the directory tree looking for Directory.Build.props and return its content."""
    current = start.parent
    for _ in range(levels):
        candidate = current / "Directory.Build.props"
        if candidate.exists():
            try:
                return candidate.read_text(encoding="utf-8", errors="replace")
            except OSError:
                return ""
        parent = current.parent
        if parent == current:
            break
        current = parent
    return ""


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    # Directory.Build.props is the central source — don't flag it for missing settings
    if path.name == "Directory.Build.props":
        return

    content = "\n".join(lines)
    props_content = _build_props_content(path)

    # Check 7 — Nullable not enabled
    nullable_covered = _NULLABLE_RE.search(content) or _NULLABLE_RE.search(props_content)
    if not nullable_covered:
        yield Issue(
            file=path, line=1, col=1,
            severity=Severity.ERROR,
            rule=f"{_RULE_BASE}/nullable-required",
            message=(
                "<Nullable>enable</Nullable> missing — "
                "add to .csproj or Directory.Build.props"
            ),
        )

    # Check 8 — TreatWarningsAsErrors not set
    warnings_covered = _WARNINGS_RE.search(content) or _WARNINGS_RE.search(props_content)
    if not warnings_covered:
        yield Issue(
            file=path, line=1, col=1,
            severity=Severity.WARNING,
            rule=f"{_RULE_BASE}/warnings-as-errors",
            message=(
                "<TreatWarningsAsErrors>true</TreatWarningsAsErrors> missing — "
                "add to .csproj or Directory.Build.props"
            ),
        )
