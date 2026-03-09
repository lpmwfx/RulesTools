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

# BinaryFormatter — RCE risk
_BINARY_FORMATTER = re.compile(r"\bBinaryFormatter\b")

# XXE via XmlDocument.Load / LoadXml
_XML_LOAD = re.compile(r"\bXmlDocument\b.*\.(Load|LoadXml)\s*\(")

# Path traversal: File.* with interpolated path
_FILE_USER_INPUT = re.compile(
    r"\bFile\.(ReadAllText|ReadAllBytes|WriteAllText|WriteAllBytes|Delete|Move|Copy)\s*\(\s*\$[\"']"
)

# Regex without timeout
_REGEX_NEW = re.compile(r"\bnew\s+Regex\s*\(")


def _is_comment(raw: str) -> bool:
    stripped = raw.lstrip()
    return stripped.startswith("//") or stripped.startswith("*")


def _is_test_context(lines: list[str], lineno: int) -> bool:
    for i in range(lineno - 2, max(lineno - 50, -1), -1):
        line = lines[i]
        if "[Test]" in line or "[Fact]" in line or "[Theory]" in line or "Assert." in line:
            return True
    return False


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

        # Check 1 — BinaryFormatter
        if m := _BINARY_FORMATTER.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-binary-formatter",
                message=(
                    "BinaryFormatter is banned — RCE risk. "
                    "Use System.Text.Json or ProtoBuf"
                ),
            )

        # Check 2 — XXE via XmlDocument
        if m := _XML_LOAD.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/xxe-risk",
                message=(
                    "XmlDocument.Load/LoadXml — XXE risk unless DtdProcessing=Prohibit is set. "
                    "Use XmlReader with XmlReaderSettings { DtdProcessing = DtdProcessing.Prohibit }"
                ),
            )

        # Check 3 — Path traversal via File.* with interpolated path
        if _FILE_USER_INPUT.search(raw):
            # Skip test context
            if not _is_test_context(lines, lineno):
                yield Issue(
                    file=path, line=lineno, col=1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/path-traversal",
                    message=(
                        "File.* with interpolated path — "
                        "validate path against allowed base dir to prevent traversal"
                    ),
                )

        # Check 4 — Regex without timeout
        if m := _REGEX_NEW.search(raw):
            after = raw[m.end():]
            if "TimeSpan" not in after and "matchTimeout" not in after.lower():
                if not _is_test_context(lines, lineno):
                    yield Issue(
                        file=path, line=lineno, col=m.start() + 1,
                        severity=Severity.WARNING,
                        rule=f"{_RULE_BASE}/regex-no-timeout",
                        message=(
                            "new Regex() without matchTimeout — ReDoS risk on user-supplied patterns. "
                            "Pass TimeSpan as third argument or use "
                            "Regex.IsMatch(input, pattern, options, timeout)"
                        ),
                    )
