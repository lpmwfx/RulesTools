"""Kotlin coroutine/threading checks — from kotlin/coroutines.md.

BANNED:
  - Thread.sleep() — blocks the thread; use delay() in a coroutine
  - runBlocking in production code — blocks the calling thread
  - Dispatchers.Main for I/O or CPU work — must be IO/Default
  - launch without storing the Job (fire-and-forget coroutine)
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from common.issue import Issue, Severity

_RULE_BASE = "kotlin/coroutines"

_THREAD_SLEEP  = re.compile(r"\bThread\s*\.\s*sleep\s*\(")
_RUN_BLOCKING  = re.compile(r"\brunBlocking\s*[({]")
_MAIN_IO       = re.compile(r"Dispatchers\s*\.\s*Main\b")
_LAUNCH_NOVAR  = re.compile(r"(?<![=\w])launch\s*[({]")  # launch not assigned
_ASYNC_NOVAR   = re.compile(r"(?<![=\w])async\s*[({]")   # async not awaited


def _is_test(path: Path, lines: list[str], lineno: int) -> bool:
    if "test" in path.parts or path.stem.endswith("Test"):
        return True
    # Check for @Test annotation within 5 lines above
    start = max(0, lineno - 5)
    return any("@Test" in lines[i] for i in range(start, lineno - 1))


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        # --- Thread.sleep() ---
        if m := _THREAD_SLEEP.search(raw):
            if not _is_test(path, lines, lineno):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-thread-sleep",
                    message=(
                        "Thread.sleep() blocks the OS thread — "
                        "use 'delay()' inside a suspend function or coroutine"
                    ),
                )

        # --- runBlocking ---
        if m := _RUN_BLOCKING.search(raw):
            if not _is_test(path, lines, lineno):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-run-blocking",
                    message=(
                        "runBlocking blocks the calling thread in production — "
                        "only allowed in tests; use coroutineScope or structured concurrency"
                    ),
                )

        # --- Dispatchers.Main for logic (heuristic: .Main used outside UI layer) ---
        if _MAIN_IO.search(raw):
            path_lower = str(path).lower()
            if not any(x in path_lower for x in ("ui", "view", "screen", "fragment", "activity")):
                yield Issue(
                    file=path, line=lineno, col=raw.index("Main") + 1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/wrong-dispatcher",
                    message=(
                        "Dispatchers.Main outside UI layer — "
                        "use Dispatchers.IO for I/O, Dispatchers.Default for CPU work"
                    ),
                )

        # --- Fire-and-forget launch (not assigned to a Job) ---
        if m := _LAUNCH_NOVAR.search(raw):
            # If the line has no assignment before launch, it's fire-and-forget
            before = raw[: m.start()].strip()
            if not before or before.endswith("{"):
                if not _is_test(path, lines, lineno):
                    yield Issue(
                        file=path, line=lineno, col=m.start() + 1,
                        severity=Severity.WARNING,
                        rule=f"{_RULE_BASE}/untracked-coroutine",
                        message=(
                            "launch{} result not stored — untracked coroutine cannot "
                            "be cancelled; assign to a Job or use a supervised scope"
                        ),
                    )
