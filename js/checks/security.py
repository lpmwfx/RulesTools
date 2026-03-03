"""JS/TS security checks — js/security.md + global/error-flow.md.

Checks:
  - .innerHTML = / .outerHTML =   — DOM injection / XSS
  - dangerouslySetInnerHTML        — React XSS
  - new Function(...)              — code injection
  - Empty catch block              — silent error swallow
"""

from __future__ import annotations
import re
from pathlib import Path
from typing import Generator

from common.issue import Issue, Severity

_RULE_BASE = "js/security"

_INNER_HTML       = re.compile(r"\.(innerHTML|outerHTML)\s*=(?!=)")
_DANGEROUS_HTML   = re.compile(r"dangerouslySetInnerHTML")
_NEW_FUNCTION     = re.compile(r"\bnew\s+Function\s*\(")
# Single-line empty catch:  catch (e) { }  or  catch { }
_EMPTY_CATCH_1    = re.compile(r"\bcatch\s*(?:\([^)]*\))?\s*\{\s*\}")


def _is_test(path: Path) -> bool:
    return (
        "test" in path.parts
        or "__tests__" in path.parts
        or path.stem.endswith((".test", ".spec"))
    )


def _check_empty_catch_multiline(
    path: Path,
    lines: list[str],
) -> Generator[Issue, None, None]:
    """Detect two-line empty catch:  } catch (e) {\n  }"""
    i = 0
    while i < len(lines):
        raw = lines[i]
        # Line ends a catch opening: catch (...) {
        if re.search(r"\bcatch\s*(?:\([^)]*\))?\s*\{", raw):
            # Look at the next non-blank line
            j = i + 1
            while j < len(lines) and not lines[j].strip():
                j += 1
            if j < len(lines) and lines[j].strip() in ("}", "},"):
                yield Issue(
                    file=path, line=i + 1, col=raw.index("catch") + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-empty-catch",
                    message=(
                        "empty catch block silently swallows the error — "
                        "log the error or recover explicitly"
                    ),
                )
        i += 1


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    is_test = _is_test(path)

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith("//"):
            continue

        # innerHTML / outerHTML assignment
        if m := _INNER_HTML.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-inner-html",
                message=(
                    f"{m.group(1)} = allows XSS via DOM injection — "
                    "use textContent for text, or sanitize with DOMPurify"
                ),
            )

        # dangerouslySetInnerHTML
        if m := _DANGEROUS_HTML.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-dangerous-html",
                message=(
                    "dangerouslySetInnerHTML allows XSS — "
                    "sanitize content with DOMPurify before use"
                ),
            )

        # new Function(...)
        if m := _NEW_FUNCTION.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-new-function",
                message=(
                    "new Function() executes arbitrary code — "
                    "use a proper parser or data structure instead"
                ),
            )

        # Single-line empty catch
        if not is_test:
            if m := _EMPTY_CATCH_1.search(raw):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-empty-catch",
                    message=(
                        "empty catch block silently swallows the error — "
                        "log the error or recover explicitly"
                    ),
                )

    # Multi-line empty catch
    if not is_test:
        yield from _check_empty_catch_multiline(path, lines)
