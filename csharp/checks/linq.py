"""C# LINQ checks — from csharp/linq.md.

BANNED:
  - First()  without null guard — use FirstOrDefault()
  - Count() > 0  — use Any()
  - Select(...).Where(...)  — Where should come before Select
  - Side effects inside LINQ  (Select with { — lambda block body)
  - ToList() inside a loop  (heuristic: ToList() on a line inside for/foreach/while)
"""

from __future__ import annotations
import re
from pathlib import Path
from typing import Generator

from common.issue import Issue, Severity

_RULE_BASE = "csharp/linq"

_FIRST_BARE     = re.compile(r"\.\s*First\s*\(")            # .First(  — not FirstOrDefault
_COUNT_GT_ZERO  = re.compile(r"\.\s*Count\s*\(\s*\)\s*[>!]=?\s*0")
_SELECT_WHERE   = re.compile(r"\.\s*Select\s*\([^)]*\)\s*\.\s*Where\s*\(")
# Lambda block body inside Select:  .Select(x => { ...
_SELECT_SIDE_FX = re.compile(r"\.\s*Select\s*\(\s*\w+\s*=>\s*\{")

# Loop keywords — used to flag ToList() inside a loop
_LOOP_KW        = re.compile(r"\b(for|foreach|while)\b")
_TO_LIST        = re.compile(r"\.\s*ToList\s*\(\s*\)")


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    loop_depth = 0
    brace_depth = 0

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//") or stripped.startswith("*"):
            continue

        # Track whether we're inside a loop (simple brace counting)
        if _LOOP_KW.search(raw):
            loop_depth = brace_depth + 1  # flag brace depth we enter at
        brace_depth += raw.count("{") - raw.count("}")
        brace_depth = max(brace_depth, 0)
        inside_loop = brace_depth >= loop_depth > 0

        # --- First() without OrDefault ---
        for m in _FIRST_BARE.finditer(raw):
            # Make sure it's not FirstOrDefault
            after = raw[m.end():]
            before = raw[:m.start()]
            name_end = raw[m.start():].split("(")[0]
            if "OrDefault" not in raw[m.start(): m.start() + 20]:
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/use-first-or-default",
                    message=(
                        ".First() throws if sequence is empty — "
                        "use .FirstOrDefault() and handle the null case"
                    ),
                )

        # --- Count() > 0 ---
        if m := _COUNT_GT_ZERO.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/use-any",
                message=(
                    ".Count() > 0 — use .Any() for existence checks "
                    "(avoids full enumeration)"
                ),
            )

        # --- Select before Where ---
        if m := _SELECT_WHERE.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/where-before-select",
                message=(
                    ".Select(...).Where(...) — filter before projecting: "
                    ".Where(...).Select(...) to avoid transforming elements that get discarded"
                ),
            )

        # --- Side effects inside Select ---
        if m := _SELECT_SIDE_FX.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-select-side-effects",
                message=(
                    ".Select(x => { ... }) — side effects inside LINQ expressions "
                    "make code unpredictable. Extract to a foreach loop."
                ),
            )

        # --- ToList() inside a loop ---
        if inside_loop and (m := _TO_LIST.search(raw)):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-tolist-in-loop",
                message=(
                    ".ToList() inside a loop — materialises the collection on every "
                    "iteration. Move outside the loop or keep as IEnumerable<T>."
                ),
            )
