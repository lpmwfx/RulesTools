"""Hardcoded secrets check — from global/secrets.md.

Flags credential-like literals assigned directly in source code.
Secrets belong in ~/.env/ and must never be copied into project files.
"""

from __future__ import annotations
from pathlib import Path
from typing import Generator
import re

from .issue import Issue, Severity

_RULE = "global/secrets"

# Assignment patterns: key = "value" where key suggests a credential
_SECRET_KEY = re.compile(
    r"""(?ix)
    \b(
        password | passwd | pwd |
        api[_-]?key | apikey |
        secret[_-]?key | client[_-]?secret |
        access[_-]?token | auth[_-]?token | bearer[_-]?token |
        private[_-]?key | signing[_-]?key |
        database[_-]?url | db[_-]?password |
        aws[_-]?secret | aws[_-]?access | aws[_-]?key
    )\s*[=:]\s*["'][^"']{4,}["']
    """
)

# PEM private key headers
_PEM_KEY = re.compile(r"-----BEGIN\s+(RSA|EC|OPENSSH|PRIVATE)\s+PRIVATE KEY-----")

# Skip files that are clearly test fixtures, templates, or documentation
_SKIP_PARTS = {"test", "tests", "fixtures", "examples", "docs", "__pycache__"}
_SKIP_SUFFIXES = {".md", ".txt", ".rst", ".toml", ".example", ".template"}


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    if path.suffix.lower() in _SKIP_SUFFIXES:
        return
    if any(part in _SKIP_PARTS for part in path.parts):
        return

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        if stripped.startswith(("#", "//", "*")):  # skip pure comment lines
            continue

        if m := _SECRET_KEY.search(raw):
            key_name = m.group(1)
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=_RULE,
                message=(
                    f"hardcoded credential '{key_name}' — "
                    f"move to ~/.env/ and load via environment variable"
                ),
            )

        if m := _PEM_KEY.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=_RULE,
                message="PEM private key in source file — must never be committed",
            )
