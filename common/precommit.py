"""Pre-commit check — scan staged files, unstage those with errors.

Only error-free files remain staged. Called by `rulestools check`
and by .git/hooks/pre-commit. Not intended for manual use.

Exit codes:
  0 — clean files remain staged (commit proceeds)
  1 — ALL staged files have errors (commit blocked)
"""

from __future__ import annotations
import subprocess
import sys
from fnmatch import fnmatch
from pathlib import Path

import click

from common.config import ScanConfig, EXT_TO_LANG
from common.issue import Severity


def _load_excludes(root: Path) -> list[str]:
    """Load [scan].exclude glob patterns from proj/rulestools.toml."""
    try:
        import tomllib
        toml_path = root / "proj" / "rulestools.toml"
        if toml_path.is_file():
            with open(toml_path, "rb") as f:
                data = tomllib.load(f)
            return data.get("scan", {}).get("exclude", [])
    except Exception:
        pass
    return []


def run_check(path: str) -> None:
    """Scan staged files, unstage those with errors, keep clean files."""
    root = Path(path).resolve()

    # ── staged files ─────────────────────────────────────────────────────
    try:
        result = subprocess.run(
            ["git", "diff", "--cached", "--name-only", "--diff-filter=ACMR"],
            capture_output=True, text=True, cwd=str(root),
        )
        staged = [f.strip() for f in result.stdout.strip().splitlines() if f.strip()]
    except Exception:
        return
    if not staged:
        return

    excludes = _load_excludes(root)

    # ── load per-file scanners on demand ─────────────────────────────────
    _file_scanners: dict[str, object] = {}

    def _get_scanner(ext: str):
        lang = EXT_TO_LANG.get(ext)
        if lang is None or lang in _file_scanners:
            return _file_scanners.get(lang)
        try:
            mod = __import__(f"{lang}.scanner", fromlist=["scan_file"])
            _file_scanners[lang] = mod.scan_file
        except (ImportError, AttributeError):
            _file_scanners[lang] = None
        return _file_scanners.get(lang)

    # ── scan each staged file ────────────────────────────────────────────
    error_files: dict[str, list] = {}

    for rel_path in staged:
        if any(fnmatch(rel_path, pat) for pat in excludes):
            continue
        abs_path = root / rel_path
        if not abs_path.is_file():
            continue
        scanner = _get_scanner(abs_path.suffix.lower())
        if scanner is None:
            continue
        issues = scanner(abs_path)
        errors = [i for i in issues if i.severity == Severity.ERROR]
        if errors:
            error_files[rel_path] = errors

    if not error_files:
        click.echo("rulestools: all staged files are clean")
        return

    clean_files = [f for f in staged if f not in error_files]

    # ── unstage files with errors ────────────────────────────────────────
    click.echo(f"\nrulestools: REJECTED ({len(error_files)} files with errors):")
    for rel_path, errors in sorted(error_files.items()):
        click.echo(f"  {rel_path}")
        for i in errors:
            click.echo(f"    {i.line}:{i.col} {i.rule}: {i.message}")
        subprocess.run(
            ["git", "reset", "HEAD", rel_path],
            capture_output=True, cwd=str(root),
        )

    # ── report ───────────────────────────────────────────────────────────
    if clean_files:
        click.echo(f"\nrulestools: KEPT ({len(clean_files)} clean files):")
        for f in clean_files:
            click.echo(f"  {f}")
        click.echo(
            f"\nresult: {len(clean_files)} staged (clean), "
            f"{len(error_files)} unstaged (errors)"
        )
        return  # exit 0 — commit proceeds with clean files

    click.echo(
        f"\nrulestools: ALL {len(error_files)} staged files have errors "
        f"— commit blocked"
    )
    sys.exit(1)
