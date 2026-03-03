"""Rust PAL isolation check — from pal/design.md.

Platform-specific conditional compilation belongs ONLY inside src/pal/.

BANNED outside src/pal/:
  #[cfg(target_os = ...)]       — OS-specific code
  #[cfg(target_arch = ...)]     — architecture-specific code
  #[cfg(target_family = ...)]   — platform-family code
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE = "pal/design/platform-cfg-outside-pal"

_CFG_PLATFORM = re.compile(
    r"#\s*\[\s*cfg\s*\(\s*(target_os|target_arch|target_family|target_env)\s*="
)


def _is_in_pal(path: Path) -> bool:
    return "pal" in [p.lower() for p in path.parts]


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    if _is_in_pal(path):
        return  # Platform cfg is allowed here

    # Also allow in main.rs / lib.rs (PAL construction at entry point)
    if path.stem in ("main", "lib", "build"):
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
        m = _CFG_PLATFORM.search(raw)
        if m:
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=_RULE,
                message=(
                    f"#[cfg({m.group(1)} = ...)] outside src/pal/ - "
                    f"platform-specific code belongs only in PAL implementations"
                ),
            )
