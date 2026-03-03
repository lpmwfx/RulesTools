"""rulestools CLI — scan a project for rule violations.

Commands:
  detect [PATH]               Auto-detect languages, write proj/rulestools.toml
  scan   [--watch] [PATH]     Scan project (reads config if present)
  init   [PATH]               Install VSCode task + pre-commit hook
"""

from __future__ import annotations
import sys
import time
from pathlib import Path

import click

# ── scanner registry ─────────────────────────────────────────────────────────

_ALL_LANGS = ["rust", "js", "slint", "css", "python", "kotlin"]

_EXTENSIONS: dict[str, set[str]] = {
    "rust":   {".rs"},
    "js":     {".js", ".mjs", ".ts", ".tsx"},
    "slint":  {".slint"},
    "css":    {".css", ".scss", ".sass"},
    "python": {".py"},
    "kotlin": {".kt", ".kts"},
}


def _load_scanner(name: str):
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


def _resolve_languages(root: Path, lang_opt: str | None) -> list[str]:
    """Priority: --lang flag > proj/rulestools.toml > error."""
    from common.config import ScanConfig

    if lang_opt:
        return [l.strip() for l in lang_opt.split(",") if l.strip()]

    cfg = ScanConfig.load(root)
    if cfg:
        click.echo(f"  config: proj/rulestools.toml ({', '.join(cfg.languages)})", err=True)
        return cfg.languages

    click.echo(
        "  no config found — run 'rulestools detect' first, or pass --lang",
        err=True,
    )
    sys.exit(1)


def _build_scanners(langs: list[str]) -> dict[str, object]:
    scanners: dict[str, object] = {}
    for lang in langs:
        try:
            scanners[lang] = _load_scanner(lang)
        except ValueError as e:
            click.echo(f"  warning: {e}", err=True)
    return scanners


def _run_once(root: Path, scanners: dict, errors_only: bool = False):
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


# ── commands ─────────────────────────────────────────────────────────────────

@click.group()
def cli() -> None:
    pass


@cli.command()
@click.argument("path", default=".", type=click.Path(exists=True, file_okay=False))
def detect(path: str) -> None:
    """Auto-detect languages and write proj/rulestools.toml."""
    from common.config import ScanConfig

    root = Path(path).resolve()
    click.echo(f"Detecting languages in {root} ...", err=True)

    cfg = ScanConfig.detect(root)

    if not cfg.languages:
        click.echo("  no supported source files found", err=True)
        sys.exit(1)

    click.echo(f"  found: {', '.join(cfg.languages)}", err=True)

    config_path = cfg.save(root)
    click.echo(f"  written: {config_path}")
    click.echo("\nRun 'rulestools scan' to scan the project.")


@cli.command()
@click.argument("path", default=".", type=click.Path(exists=True, file_okay=False))
@click.option("--watch",       is_flag=True, help="Stay alive and re-scan on file changes.")
@click.option("--lang",        default=None, help="Override config: comma-separated languages.")
@click.option("--pre-commit",  is_flag=True, hidden=True,
              help="Pre-commit mode: silent scan, only print errors, exit 1 on errors.")
def scan(path: str, watch: bool, lang: str | None, pre_commit: bool) -> None:
    """Scan project for rule violations (reads proj/rulestools.toml)."""
    root  = Path(path).resolve()
    langs = _resolve_languages(root, lang)
    scanners = _build_scanners(langs)

    if not scanners:
        click.echo("No scanners available.", err=True)
        sys.exit(1)

    if not pre_commit:
        click.echo(f"Scanning {root} — {', '.join(scanners)}", err=True)

    if not watch:
        issues, delta = _run_once(root, scanners, errors_only=pre_commit)
        sys.exit(1 if delta.errors else 0)

    # ── watch mode ───────────────────────────────────────────────────────────
    try:
        from watchdog.observers import Observer
        from watchdog.events import FileSystemEventHandler
    except ImportError:
        click.echo("pip install watchdog  (required for --watch)", err=True)
        sys.exit(1)

    watched_exts: set[str] = set()
    for name in scanners:
        watched_exts |= _EXTENSIONS.get(name, set())
    # Also watch the config file
    watched_exts.add(".toml")

    class Handler(FileSystemEventHandler):
        def __init__(self) -> None:
            self._pending = False

        def on_any_event(self, event):
            if event.is_directory:
                return
            if Path(event.src_path).suffix in watched_exts:
                self._pending = True

        def consume(self) -> bool:
            if self._pending:
                self._pending = False
                return True
            return False

    handler = Handler()
    observer = Observer()
    observer.schedule(handler, str(root), recursive=True)
    observer.start()

    click.echo("Watching — Ctrl+C to stop", err=True)
    try:
        _run_once(root, scanners)
        while True:
            time.sleep(0.5)
            if handler.consume():
                langs    = _resolve_languages(root, lang)
                scanners = _build_scanners(langs)
                click.echo("\n[change — rescanning]", err=True)
                _run_once(root, scanners)
    except KeyboardInterrupt:
        observer.stop()
    observer.join()


