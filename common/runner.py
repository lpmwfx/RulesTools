"""Scanner orchestration helpers — used by cli.py commands."""

from __future__ import annotations
import sys
from pathlib import Path

import click

ALL_LANGS = ["rust", "js", "slint", "css", "python", "kotlin"]

EXTENSIONS: dict[str, set[str]] = {
    "rust":   {".rs"},
    "js":     {".js", ".mjs", ".ts", ".tsx"},
    "slint":  {".slint"},
    "css":    {".css", ".scss", ".sass"},
    "python": {".py"},
    "kotlin": {".kt", ".kts"},
}


def load_scanner(name: str):
    if name == "rust":
        from rust.scanner import scan_tree
    elif name == "js":
        from js.scanner import scan_tree
    elif name == "slint":
        from slint.scanner import scan_tree
    elif name == "css":
        from css.scanner import scan_tree
    elif name == "python":
        from python.scanner import scan_tree
    elif name == "kotlin":
        from kotlin.scanner import scan_tree
    else:
        raise ValueError(f"Unknown language: {name}")
    return scan_tree


def resolve_languages(root: Path, lang_opt: str | None) -> list[str]:
    """Priority: --lang flag > proj/rulestools.toml > exit with error."""
    from common.config import ScanConfig

    if lang_opt:
        return [lx.strip() for lx in lang_opt.split(",") if lx.strip()]

    cfg = ScanConfig.load(root)
    if cfg:
        click.echo(f"  config: proj/rulestools.toml ({', '.join(cfg.languages)})", err=True)
        return cfg.languages

    click.echo(
        "  no config found — run 'rulestools detect' first, or pass --lang",
        err=True,
    )
    sys.exit(1)


def build_scanners(langs: list[str]) -> dict[str, object]:
    scanners: dict[str, object] = {}
    for lang in langs:
        try:
            scanners[lang] = load_scanner(lang)
        except ValueError as e:
            click.echo(f"  warning: {e}", err=True)
    return scanners


def run_once(root: Path, scanners: dict[str, object], errors_only: bool = False):
    from common.issue import Issue
    from common.writer import print_issues, write_issues_file

    issues: list[Issue] = []
    for name, scan_tree in scanners.items():
        found = list(scan_tree(root))
        if found and not errors_only:
            click.echo(f"  {name}: {len(found)} issues", err=True)
        issues.extend(found)

    issues.sort()
    print_issues(issues, errors_only=errors_only)
    delta = write_issues_file(issues, root)
    delta.print_banner(pre_commit=errors_only)
    return issues, delta
