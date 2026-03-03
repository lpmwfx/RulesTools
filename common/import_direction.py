"""Import-direction check — from global/topology.md.

Two complementary checks:
1. Folder-path: detects layer folder name in import path
2. Type-suffix: detects _layertag identifier on import lines (language-agnostic)

BANNED imports (from the Forbidden Cross-Suffix Imports table):
  core/    -> adapter/, ui/, gateway/
  ui/      -> core/, pal/, gateway/
  pal/     -> core/, adapter/, ui/, gateway/
  gateway/ -> adapter/, ui/
"""

from __future__ import annotations
from pathlib import Path
from typing import Callable, Generator
import re

from common.issue import Issue, Severity

_RULE = "global/topology/import-direction"

_LAYER_NAMES = frozenset({"ui", "adapter", "core", "pal", "gateway", "shared"})

# Folder layer -> forbidden folder layers (from topology.md DAG)
_FORBIDDEN: dict[str, frozenset[str]] = {
    "core":    frozenset({"adapter", "ui", "gateway"}),
    "ui":      frozenset({"core", "pal", "gateway"}),
    "pal":     frozenset({"core", "adapter", "ui", "gateway"}),
    "gateway": frozenset({"adapter", "ui"}),
}

# Tag -> forbidden tags (mirrors the table in topology.md)
_TAG_FORBIDDEN: dict[str, frozenset[str]] = {
    "core": frozenset({"adp", "ui", "gtw"}),
    "ui":   frozenset({"core", "pal", "gtw"}),
    "pal":  frozenset({"core", "adp", "ui", "gtw"}),
    "gateway": frozenset({"adp", "ui"}),
}

# Folder name -> layer tag (for suffix-based check)
_FOLDER_TO_TAG: dict[str, str] = {
    "ui": "ui", "adapter": "adp", "core": "core",
    "pal": "pal", "gateway": "gtw", "shared": "x",
}

# Detects _tag suffix identifiers in import lines
_SUFFIX_RE = re.compile(r"\b\w+_(ui|adp|core|pal|gtw)\b")

# --- Language-specific import path extractors ---

_RUST_USE   = re.compile(r"^\s*use\s+crate\s*::\s*(\w+)")
_PY_FROM    = re.compile(r"^\s*from\s+\.{0,3}(\w+)")
_PY_IMPORT  = re.compile(r"^\s*import\s+(\w+)")
_KT_IMPORT  = re.compile(r"^\s*import\s+([\w.]+)")
_JS_FROM    = re.compile(r"""(?:^|\s)from\s+['"]([^'"]+)['"]""")
_JS_IMPORT  = re.compile(r"""^\s*import\s+['"]([^'"]+)['"]""")

_COMMENT_STARTS = ("//", "#", "/*", "*", "--")


def _segment_layer(raw_path: str) -> str | None:
    for seg in raw_path.replace("\\", "/").split("/"):
        if seg.lower() in _LAYER_NAMES:
            return seg.lower()
    return None


def _rust_layer(line: str) -> str | None:
    m = _RUST_USE.match(line)
    if m:
        name = m.group(1).lower()
        return name if name in _LAYER_NAMES else None
    return None


def _py_layer(line: str) -> str | None:
    for pat in (_PY_FROM, _PY_IMPORT):
        m = pat.match(line)
        if m:
            name = m.group(1).lower()
            return name if name in _LAYER_NAMES else None
    return None


def _kt_layer(line: str) -> str | None:
    m = _KT_IMPORT.match(line)
    if not m:
        return None
    for seg in m.group(1).split("."):
        if seg.lower() in _LAYER_NAMES:
            return seg.lower()
    return None


def _js_layer(line: str) -> str | None:
    for pat in (_JS_FROM, _JS_IMPORT):
        m = pat.search(line)
        if m:
            layer = _segment_layer(m.group(1))
            if layer:
                return layer
    return None


_FINDERS: dict[str, Callable[[str], str | None]] = {
    "rs":  _rust_layer,
    "py":  _py_layer,
    "kt":  _kt_layer,
    "kts": _kt_layer,
    "ts":  _js_layer,
    "tsx": _js_layer,
    "js":  _js_layer,
    "mjs": _js_layer,
}

_SUFFIX_LANGS = frozenset(_FINDERS)


def _file_layer(path: Path) -> str | None:
    for part in path.parts:
        if part.lower() in _LAYER_NAMES:
            return part.lower()
    return None


def _is_test_context(lines: list[str], lineno: int) -> bool:
    """Return True if line (1-based) appears to be inside a #[cfg(test)] block."""
    for i in range(lineno - 2, max(lineno - 60, -1), -1):
        l = lines[i].strip()
        if "#[cfg(test)]" in l or "#[test]" in l:
            return True
        # Stop at top-level definitions that are clearly not test helpers
        if re.match(r"^(pub\s+)?((?:async\s+)?fn|impl|struct|enum|type|trait)\s+\w+", l):
            if "test" not in l.lower():
                return False
    return False


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    file_layer = _file_layer(path)
    if file_layer is None:
        return

    forbidden_folders = _FORBIDDEN.get(file_layer)
    file_tag = _FOLDER_TO_TAG.get(file_layer)
    forbidden_tags = _TAG_FORBIDDEN.get(file_layer, frozenset())

    if not forbidden_folders and not forbidden_tags:
        return  # adapter and shared: no restrictions

    lang = path.suffix.lstrip(".")
    find_layer = _FINDERS.get(lang)

    # Skip test files
    parts_lower = [p.lower() for p in path.parts]
    if (
        "test" in parts_lower
        or "tests" in parts_lower
        or path.stem.endswith("_test")
        or path.stem.startswith("test_")
    ):
        return

    reported_lines: set[int] = set()

    for lineno, raw in enumerate(lines, start=1):
        if raw.lstrip().startswith(_COMMENT_STARTS):
            continue

        # Skip lines inside #[cfg(test)] / #[test] blocks
        if _is_test_context(lines, lineno):
            continue

        # Check 1: folder name in import path
        if find_layer and forbidden_folders:
            imported = find_layer(raw)
            if imported and imported in forbidden_folders:
                reported_lines.add(lineno)
                yield Issue(
                    file=path, line=lineno, col=1,
                    severity=Severity.ERROR,
                    rule=_RULE,
                    message=(
                        f"{file_layer}/ imports from {imported}/ - "
                        f"topology DAG violation "
                        f"(BANNED: {file_layer} -> {imported})"
                    ),
                )
                continue  # one violation per line

        # Check 2: type suffix in import line (catches cross-package imports)
        if lineno in reported_lines or not forbidden_tags:
            continue
        if not (
            "import" in raw
            or "use " in raw
            or "require" in raw
            or "from " in raw
        ):
            continue
        for m in _SUFFIX_RE.finditer(raw):
            imported_tag = m.group(1)
            if imported_tag in forbidden_tags:
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=_RULE,
                    message=(
                        f"'{m.group(0)}' has suffix '_{imported_tag}' - "
                        f"BANNED in {file_layer}/ "
                        f"(_{file_tag or file_layer} must not import _{imported_tag})"
                    ),
                )
                break  # one per line
