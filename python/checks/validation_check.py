"""Python validation checks — from python/validation.md.

BANNED at system boundaries:
  - json.loads() without nearby model_validate / BaseModel parse
  - requests/httpx response used without pydantic validation
  - bare dict as function parameter type (should be a typed model)
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "python/validation"

_JSON_LOADS    = re.compile(r"\bjson\.loads\s*\(")
# response.json() / response.text() — but not inside strings or docstrings
_RESP_JSON     = re.compile(r"\.(json|text)\s*\(\s*\)")

# Pydantic / beartype validation indicators
_PYDANTIC_VAL  = re.compile(
    r"\b(model_validate|model_validate_json|parse_obj|parse_raw"
    r"|BaseModel|TypeAdapter|from_orm)\b"
)

# Bare dict param: def foo(..., name: dict, ...) but NOT -> dict or dict[...]
_DICT_PARAM    = re.compile(r"\bdef\s+\w+\s*\([^)]*[,\s]\w+\s*:\s*dict\s*[,)]")


def _iter_code_lines(lines: list[str]):
    """Yield (lineno, raw) skipping comments and triple-quoted docstring content."""
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
        if in_triple:
            continue
        if stripped.startswith("#"):
            continue
        yield lineno, raw


def _in_string(raw: str, pos: int) -> bool:
    """Heuristic: check if position is inside a string literal."""
    before = raw[:pos]
    return bool(before.count('"') % 2 or before.count("'") % 2)


def _nearby(lines: list[str], lineno: int, pattern: re.Pattern, window: int = 6) -> bool:
    start = max(0, lineno - window - 1)
    end   = min(len(lines), lineno + window)
    return any(pattern.search(l) for l in lines[start:end])


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    for lineno, raw in _iter_code_lines(lines):

        # --- json.loads() without pydantic validation nearby ---
        if m := _JSON_LOADS.search(raw):
            if not _in_string(raw, m.start()) and not _nearby(lines, lineno, _PYDANTIC_VAL):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/schema-at-boundary",
                    message=(
                        "json.loads() without pydantic validation — "
                        "pipe through Model.model_validate() at this boundary"
                    ),
                )

        # --- response.json() without pydantic validation nearby ---
        if m := _RESP_JSON.search(raw):
            if not _in_string(raw, m.start()) and not _nearby(lines, lineno, _PYDANTIC_VAL, 8):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/schema-at-boundary",
                    message=(
                        "response.json() without pydantic validation — "
                        "validate with Model.model_validate(response.json())"
                    ),
                )

        # --- bare dict as function parameter type ---
        if m := _DICT_PARAM.search(raw):
            if not _in_string(raw, m.start()):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/no-raw-dict-boundary",
                    message=(
                        "bare dict parameter — define a pydantic BaseModel "
                        "or TypedDict and validate at the boundary instead"
                    ),
                )
