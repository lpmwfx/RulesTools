"""Python type-safety checks — from python/types.md.

BANNED:
  - Missing `from __future__ import annotations` in files with type hints
  - `Optional[X]` — use `X | None` instead (modern union syntax)
  - Bare `except:` without exception type
  - `print()` in non-script files (use logging)
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "python/types"

_FUTURE_IMPORT = re.compile(r"from\s+__future__\s+import\s+annotations")
_TYPE_HINT     = re.compile(r":\s*\w|->")          # presence of type hints
_OPTIONAL      = re.compile(r"\bOptional\s*\[")
_BARE_EXCEPT   = re.compile(r"^\s*except\s*:\s*$")
_PRINT_CALL    = re.compile(r"\bprint\s*\(")


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    text = "\n".join(lines)
    is_script = path.name in {"__main__.py", "cli.py", "main.py"} or path.stem == "manage"

    # --- Missing future annotations ---
    has_future = bool(_FUTURE_IMPORT.search(text))
    has_hints  = bool(_TYPE_HINT.search(text))
    if not has_future and has_hints:
        yield Issue(
            file=path, line=1, col=1,
            severity=Severity.WARNING,
            rule=f"{_RULE_BASE}/future-annotations",
            message=(
                "missing 'from __future__ import annotations' — "
                "required in all files that use type hints"
            ),
        )

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("#"):
            continue

        # --- Optional[X] instead of X | None ---
        for m in _OPTIONAL.finditer(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.WARNING,
                rule=f"{_RULE_BASE}/modern-union",
                message="Optional[X] — use 'X | None' (PEP 604 modern syntax)",
            )

        # --- Bare except: ---
        if _BARE_EXCEPT.match(raw):
            yield Issue(
                file=path, line=lineno, col=raw.index("except") + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-bare-except",
                message=(
                    "bare 'except:' catches everything including KeyboardInterrupt — "
                    "specify exception type: 'except ValueError:'"
                ),
            )

        # --- print() in non-script files ---
        if not is_script:
            for m in _PRINT_CALL.finditer(raw):
                # Skip if inside a string (simplified check)
                before = raw[: m.start()]
                if before.count('"') % 2 or before.count("'") % 2:
                    continue
                if stripped.startswith("#"):
                    continue
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/no-print",
                    message="print() in library code — use logging or structlog instead",
                )
