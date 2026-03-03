"""Slint event/callback architecture checks.

Rules (from uiux/state-flow.md + js/safety.md layer rules):

  RULE: UI callbacks delegate to gateway — one call per callback
  RULE: State mutations belong in the gateway layer, not in UI callbacks
  RULE: Conditional logic in callbacks must go to the backend
  BANNED: if/else inside a callback body
  BANNED: Multiple root.x = assignments in one callback (use a single bridge call)
  BANNED: Callback body longer than 3 meaningful lines

Pattern the scanner detects — each callback body is extracted and analysed:

  callback-name(args) => {   ← body start
      ... body lines ...      ← extracted
  }                           ← body end (brace depth back to 0)
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "uiux/state-flow"

# Callback definition: identifier (optional args) => {
_CB_START = re.compile(r"([\w-]+)\s*(?:\([^)]*\))?\s*=>\s*\{")

# Patterns inside a callback body
_IF_STMT      = re.compile(r"\bif\b")
_ROOT_ASSIGN  = re.compile(r"\broot\.([\w-]+)\s*=(?!=)")   # root.x = ... (not ==)
_BRIDGE_CALL  = re.compile(r"\w+\.\w+\s*\(")               # Something.method(


def _extract_callbacks(lines: list[str]) -> list[tuple[int, str, list[tuple[int, str]]]]:
    """
    Parse lines and return list of (start_lineno, cb_name, body_lines).
    body_lines is list of (lineno, raw_line) inside the callback braces.
    Single-line callbacks  `name => { stmt; }`  are included with one body line.
    """
    results = []
    i = 0
    while i < len(lines):
        raw = lines[i]
        m = _CB_START.search(raw)
        if m:
            cb_name   = m.group(1)
            start_ln  = i + 1          # 1-indexed
            body: list[tuple[int, str]] = []

            # Count net braces after the opening { on this line
            after_open = raw[m.end():]            # text after the {
            depth = 1 + after_open.count("{") - after_open.count("}")

            if depth <= 0:
                # Entire callback is on one line
                body = [(i + 1, raw)]
                results.append((start_ln, cb_name, body))
                i += 1
                continue

            # Multi-line callback — collect until depth returns to 0
            i += 1
            while i < len(lines) and depth > 0:
                line = lines[i]
                depth += line.count("{") - line.count("}")
                if depth > 0:
                    body.append((i + 1, line))
                i += 1
            results.append((start_ln, cb_name, body))
            continue
        i += 1
    return results


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    for start_ln, cb_name, body in _extract_callbacks(lines):

        # Skip trivially empty or single-statement callbacks
        meaningful = [
            (ln, l) for ln, l in body
            if l.strip() and not l.strip().startswith("//")
        ]

        # ── 1. Conditional logic in callback ────────────────────────────────
        if_lines = [(ln, l) for ln, l in meaningful if _IF_STMT.search(l)]
        if if_lines:
            first_ln, first_l = if_lines[0]
            yield Issue(
                file=path, line=first_ln, col=first_l.index("if") + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-callback-logic",
                message=(
                    f"callback '{cb_name}': if-statement in UI callback — "
                    f"move conditional logic to a single AppBridge method"
                ),
            )

        # ── 2. Multiple root.x = assignments ────────────────────────────────
        root_assigns = [(ln, l, _ROOT_ASSIGN.findall(l)) for ln, l in meaningful
                        if _ROOT_ASSIGN.search(l)]
        all_props = [prop for _, _, props in root_assigns for prop in props]
        if len(all_props) >= 2:
            first_ln = root_assigns[0][0]
            yield Issue(
                file=path, line=first_ln, col=1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-state-mutation-in-callback",
                message=(
                    f"callback '{cb_name}': {len(all_props)} root state mutations "
                    f"({', '.join(all_props[:3])}{' ...' if len(all_props) > 3 else ''}) — "
                    f"replace with a single AppBridge call that owns this state"
                ),
            )
        elif len(all_props) == 1:
            # Single root.x = is a warning if there's also a bridge call
            # (mixing UI state + bridge = split responsibility)
            has_bridge = any(_BRIDGE_CALL.search(l) for _, l in meaningful)
            if has_bridge:
                first_ln = root_assigns[0][0]
                yield Issue(
                    file=path, line=first_ln, col=1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/no-state-mutation-in-callback",
                    message=(
                        f"callback '{cb_name}': mixes root.{all_props[0]} mutation "
                        f"with a bridge call — let the bridge own all state changes"
                    ),
                )

        # ── 3. Callback body too long (no if, no multi-assign — just verbose) ──
        if len(meaningful) > 3 and not if_lines and len(all_props) < 2:
            yield Issue(
                file=path, line=start_ln, col=1,
                severity=Severity.WARNING,
                rule=f"{_RULE_BASE}/no-callback-logic",
                message=(
                    f"callback '{cb_name}': {len(meaningful)}-line body — "
                    f"callbacks should be a single delegation call"
                ),
            )
