"""C# error handling checks — from csharp/errors.md.

BANNED:
  - catch (Exception ex) { }         — silent swallow
  - catch (Exception)  { }           — silent swallow (no variable)
  - throw new Exception(...)          — use typed subclass
  - catch block that only logs without rethrowing (heuristic)
"""

from __future__ import annotations
import re
from pathlib import Path
from typing import Generator

from common.issue import Issue, Severity

_RULE_BASE = "csharp/errors"

# catch (Exception ...) — catches everything
_CATCH_BARE   = re.compile(r"\bcatch\s*\(\s*Exception\b")
# catch { } — no type at all (C# allows this)
_CATCH_NAKED  = re.compile(r"\bcatch\s*\{")
# throw new Exception( — not a subclass
_THROW_BASE   = re.compile(r"\bthrow\s+new\s+Exception\s*\(")


def _is_test_context(lines: list[str], lineno: int) -> bool:
    for i in range(lineno - 2, max(lineno - 50, -1), -1):
        line = lines[i]
        if "[Test]" in line or "[Fact]" in line or "[Theory]" in line or "Assert." in line:
            return True
    return False


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    # Collect catch blocks: track if they contain only closing brace (empty body)
    in_catch_empty = False
    catch_brace_depth = 0
    catch_lineno = 0
    catch_rule = ""
    catch_msg = ""

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//") or stripped.startswith("*"):
            continue

        if _is_test_context(lines, lineno):
            continue

        # --- throw new Exception(...) ---
        if m := _THROW_BASE.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.WARNING,
                rule=f"{_RULE_BASE}/no-base-exception",
                message=(
                    "throw new Exception() — use a typed exception subclass "
                    "(e.g. InvalidOperationException, StorageException) "
                    "to allow callers to catch selectively"
                ),
            )

        # --- catch (Exception ...) or catch { ---
        if m := _CATCH_BARE.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.WARNING,
                rule=f"{_RULE_BASE}/no-pokemon-catch",
                message=(
                    "catch (Exception) — catching all exceptions hides bugs. "
                    "Catch the specific exception type and add a named recovery action."
                ),
            )

        if m := _CATCH_NAKED.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.WARNING,
                rule=f"{_RULE_BASE}/no-naked-catch",
                message=(
                    "catch { } without exception type — "
                    "catches everything including ThreadAbortException. "
                    "Specify the exception type."
                ),
            )
