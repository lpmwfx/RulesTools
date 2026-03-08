"""Self-check scanner — validates the RulesTools/RulesMCP source against
global/install-architecture.md.

Ensures no source file in the rules repos contains hardcoded local drive
paths, file:/// local URLs, sys.path local inserts, or editable installs.

Run via:  rulestools selfcheck [PATH]
"""

from __future__ import annotations
import re
from pathlib import Path
from typing import Generator

from common.issue import Issue, Severity

_RULE = "global/install-architecture"

# ── patterns ────────────────────────────────────────────────────────────────

# Drive-letter path: D:/ D:\\ C:/REPO etc. — must be preceded by a non-alpha char
_DRIVE_PATH = re.compile(r"""(?<![A-Za-z])[A-Za-z]:[/\\]""")

# file:/// local URL
_FILE_URL = re.compile(r"""file:///[A-Za-z]:[/\\]""", re.IGNORECASE)

# sys.path manipulation
_SYS_PATH = re.compile(r"""\bsys\.path\.(insert|append)\s*\(""")

# Editable install not pointing at git+https or https
_EDITABLE = re.compile(r"""-e\s+(?!git\+https)(?!https)""")

# ── files/dirs to skip ──────────────────────────────────────────────────────

_SKIP_DIRS = {
    ".git", "__pycache__", ".venv", "venv",
    "node_modules", "target", "dist", "build",
    ".tox", "site-packages", ".egg-info",
    ".claude",   # Claude Desktop settings — not source code
}

# Filenames that are legitimately about installation (skip entirely)
_SKIP_NAMES = {
    "install.sh", "install.ps1", "install.bat",
    "direct_url.json",   # pip metadata — managed by pip
    "RECORD", "WHEEL", "METADATA",
}

# These suffixes are not source and should not be scanned
_SKIP_SUFFIXES = {
    ".exe", ".dll", ".so", ".pyc", ".pyo",
    ".png", ".jpg", ".gif", ".ico",
    ".lock", ".sum",
}

# Suffixes we DO scan
_SCAN_SUFFIXES = {
    ".py", ".toml", ".json", ".yaml", ".yml",
    ".sh", ".ps1", ".bat", ".md", ".txt",
}


def _comment_prefixes(suffix: str) -> tuple[str, ...]:
    if suffix == ".py":
        return ("#",)
    if suffix in {".toml", ".ini", ".cfg", ".sh"}:
        return ("#",)
    if suffix in {".yaml", ".yml"}:
        return ("#",)
    return ()


def check_file(path: Path) -> list[Issue]:
    if path.name in _SKIP_NAMES:
        return []
    if path.suffix.lower() in _SKIP_SUFFIXES:
        return []
    if path.suffix.lower() not in _SCAN_SUFFIXES:
        return []
    if any(part in _SKIP_DIRS for part in path.parts):
        return []

    try:
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return []

    suffix = path.suffix.lower()
    comment_pfx = _comment_prefixes(suffix)
    issues: list[Issue] = []

    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()

        # Skip pure comment lines (informational drive-path mentions in docs are OK)
        if comment_pfx and stripped.startswith(comment_pfx):
            continue

        # file:/// local URL
        if m := _FILE_URL.search(raw):
            issues.append(Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE}/no-file-url",
                message=(
                    "file:/// URL points to a local drive — "
                    "install from GitHub: pip install git+https://..."
                ),
            ))
            continue  # don't double-report the drive path too

        # Drive-letter path in non-comment line
        if m := _DRIVE_PATH.search(raw):
            issues.append(Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE}/no-local-path",
                message=(
                    "hardcoded local drive path — "
                    "use GitHub URL or platformdirs.user_cache_dir(), never a drive letter"
                ),
            ))

        # sys.path manipulation (Python only)
        if suffix == ".py":
            if m := _SYS_PATH.search(raw):
                issues.append(Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE}/no-sys-path",
                    message=(
                        "sys.path.insert/append adds a runtime local-path dependency — "
                        "install the package via pip instead"
                    ),
                ))

        # Editable install markers
        if suffix in {".txt", ".toml"}:
            if m := _EDITABLE.search(raw):
                issues.append(Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE}/no-editable",
                    message=(
                        "editable install (-e local/path) creates a local-path dependency — "
                        "use: pip install git+https://github.com/..."
                    ),
                ))

    return issues


def scan_tree(root: Path) -> Generator[Issue, None, None]:
    """Yield all local-path violations under root."""
    for path in sorted(root.rglob("*")):
        if not path.is_file():
            continue
        if any(part in _SKIP_DIRS for part in path.parts):
            continue
        yield from check_file(path)
