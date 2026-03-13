"""Slint architecture check — single gateway/bridge API layer.

The UI layer must speak to exactly ONE gateway object.
All callbacks must delegate to the same bridge (e.g. AppBridge).

If callbacks in different files call AppBridge.foo() AND AppState.bar()
AND SomeOther.baz(), the API layer is split — a clear architecture violation.

This is a TREE-LEVEL check: call check_tree(paths) once per scan,
not check(path, lines) per file.
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re
from collections import defaultdict

from common.issue import Issue, Severity

_RULE = "uiux/state-flow/single-gateway"

# Callback: name => { ... }  — grab the body
_CB_START    = re.compile(r"[\w-]+\s*(?:\([^)]*\))?\s*=>\s*\{")
# Bridge call inside callback: Identifier.method(  — PascalCase receiver
_BRIDGE_CALL = re.compile(r"\b([A-Z][A-Za-z0-9]*)\.([\w-]+)\s*\(")
# Ignore pure Slint built-ins
_BUILTIN_RECEIVERS = {"Math", "Colors", "Palette", "Theme", "StyleMetrics", "TextInputInterface"}


def _collect_bridge_calls(path: Path) -> list[tuple[int, str]]:
    """Return list of (lineno, ReceiverName) for all bridge calls in callbacks."""
    try:
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return []

    results: list[tuple[int, str]] = []
    in_callback = False
    depth = 0

    for lineno, raw in enumerate(lines, start=1):
        if not in_callback:
            if _CB_START.search(raw):
                in_callback = True
                depth = raw.count("{") - raw.count("}")
                if depth <= 0:
                    in_callback = False
                    continue
        else:
            depth += raw.count("{") - raw.count("}")
            if depth <= 0:
                in_callback = False
                continue

        if in_callback:
            for m in _BRIDGE_CALL.finditer(raw):
                receiver = m.group(1)
                if receiver not in _BUILTIN_RECEIVERS:
                    results.append((lineno, receiver))

    return results


def check_tree(paths: list[Path]) -> Generator[Issue, None, None]:
    """Scan all .slint files and flag if callbacks use multiple gateway objects."""
    # file -> list of (lineno, receiver)
    calls_by_file: dict[Path, list[tuple[int, str]]] = {}
    for path in paths:
        if path.suffix != ".slint":
            continue
        calls = _collect_bridge_calls(path)
        if calls:
            calls_by_file[path] = calls

    if not calls_by_file:
        return

    # Count unique receivers across ALL files
    all_receivers: dict[str, list[tuple[Path, int]]] = defaultdict(list)
    for path, calls in calls_by_file.items():
        for lineno, receiver in calls:
            all_receivers[receiver].append((path, lineno))

    # If only one receiver: clean architecture
    if len(all_receivers) <= 1:
        return

    # Multiple receivers: flag each file that calls a non-dominant receiver
    # Dominant = the one used most
    dominant = max(all_receivers, key=lambda r: len(all_receivers[r]))
    others = {r: locs for r, locs in all_receivers.items() if r != dominant}

    for receiver, locations in others.items():
        for path, lineno in locations:
            yield Issue(
                file=path, line=lineno, col=1,
                severity=Severity.WARNING,
                rule=_RULE,
                message=(
                    f"callback calls '{receiver}' but the gateway is '{dominant}' — "
                    f"all UI callbacks must delegate through the single gateway object"
                ),
            )
