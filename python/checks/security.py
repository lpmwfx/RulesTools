"""Python security checks — python/security.md + global/error-flow.md.

Checks:
  - subprocess with shell=True  — command injection
  - pickle.load / pickle.loads  — arbitrary code execution
  - yaml.load without SafeLoader — arbitrary code execution
  - os.system()                 — shell injection, use subprocess.run(list)
  - Hardcoded /tmp/ paths       — use tempfile module
"""

from __future__ import annotations
import re
from pathlib import Path
from typing import Generator

from common.issue import Issue, Severity

_RULE_BASE = "python/security"

_SHELL_TRUE     = re.compile(r"\bshell\s*=\s*True")
_PICKLE         = re.compile(r"\bpickle\s*\.\s*loads?\s*\(")
_YAML_LOAD      = re.compile(r"\byaml\s*\.\s*load\s*\(")
_OS_SYSTEM      = re.compile(r"\bos\s*\.\s*system\s*\(")
_HARDCODED_TMP  = re.compile(r"""['"][/\\]tmp[/\\]""")


def _is_test(path: Path) -> bool:
    return "test" in path.parts or path.stem.startswith("test_")


def _code_lines(lines: list[str]) -> Generator[tuple[int, str], None, None]:
    """Yield (lineno, raw) skipping comments and triple-quoted docstrings."""
    in_triple = False
    triple_char = ""
    for lineno, raw in enumerate(lines, start=1):
        stripped = raw.lstrip()
        for q in ('"""', "'''"):
            if raw.count(q) % 2 == 1:
                if in_triple and triple_char == q:
                    in_triple = False
                else:
                    in_triple = True
                    triple_char = q
        if in_triple or stripped.startswith("#"):
            continue
        yield lineno, raw


def check(path: Path, lines: list[str]) -> Generator[Issue, None, None]:
    is_test = _is_test(path)

    for lineno, raw in _code_lines(lines):

        # subprocess shell=True — flag everywhere (not just tests)
        if m := _SHELL_TRUE.search(raw):
            yield Issue(
                file=path, line=lineno, col=m.start() + 1,
                severity=Severity.ERROR,
                rule=f"{_RULE_BASE}/no-shell-true",
                message=(
                    "shell=True passes command through the shell — "
                    "use list args: subprocess.run(['prog', arg1, arg2])"
                ),
            )

        if not is_test:
            # pickle
            if m := _PICKLE.search(raw):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.ERROR,
                    rule=f"{_RULE_BASE}/no-pickle",
                    message=(
                        "pickle.load/loads executes arbitrary code on untrusted data — "
                        "use json, tomllib, or msgpack instead"
                    ),
                )

            # yaml.load without explicit SafeLoader
            if m := _YAML_LOAD.search(raw):
                if "SafeLoader" not in raw and "Loader=" not in raw:
                    yield Issue(
                        file=path, line=lineno, col=m.start() + 1,
                        severity=Severity.ERROR,
                        rule=f"{_RULE_BASE}/no-yaml-load",
                        message=(
                            "yaml.load() executes arbitrary code — "
                            "use yaml.safe_load() or yaml.load(data, Loader=yaml.SafeLoader)"
                        ),
                    )

            # os.system
            if m := _OS_SYSTEM.search(raw):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/no-os-system",
                    message=(
                        "os.system() invokes the shell — "
                        "use subprocess.run(['prog', ...], check=True) instead"
                    ),
                )

            # Hardcoded /tmp/ path
            if m := _HARDCODED_TMP.search(raw):
                yield Issue(
                    file=path, line=lineno, col=m.start() + 1,
                    severity=Severity.WARNING,
                    rule=f"{_RULE_BASE}/no-hardcoded-tmp",
                    message=(
                        "hardcoded /tmp/ path is not portable and may be predictable — "
                        "use tempfile.mkstemp() or tempfile.TemporaryDirectory()"
                    ),
                )
