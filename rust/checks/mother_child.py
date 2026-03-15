"""Rust mother-child checks: mother-too-many-fns + child-owns-state.

Proc-macro exemption: fns inside #[tool_router]/#[async_trait] impl blocks are
exempt — the macro requires all methods in one impl; developer cannot split them.
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "uiux/mother-child"

_MOTHER_FILENAMES = {"mod.rs", "main.rs", "lib.rs"}

# fn foo(  pub fn foo(  pub(crate) fn foo(  async fn foo(
_FN_DEF = re.compile(
    r"^\s*(?:pub(?:\([^)]+\))?\s+)?(?:async\s+)?fn\s+\w+\s*[<(]"
)

# impl-level macros that bundle all methods in one block (compiler constraint)
_BUNDLED_IMPL_ATTR = re.compile(
    r"^\s*#\[(?:tool_router|async_trait|tonic::async_trait)\b"
)

# fn-level tool attributes (e.g. rmcp #[tool(...)])
_TOOL_FN_ATTR = re.compile(r"^\s*#\[tool[\s(,\]]")

_IMPL_LINE = re.compile(r"^\s*(?:pub\s+)?(?:unsafe\s+)?impl\b")

_STATIC = re.compile(r"^\s*(?:pub(?:\([^)]+\))?\s+)?static\s+mut\s+[A-Z_]")
_LAZY_STATIC = re.compile(r"\blazy_static\s*!")
_THREAD_LOCAL = re.compile(r"\bthread_local\s*!")
_ONCE = re.compile(r"\b(?:OnceLock|OnceCell|Lazy)\s*::\s*new\s*\(")

_FN_WARN = 3
_FN_ERROR = 6


def _is_mother(path: Path) -> bool:
    return path.name in _MOTHER_FILENAMES


def _macro_bundled_fn_lines(lines: list[str]) -> set[int]:
    """Return 1-based line numbers of fn defs inside proc-macro-bundled impl blocks."""
    exempt: set[int] = set()
    n = len(lines)
    i = 0
    while i < n:
        if not _BUNDLED_IMPL_ATTR.match(lines[i]):
            i += 1
            continue
        # Find impl keyword within next 3 lines
        impl_line = -1
        for j in range(i + 1, min(i + 4, n)):
            if _IMPL_LINE.match(lines[j]):
                impl_line = j
                break
        if impl_line == -1:
            i += 1
            continue
        # Brace-depth scan: collect fn lines inside this impl block
        depth = 0
        impl_closed_at = n
        for k in range(impl_line, n):
            for ch in lines[k]:
                if ch == '{':
                    depth += 1
                elif ch == '}':
                    depth -= 1
            if _FN_DEF.match(lines[k]) and depth > 0:
                exempt.add(k + 1)
            if depth == 0 and k > impl_line:
                impl_closed_at = k
                break
        i = impl_closed_at + 1
    return exempt


def _has_tool_fn_attr(lines: list[str], fn_lineno: int) -> bool:
    """Return True if fn is preceded by a #[tool...] attribute (up to 10 lines back)."""
    for i in range(fn_lineno - 2, max(fn_lineno - 11, -1), -1):
        if _TOOL_FN_ATTR.match(lines[i]):
            return True
        if re.match(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+", lines[i]):
            break
        if re.match(r"^\s*(?:pub\s+)?(?:impl|struct|enum|trait)\b", lines[i]):
            break
    return False


def _is_test_file(path: Path) -> bool:
    parts = path.parts
    return ("tests" in parts or "test" in parts
            or path.stem.endswith("_test")
            or path.stem.startswith("test_"))


def _is_test_context(lines: list[str], lineno: int) -> bool:
    for i in range(lineno - 1, max(lineno - 50, -1), -1):
        line = lines[i]
        if "#[test]" in line or "#[cfg(test)]" in line:
            return True
        if line.strip().startswith("mod tests") or line.strip().startswith("mod test"):
            return True
    return False


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    if _is_test_file(path):
        return
    if _is_mother(path):
        yield from _check_mother(path, lines)
    else:
        yield from _check_child(path, lines)


def _check_mother(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    """Count non-exempt fn definitions in mother files."""
    bundled = _macro_bundled_fn_lines(lines)
    fn_locations: list[tuple[int, str]] = []

    for lineno, raw in enumerate(lines, start=1):
        if raw.lstrip().startswith("//"):
            continue
        if _is_test_context(lines, lineno):
            continue
        if not _FN_DEF.match(raw):
            continue
        if lineno in bundled or _has_tool_fn_attr(lines, lineno):
            continue
        name_match = re.search(r"\bfn\s+(\w+)", raw)
        fn_locations.append((lineno, name_match.group(1) if name_match else "?"))

    count = len(fn_locations)
    if count <= _FN_WARN:
        return

    first_ln = fn_locations[0][0]
    names = ", ".join(name for _, name in fn_locations[:5])
    extra = f" ... +{count - 5} more" if count > 5 else ""

    if count > _FN_ERROR:
        msg = (f"mother file has {count} fn definitions ({names}{extra}) — "
               f"extract logic functions to child files. "
               f"Mother should only compose and wire children.")
    else:
        msg = (f"mother file has {count} fn definitions ({names}) — "
               f"consider extracting to child files. "
               f"Mother should primarily be a compositor.")

    yield Issue(
        file=path, line=first_ln, col=1,
        severity=Severity.ERROR,
        rule=f"{_RULE_BASE}/mother-too-many-fns",
        message=msg,
    )


def _check_child(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    """Child files must not own module-level state."""
    for lineno, raw in enumerate(lines, start=1):
        if raw.lstrip().startswith("//"):
            continue
        if _is_test_context(lines, lineno):
            continue

        if _STATIC.match(raw):
            yield Issue(file=path, line=lineno, col=1, severity=Severity.ERROR,
                        rule=f"{_RULE_BASE}/child-owns-state",
                        message=("static variable in child file — children must be stateless. "
                                 "Move state to the mother (mod.rs/main.rs) and pass it as a parameter."))

        if _LAZY_STATIC.search(raw):
            yield Issue(file=path, line=lineno, col=1, severity=Severity.ERROR,
                        rule=f"{_RULE_BASE}/child-owns-state",
                        message=("lazy_static! in child file — children must be stateless. "
                                 "Move state to the mother and pass it as a parameter."))

        if _THREAD_LOCAL.search(raw):
            yield Issue(file=path, line=lineno, col=1, severity=Severity.ERROR,
                        rule=f"{_RULE_BASE}/child-owns-state",
                        message=("thread_local! in child file — children must be stateless. "
                                 "Move state to the mother and pass it as a parameter."))

        if _ONCE.search(raw):
            yield Issue(file=path, line=lineno, col=1, severity=Severity.ERROR,
                        rule=f"{_RULE_BASE}/child-owns-state",
                        message=("OnceLock/OnceCell in child file — children must be stateless. "
                                 "Move state to the mother and pass it as a parameter."))
