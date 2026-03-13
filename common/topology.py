"""Topology layer-suffix check — from global/topology.md.

Each type's suffix tag must match the folder it lives in:
  src/ui/       → _ui
  src/adapter/  → _adp
  src/core/     → _core
  src/pal/      → _pal
  src/gateway/  → _gtw
  src/shared/   → _x

_sta and _cfg are valid in any layer (state / config structs).
_test is valid in any layer for test helpers.
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE = "global/topology"

_FOLDER_TO_TAG: dict[str, str] = {
    "ui":      "ui",
    "adapter": "adp",
    "core":    "core",
    "pal":     "pal",
    "gateway": "gtw",
    "shared":  "x",
}

# Tags valid in any layer
_CROSS_LAYER_TAGS = {"sta", "cfg", "test", "x"}

# All known tags
_ALL_TAGS = set(_FOLDER_TO_TAG.values()) | _CROSS_LAYER_TAGS

# Type-definition patterns per file extension — group 1 = type name
_TYPE_RE: dict[str, re.Pattern] = {
    "rs": re.compile(
        r"^\s*(?:pub(?:\s*\(\w+\))?\s+)?(?:struct|enum|type|trait)\s+(\w+)"
    ),
    "py": re.compile(r"^\s*class\s+(\w+)"),
    "kt": re.compile(
        r"^\s*(?:(?:data|sealed|open|abstract|inner|enum)\s+)*"
        r"(?:class|interface|object)\s+(\w+)"
    ),
    "kts": re.compile(
        r"^\s*(?:(?:data|sealed|open|abstract|inner|enum)\s+)*"
        r"(?:class|interface|object)\s+(\w+)"
    ),
    "ts": re.compile(
        r"^\s*(?:export\s+)?(?:default\s+)?(?:interface|type|class|enum)\s+(\w+)"
    ),
    "tsx": re.compile(
        r"^\s*(?:export\s+)?(?:default\s+)?(?:interface|type|class|enum)\s+(\w+)"
    ),
}

_SUFFIX_RE = re.compile(r"_([a-z][a-z0-9]*)$")

_COMMENT_STARTS = ("//", "#", "/*", "*", "--")


def _expected_tag(path: Path) -> str | None:
    """Return the expected layer tag from the file's folder, or None."""
    for part in path.parts:
        tag = _FOLDER_TO_TAG.get(part.lower())
        if tag is not None:
            return tag
    return None


def _type_tag(name: str) -> str | None:
    """Return the trailing layer tag if it is a known tag, else None."""
    m = _SUFFIX_RE.search(name)
    if m and m.group(1) in _ALL_TAGS:
        return m.group(1)
    return None


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    lang = path.suffix.lstrip(".")
    pattern = _TYPE_RE.get(lang)
    if pattern is None:
        return

    expected = _expected_tag(path)
    if expected is None:
        return  # Not in a recognised layer folder

    # Skip test files
    parts_lower = [p.lower() for p in path.parts]
    if (
        "test" in parts_lower
        or "tests" in parts_lower
        or path.stem.endswith("_test")
        or path.stem.startswith("test_")
        or path.stem == "tests"
    ):
        return

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith(_COMMENT_STARTS):
            continue

        m = pattern.match(raw)
        if not m:
            continue

        type_name = m.group(1)

        # Skip types whose name is clearly a test helper
        lower_name = type_name.lower()
        if lower_name.startswith("test") or lower_name.endswith("test"):
            continue

        tag = _type_tag(type_name)

        if tag is None:
            yield Issue(
                file=path, line=lineno, col=m.start(1) + 1,
                severity=Severity.ERROR,
                rule=_RULE,
                message=(
                    f"'{type_name}' in {expected}/ has no layer suffix "
                    f"-- append '_{expected}' "
                    f"(or '_sta' / '_cfg' for state/config types)"
                ),
            )
        elif tag not in _CROSS_LAYER_TAGS and tag != expected:
            yield Issue(
                file=path, line=lineno, col=m.start(1) + 1,
                severity=Severity.ERROR,
                rule=_RULE,
                message=(
                    f"'{type_name}' has suffix '_{tag}' but lives in {expected}/ "
                    f"— rename to end with '_{expected}' or move the file"
                ),
            )
