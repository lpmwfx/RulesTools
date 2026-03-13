"""C# error handling checks — from csharp/errors.md.

BANNED:
  - catch (Exception ex) { }         — silent swallow
  - catch (Exception)  { }           — silent swallow (no variable)
  - throw new Exception(...)          — use typed subclass
  - catch block that only logs without rethrowing (heuristic)
  - wildcard arm `_ =>` that discards the value (global/error-flow.md)
  - catch block that only logs without rethrow/report/return
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

# Check 5 — wildcard arm that discards value
_WILDCARD_EMPTY = re.compile(r"\b_\s*=>\s*(?:\{\s*\}|default\b|null\b)")
# Switch context look-back
_SWITCH_RE = re.compile(r"\bswitch\b")

# Check 6 — catch-only-logs state machine patterns
_CATCH_OPEN  = re.compile(r"\bcatch\s*[\({]")
_LOG_ONLY    = re.compile(
    r"(_logger|logger|Logger|Console|Debug|Trace|Log)\s*[\.\(]"
    r"|\.Log(Warning|Error|Information|Critical|Debug|Trace)\s*\("
)
_RETHROW     = re.compile(r"\bthrow\b|\bsink\.Report\b|\breturn\s+Result\b|\breturn\s+Error\b")


def _is_test_context(lines: list[str], lineno: int) -> bool:
    for i in range(lineno - 2, max(lineno - 50, -1), -1):
        line = lines[i]
        if "[Test]" in line or "[Fact]" in line or "[Theory]" in line or "Assert." in line:
            return True
    return False


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:  # noqa: C901
    # State machine for check 6: catch-only-logs
    in_catch = False
    catch_brace_depth = 0
    catch_start_lineno = 0
    catch_has_log = False
    catch_has_action = False

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//") or stripped.startswith("*"):
            continue

        is_test = _is_test_context(lines, lineno)

        # --- throw new Exception(...) ---
        if m := _THROW_BASE.search(raw):
            if not is_test:
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
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
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-pokemon-catch",
                message=(
                    "catch (Exception) — catching all exceptions hides bugs. "
                    "Catch the specific exception type and add a named recovery action."
                ),
            )

        if m := _CATCH_NAKED.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-naked-catch",
                message=(
                    "catch { } without exception type — "
                    "catches everything including ThreadAbortException. "
                    "Specify the exception type."
                ),
            )

        # --- Check 5: wildcard arm that discards value ---
        if not is_test and _WILDCARD_EMPTY.search(raw):
            # Only flag when preceded by a switch context
            switch_found = any(
                _SWITCH_RE.search(lines[i])
                for i in range(max(0, lineno - 6), lineno - 1)
            )
            if switch_found:
                yield Issue(
                    file=path, line=lineno, col=1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-empty-wildcard",
                    message=(
                        "Wildcard `_ =>` arm that discards the value — "
                        "match exhaustively per global/error-flow.md. "
                        "Every error variant must have a named recovery action."
                    ),
                )

        # --- Check 6: catch block that only logs (state machine) ---
        if not in_catch and _CATCH_OPEN.search(raw):
            in_catch = True
            # Count braces only from the opening `{` of the catch block,
            # so that the preceding `}` (closing try) doesn't cancel it out.
            catch_open_pos = raw.rfind("{")
            if catch_open_pos >= 0:
                tail = raw[catch_open_pos:]
                catch_brace_depth = tail.count("{") - tail.count("}")
            else:
                catch_brace_depth = 0
            catch_start_lineno = lineno
            catch_has_log = bool(_LOG_ONLY.search(raw))
            catch_has_action = bool(_RETHROW.search(raw))
            continue

        if in_catch:
            catch_brace_depth += raw.count("{") - raw.count("}")
            if _LOG_ONLY.search(raw):
                catch_has_log = True
            if _RETHROW.search(raw):
                catch_has_action = True

            if catch_brace_depth <= 0:
                # Catch block closed
                if catch_has_log and not catch_has_action and not is_test:
                    yield Issue(
                        file=path, line=catch_start_lineno, col=1,
                        severity=Severity.ERROR,
                        rule=f"{_RULE_BASE}/log-without-rethrow",
                        message=(
                            "catch block only logs without rethrow/sink.Report/return Result — "
                            "logging alone is not error handling. "
                            "Add throw, return an error result, or report to error sink."
                        ),
                    )
                in_catch = False
                catch_has_log = False
                catch_has_action = False