@cli.command()
@click.argument("path", default=".", type=click.Path(exists=True, file_okay=False))
def init(path: str) -> None:
    """Install VSCode task and pre-commit hook into the project."""
    import json

    root     = Path(path).resolve()
    cli_path = Path(__file__).resolve()
    rt_root  = cli_path.parent
    cli_str  = cli_path.as_posix()
    rt_str   = rt_root.as_posix()
    py_str   = sys.executable.replace("\\", "/")

    # ── VSCode tasks.json ─────────────────────────────────────────────────────
    vscode_dir = root / ".vscode"
    vscode_dir.mkdir(exist_ok=True)
    tasks_dst = vscode_dir / "tasks.json"

    tasks = {
        "version": "2.0.0",
        "tasks": [
            {
                "label": "Rules: watch",
                "type": "shell",
                "command": py_str,
                "args": [cli_str, "scan", "--watch", "${workspaceFolder}"],
                "options": {"env": {"PYTHONPATH": rt_str}},
                "isBackground": True,
                "runOptions": {"runOn": "folderOpen"},
                "presentation": {
                    "reveal": "silent",
                    "panel": "dedicated",
                    "label": "Rules scanner",
                },
                "problemMatcher": {
                    "owner": "rulestools",
                    "fileLocation": ["autoDetect", "${workspaceFolder}"],
                    "background": {
                        "activeOnStart": True,
                        "beginsPattern": "Scanning",
                        "endsPattern": "issues",
                    },
                    "pattern": {
                        "regexp": r"^(.+):(\d+):(\d+): (error|warning|info) ([^:]+): (.+)$",
                        "file": 1, "line": 2, "column": 3,
                        "severity": 4, "code": 5, "message": 6,
                    },
                },
            },
            {
                "label": "Rules: scan once",
                "type": "shell",
                "command": py_str,
                "args": [cli_str, "scan", "${workspaceFolder}"],
                "options": {"env": {"PYTHONPATH": rt_str}},
                "presentation": {"reveal": "always", "panel": "shared"},
                "problemMatcher": {
                    "owner": "rulestools",
                    "fileLocation": ["autoDetect", "${workspaceFolder}"],
                    "pattern": {
                        "regexp": r"^(.+):(\d+):(\d+): (error|warning|info) ([^:]+): (.+)$",
                        "file": 1, "line": 2, "column": 3,
                        "severity": 4, "code": 5, "message": 6,
                    },
                },
            },
            {
                "label": "Rules: detect languages",
                "type": "shell",
                "command": py_str,
                "args": [cli_str, "detect", "${workspaceFolder}"],
                "options": {"env": {"PYTHONPATH": rt_str}},
                "presentation": {"reveal": "always", "panel": "shared"},
                "problemMatcher": [],
            },
        ],
    }

    if tasks_dst.exists():
        click.echo(f"  skipped (exists): {tasks_dst}")
    else:
        tasks_dst.write_text(json.dumps(tasks, indent=2), encoding="utf-8")
        click.echo(f"  created: {tasks_dst}")

    # ── pre-commit hook ───────────────────────────────────────────────────────
    git_hooks = root / ".git" / "hooks"
    if git_hooks.exists():
        hook_dst = git_hooks / "pre-commit"
        hook_content = (
            "#!/usr/bin/env bash\n"
            "# pre-commit hook — rulestools\n"
            "# Silent scan: only errors are printed, proj/ISSUES is always updated.\n"
            "set -euo pipefail\n"
            f'PYTHONPATH="{rt_str}" \\\n'
            f'  "{py_str}" "{cli_str}" scan --pre-commit "$(git rev-parse --show-toplevel)"\n'
        )
        if hook_dst.exists():
            click.echo(f"  skipped (exists): {hook_dst}")
        else:
            hook_dst.write_text(hook_content, encoding="utf-8")
            hook_dst.chmod(0o755)
            click.echo(f"  created: {hook_dst}")
    else:
        click.echo("  skipped pre-commit (no .git found)")

    # ── proj/RULES-MCP.md — AI context file ──────────────────────────────────
    proj_dir = root / "proj"
    proj_dir.mkdir(exist_ok=True)
    rules_mcp_dst = proj_dir / "RULES-MCP.md"

    rules_mcp_content = (
        "# Rules MCP — AI context for proj/ISSUES\n\n"
        "This project is scanned by rulestools.\n"
        "Violations are written to `proj/ISSUES` after every commit\n"
        "and on file change when the VSCode scanner task is running.\n\n"
        "## Reading proj/ISSUES\n\n"
        "Every issue line follows this format:\n\n"
        "    path/to/file.rs:42:5: error rust/errors/no-unwrap: unwrap() in non-test code\n\n"
        "Fields: `file:line:col: severity rule-id: message`\n\n"
        "New issues since the last scan are marked `[NEW]`.\n\n"
        "## Getting fix guidance via MCP\n\n"
        "The rule ID maps directly to a Rules MCP file:\n\n"
        "    Take the first two segments of the rule ID and append .md\n\n"
        "    rust/errors/no-unwrap            ->  rust/errors.md\n"
        "    rust/modules/no-sibling-coupling ->  rust/modules.md\n"
        "    global/nesting                   ->  global/nesting.md\n"
        "    uiux/state-flow/no-callback-logic ->  uiux/state-flow.md\n\n"
        "Then call:\n\n"
        "    mcp__rules__get_rule(file=\"rust/errors.md\")\n\n"
        "to get the full rule text with examples and fix guidance.\n\n"
        "## Fix workflow\n\n"
        "1. Open `proj/ISSUES` — look for `[NEW]` markers\n"
        "2. For each rule ID, derive the MCP file and call `mcp__rules__get_rule`\n"
        "3. Fix the violation\n"
        "4. Run `rulestools scan` to confirm it is gone\n"
    )

    if rules_mcp_dst.exists():
        click.echo(f"  skipped (exists): {rules_mcp_dst}")
    else:
        rules_mcp_dst.write_text(rules_mcp_content, encoding="utf-8")
        click.echo(f"  created: {rules_mcp_dst}")

    click.echo(
        f"\nDone. Run 'rulestools detect' to create proj/rulestools.toml\n"
        f"Then open VSCode — the scanner starts automatically.\n"
        f"AI context: add '@proj/RULES-MCP.md' to your project CLAUDE.md."
    )


if __name__ == "__main__":
    cli()
