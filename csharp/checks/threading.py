"""C# threading checks — from csharp/threading.md.

BANNED:
  - .Result  on a Task  — deadlock risk
  - .Wait()  on a Task  — deadlock risk
  - .GetAwaiter().GetResult()  — deadlock risk
  - Thread.Sleep(...)  — use await Task.Delay
  - async void  (except event handlers — flagged as warning with guidance)
  - Task.Run(...)  without assignment — fire-and-forget
"""

from __future__ import annotations
import re
from pathlib import Path
from typing import Generator

from common.issue import Issue, Severity

_RULE_BASE = "csharp/threading"

_TASK_RESULT     = re.compile(r"\.\s*Result\b")
_TASK_WAIT       = re.compile(r"\.\s*Wait\s*\(")
_GET_RESULT      = re.compile(r"\.GetAwaiter\s*\(\s*\)\s*\.GetResult\s*\(")
_THREAD_SLEEP    = re.compile(r"\bThread\s*\.\s*Sleep\s*\(")
_ASYNC_VOID      = re.compile(r"\basync\s+void\s+\w")
# Fire-and-forget: Task.Run( not preceded by assignment (=, await, return)
_TASK_RUN_FF     = re.compile(r"(?<![=\w])Task\s*\.\s*Run\s*\(")


def _is_comment(raw: str) -> bool:
    stripped = raw.lstrip()
    return stripped.startswith("//") or stripped.startswith("*")


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    for lineno, raw in enumerate(lines, start=1):
        if _is_comment(raw):
            continue

        if m := _TASK_RESULT.search(raw):
            # Avoid false positive on .Result in LINQ result vars
            before = raw[: m.start()].rstrip()
            if before and before[-1] not in ('"', "'"):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-task-result",
                    message=(
                        ".Result on Task — blocks the thread and causes deadlocks "
                        "in async contexts. Use 'await' instead."
                    ),
                )

        if m := _TASK_WAIT.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-task-wait",
                message=(
                    ".Wait() on Task — blocks the thread and causes deadlocks. "
                    "Use 'await' instead."
                ),
            )

        if m := _GET_RESULT.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-getawaiter-getresult",
                message=(
                    ".GetAwaiter().GetResult() — synchronously blocks on async work. "
                    "Use 'await' instead."
                ),
            )

        if m := _THREAD_SLEEP.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-thread-sleep",
                message=(
                    "Thread.Sleep() — blocks the OS thread. "
                    "Use 'await Task.Delay(ms, ct)' instead."
                ),
            )

        if m := _ASYNC_VOID.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.WARNING,
                rule=f"{_RULE_BASE}/no-async-void",
                message=(
                    "async void — exceptions are unobservable and crash the process. "
                    "Use async Task. Exception: UI event handlers (document with a comment)."
                ),
            )

        if m := _TASK_RUN_FF.search(raw):
            stripped = raw.lstrip()
            # Allow:  var x = Task.Run /  await Task.Run /  return Task.Run /  _ = Task.Run
            if not re.search(r"(\bawait\b|=\s*Task\.Run|\breturn\b)", raw):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/no-fire-and-forget",
                    message=(
                        "Task.Run(...) without storing the task — fire-and-forget. "
                        "Assign to a field and cancel via CancellationToken on shutdown."
                    ),
                )
