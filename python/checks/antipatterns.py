"""Python anti-pattern checks — from python/types.md + python/naming.md.

BANNED:
  - Mutable default arguments: def foo(items=[]) or def foo(data={})
  - `global` keyword — use dependency injection or module-level constants instead
  - `eval()` / `exec()` — dynamic code execution
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "python/types"

# def foo(name: list = [], name2={} ...) — mutable default
_MUTABLE_DEFAULT = re.compile(
    r"\bdef\s+\w+\s*\([^)]*=\s*(\[\s*\]|\{\s*\})",
)

# global x — global state
_GLOBAL_STMT = re.compile(r"^\s*global\s+\w")

# eval( / exec(
_EVAL_EXEC = re.compile(r"\b(eval|exec)\s*\(")


def _iter_code_lines(lines: list[str]):
    in_triple = False
    triple_char = ""
    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        for q in ('"""', "'''"):
            if raw.count(q) % 2 == 1:
                if in_triple and triple_char == q:
                    in_triple = False
                    triple_char = ""
                elif not in_triple:
                    in_triple = True
                    triple_char = q
                break
        if in_triple or stripped.startswith("#"):
            continue
        yield lineno, raw


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    is_test = "test" in path.parts or path.stem.startswith("test_")

    for lineno, raw in _iter_code_lines(lines):
        stripped = raw.lstrip()

        # --- Mutable default argument ---
        if m := _MUTABLE_DEFAULT.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-mutable-default",
                message=(
                    "mutable default argument — the same object is shared across "
                    "all calls; use 'None' and assign inside the function body"
                ),
            )

        # --- global keyword ---
        if _GLOBAL_STMT.match(raw):
            yield Issue(
                file=path, line=lineno, col=len(raw) - len(raw.lstrip()) + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-global-state",
                message=(
                    "'global' keyword introduces shared mutable state — "
                    "pass the value as a parameter or use a typed config object"
                ),
            )

        # --- eval() / exec() ---
        if not is_test:
            if m := _EVAL_EXEC.search(raw):
                before = raw[: m.start()]
                if not (before.count('"') % 2 or before.count("'") % 2):
                    yield Issue(
                        file=path, line=lineno, col=m.start() + 1,
                        severity=Severity.ERROR,
                        rule=f"{_RULE_BASE}/no-eval",
                        message=(
                            f"{m.group(1)}() executes arbitrary code — "
                            f"never use in production; use ast.literal_eval for data"
                        ),
                    )
