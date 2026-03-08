"""C# security checks — global/secrets.md + csharp-specific risks.

Checks:
  - SQL string concatenation: new SqlCommand("... " + var  or string.Format with SQL
  - Process.Start with string concat or user input
  - Hardcoded connection strings with passwords in source (common.secrets covers keys,
    this catches SqlConnection/SqlCommand patterns)
  - Environment.Exit() — abrupt shutdown skips IDisposable cleanup
"""

from __future__ import annotations
import re
from pathlib import Path
from typing import Generator

from common.issue import Issue, Severity

_RULE_BASE = "csharp/security"

# SQL concatenation:  "SELECT ... " + variable  or  $"SELECT ... {var}"
_SQL_CONCAT = re.compile(
    r'(?:SqlCommand|ExecuteReader|ExecuteNonQuery|ExecuteScalar)\s*\(\s*'
    r'(?:["\'].*["\']|\$["\'].*["\'])\s*\+',
    re.IGNORECASE,
)
_SQL_FORMAT = re.compile(
    r'(?:SqlCommand|ExecuteReader|ExecuteNonQuery|ExecuteScalar)\s*\(\s*'
    r'string\.Format\s*\(',
    re.IGNORECASE,
)
# $"... {var}" passed to SqlCommand
_SQL_INTERPOLATED = re.compile(
    r'(?:new\s+SqlCommand|ExecuteReader|ExecuteNonQuery)\s*\(\s*\$["\']',
    re.IGNORECASE,
)

# Process.Start with string concat or interpolation
_PROCESS_START = re.compile(r"\bProcess\s*\.\s*Start\s*\(")

# Environment.Exit
_ENV_EXIT = re.compile(r"\bEnvironment\s*\.\s*Exit\s*\(")

# Hardcoded password= in connection string literal
_CONN_PASSWORD = re.compile(r'["\'].*[Pp]assword\s*=\s*[^;\'\"]+', re.IGNORECASE)


def _is_comment(raw: str) -> bool:
    stripped = raw.lstrip()
    return stripped.startswith("//") or stripped.startswith("*")


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    for lineno, raw in enumerate(lines, start=1):
        if _is_comment(raw):
            continue

        if _SQL_CONCAT.search(raw) or _SQL_FORMAT.search(raw) or _SQL_INTERPOLATED.search(raw):
            yield Issue(
                file=path, line=lineno, col=1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/sql-injection",
                message=(
                    "SQL query built with string concatenation or interpolation — "
                    "use parameterised queries (SqlParameter) to prevent SQL injection"
                ),
            )

        if m := _PROCESS_START.search(raw):
            # Only flag if there's a string concat or interpolation on the same line
            after = raw[m.end():]
            if "+" in after or "$\"" in after or "$'" in after:
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/process-start-injection",
                    message=(
                        "Process.Start() with dynamic argument — "
                        "validate and sanitize user-supplied values before use"
                    ),
                )

        if m := _ENV_EXIT.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.WARNING,
                rule=f"{_RULE_BASE}/no-environment-exit",
                message=(
                    "Environment.Exit() — bypasses IDisposable cleanup and "
                    "graceful shutdown. Throw an exception or use CancellationToken."
                ),
            )

        if m := _CONN_PASSWORD.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/hardcoded-password",
                message=(
                    "Hardcoded password in connection string — "
                    "use environment variables or a secrets manager"
                ),
            )
