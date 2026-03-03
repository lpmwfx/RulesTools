"""Rust adapter layer checks — from adapter/event-flow.md.

The Adapter layer registers UI event listeners and pushes complete state.

BANNED:
  - ui.get_*() inside event handler closures (state must live in Adapter, not read from UI)
  - ui.on_*() registered outside an init() or setup() function (lazy/conditional registration)
  - Business logic in event handlers (if/match before the core dispatch call)
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "adapter/event-flow"

# ui.get_XXX() — reading state back from the UI widget
_UI_GET     = re.compile(r"\bui\s*\.\s*(get_|as_weak\(\).*get_)\w+\s*\(")

# ui.on_XXX( registrations
_UI_ON      = re.compile(r"\bui\s*\.\s*on_(\w+)\s*\(")

# Function definitions — to detect if we're inside init/setup
_FN_DEF     = re.compile(r"^\s*(pub\s+)?(async\s+)?fn\s+(\w+)")


def _is_init_fn(lines: list[str], lineno: int) -> bool:
    """Check if the on_* registration is inside an init/setup function."""
    for i in range(lineno - 2, max(lineno - 80, -1), -1):
        m = _FN_DEF.match(lines[i])
        if m:
            fn_name = m.group(3).lower()
            return any(kw in fn_name for kw in ("init", "setup", "new", "build", "wire"))
    return False


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    # Only check adapter layer files
    parts_lower = [p.lower() for p in path.parts]
    if "adapter" not in parts_lower:
        return

    # Skip test files
    if "test" in parts_lower or path.stem.endswith("_test"):
        return

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        # --- ui.get_*() inside event handler — reading UI state ---
        if m := _UI_GET.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.WARNING,
                rule=f"{_RULE_BASE}/no-ui-read-in-handler",
                message=(
                    "ui.get_*() reads state from the widget — "
                    "Adapter owns state in AdapterState_sta; read from there, not from ui"
                ),
            )

        # --- ui.on_*() registered outside init/setup ---
        if m := _UI_ON.search(raw):
            if not _is_init_fn(lines, lineno):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/register-in-init",
                    message=(
                        f"ui.on_{m.group(1)}() registered outside init() — "
                        f"all event listeners must be registered in Adapter::init() "
                        f"before ui.run()"
                    ),
                )
