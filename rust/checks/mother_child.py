"""Rust mother-child architecture checks.

Rules (from uiux/mother-child.md + global/module-tree.md):

  VITAL: Mother files (mod.rs, main.rs, lib.rs) are compositors — they wire
         child modules together, they do not contain business logic functions.
  VITAL: Child files are stateless — they receive state as parameters,
         they never create module-level mutable state.
  BANNED: Mother file with many fn definitions (extract to child files)
  BANNED: Child file with static/lazy_static/thread_local/OnceLock (state ownership)
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "uiux/mother-child"

# ── Mother detection ────────────────────────────────────────────────────────

_MOTHER_FILENAMES = {"mod.rs", "main.rs", "lib.rs"}

# ── Function definition (not inside a comment) ─────────────────────────────
# Matches: fn foo(  pub fn foo(  pub(crate) fn foo(  async fn foo(
_FN_DEF = re.compile(
    r"^\s*(?:pub(?:\([^)]+\))?\s+)?(?:async\s+)?fn\s+\w+\s*[<(]"
)

# mod declarations — these are wiring, not logic
_MOD_DECL = re.compile(r"^\s*(?:pub(?:\([^)]+\))?\s+)?mod\s+\w+\s*;")

# use/import — wiring
_USE_DECL = re.compile(r"^\s*(?:pub\s+)?use\s+")

# ── State ownership patterns in child files ─────────────────────────────────

# static / static mut
_STATIC = re.compile(r"^\s*(?:pub(?:\([^)]+\))?\s+)?static\s+(?:mut\s+)?[A-Z_]")

# lazy_static! { or lazy_static::lazy_static! {
_LAZY_STATIC = re.compile(r"\blazy_static\s*!")

# thread_local! {
_THREAD_LOCAL = re.compile(r"\bthread_local\s*!")

# OnceLock::new() / OnceCell::new()
_ONCE = re.compile(r"\b(?:OnceLock|OnceCell|Lazy)\s*::\s*new\s*\(")

# Thresholds for fn count in mother files
_FN_WARN = 3   # >3 functions in mother = warning
_FN_ERROR = 6  # >6 functions in mother = error


def _is_mother(path: Path) -> bool:
    return path.name in _MOTHER_FILENAMES


def _is_test_file(path: Path) -> bool:
    """Skip test files — they legitimately create state."""
    parts = path.parts
    return ("tests" in parts or "test" in parts
            or path.stem.endswith("_test")
            or path.stem.startswith("test_"))


def _is_test_context(lines: list[str], lineno: int) -> bool:
    """Check if we're inside a #[test] or #[cfg(test)] block."""
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
    """Mother files should be compositors — few or no fn definitions."""
    fn_locations: list[tuple[int, str]] = []

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue
        if _is_test_context(lines, lineno):
            continue

        if _FN_DEF.match(raw):
            # Extract function name for the message
            name_match = re.search(r"\bfn\s+(\w+)", raw)
            name = name_match.group(1) if name_match else "?"
            fn_locations.append((lineno, name))

    count = len(fn_locations)

    if count > _FN_ERROR:
        first_ln = fn_locations[0][0]
        names = ", ".join(name for _, name in fn_locations[:5])
        extra = f" ... +{count - 5} more" if count > 5 else ""
        yield Issue(
            file=path, line=first_ln, col=1,
            severity=Severity.ERROR,
            rule=f"{_RULE_BASE}/mother-too-many-fns",
            message=(
                f"mother file has {count} fn definitions ({names}{extra}) — "
                f"extract logic functions to child files. "
                f"Mother should only compose and wire children."
            ),
        )
    elif count > _FN_WARN:
        first_ln = fn_locations[0][0]
        names = ", ".join(name for _, name in fn_locations)
        yield Issue(
            file=path, line=first_ln, col=1,
            severity=Severity.WARNING,
            rule=f"{_RULE_BASE}/mother-too-many-fns",
            message=(
                f"mother file has {count} fn definitions ({names}) — "
                f"consider extracting to child files. "
                f"Mother should primarily be a compositor."
            ),
        )


def _check_child(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    """Child files must not own module-level state."""

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue
        if _is_test_context(lines, lineno):
            continue

        # static / static mut
        if _STATIC.match(raw):
            yield Issue(
                file=path, line=lineno, col=1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/child-owns-state",
                message=(
                    "static variable in child file — children must be stateless. "
                    "Move state to the mother (mod.rs/main.rs) and pass it as "
                    "a parameter."
                ),
            )

        # lazy_static!
        if _LAZY_STATIC.search(raw):
            yield Issue(
                file=path, line=lineno, col=1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/child-owns-state",
                message=(
                    "lazy_static! in child file — children must be stateless. "
                    "Move state to the mother and pass it as a parameter."
                ),
            )

        # thread_local!
        if _THREAD_LOCAL.search(raw):
            yield Issue(
                file=path, line=lineno, col=1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/child-owns-state",
                message=(
                    "thread_local! in child file — children must be stateless. "
                    "Move state to the mother and pass it as a parameter."
                ),
            )

        # OnceLock::new() / OnceCell::new()
        if _ONCE.search(raw):
            yield Issue(
                file=path, line=lineno, col=1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/child-owns-state",
                message=(
                    "OnceLock/OnceCell in child file — children must be stateless. "
                    "Move state to the mother and pass it as a parameter."
                ),
            )
