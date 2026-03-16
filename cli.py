"""rulestools CLI — scan a project for rule violations.

Commands:
  setup  [PATH]               Detect languages + install VSCode task + pre-commit hook
  detect [PATH]               Auto-detect languages, write proj/rulestools.toml
  scan   [--watch] [PATH]     Scan project (reads config if present)
  init   [PATH]               Install VSCode task + pre-commit hook
"""

from __future__ import annotations
import sys
import time
from pathlib import Path

import click

from common.runner import EXTENSIONS, resolve_languages, build_scanners, run_once
from common.installer import make_tasks, RULES_MCP_MD


# ── commands ─────────────────────────────────────────────────────────────────

@click.group()
def cli() -> None:
    pass


@cli.command()
@click.argument("path", default=".", type=click.Path(exists=True, file_okay=False))
@click.pass_context
def setup(ctx, path: str) -> None:
    """Detect languages + install VSCode task + pre-commit hook in one step."""
    ctx.invoke(detect, path=path)
    ctx.invoke(init, path=path)


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
    root     = Path(path).resolve()
    langs    = resolve_languages(root, lang)
    scanners = build_scanners(langs)

    if not scanners:
        click.echo("No scanners available.", err=True)
        sys.exit(1)

    if not pre_commit:
        click.echo(f"Scanning {root} — {', '.join(scanners)}", err=True)

    if not watch:
        _, delta = run_once(root, scanners, errors_only=pre_commit)
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
        watched_exts |= EXTENSIONS.get(name, set())
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
        run_once(root, scanners)
        while True:
            time.sleep(0.5)
            if handler.consume():
                langs    = resolve_languages(root, lang)
                scanners = build_scanners(langs)
                click.echo("\n[change — rescanning]", err=True)
                run_once(root, scanners)
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

    if tasks_dst.exists():
        click.echo(f"  skipped (exists): {tasks_dst}")
    else:
        tasks = make_tasks(py_str, cli_str, rt_str)
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
            "\n"
            "# Rust: block commit if public items lack /// doc comments\n"
            'ROOT="$(git rev-parse --show-toplevel)"\n'
            'if [ -f "$ROOT/Cargo.toml" ] && command -v rustdocumenter &>/dev/null; then\n'
            '  rustdocumenter check "$ROOT" || {\n'
            '    echo "Run \'rustdocumenter\' to auto-generate missing /// doc comments." >&2\n'
            "    exit 1\n"
            "  }\n"
            "fi\n"
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

    if rules_mcp_dst.exists():
        click.echo(f"  skipped (exists): {rules_mcp_dst}")
    else:
        rules_mcp_dst.write_text(RULES_MCP_MD, encoding="utf-8")
        click.echo(f"  created: {rules_mcp_dst}")

    click.echo(
        f"\nDone. Run 'rulestools detect' to create proj/rulestools.toml\n"
        f"Then open VSCode — the scanner starts automatically.\n"
        f"AI context: add '@proj/RULES-MCP.md' to your project CLAUDE.md."
    )


@cli.command()
@click.argument("path", default=".", type=click.Path(exists=True, file_okay=False))
def check(path: str) -> None:
    """Pre-commit check: scan staged files, unstage those with errors."""
    from common.precommit import run_check
    run_check(path)


@cli.command()
@click.argument("path", default=".", type=click.Path(exists=True, file_okay=False))
def selfcheck(path: str) -> None:
    """Scan the RulesTools/RulesMCP source for local drive-path references.

    Checks every source file under PATH for violations of
    global/install-architecture.md — no local drive-letter paths,
    no file:/// URLs, no sys.path local inserts, no editable installs.

    Use this after editing the rules repos to confirm no local-path
    dependency was accidentally introduced.
    """
    from common.selfcheck import scan_tree
    from common.writer import print_issues

    root = Path(path).resolve()
    click.echo(f"Self-check: scanning {root} for local drive-path references ...", err=True)

    issues = sorted(scan_tree(root))
    print_issues(issues)

    if issues:
        click.echo(
            f"\n{len(issues)} violation(s) found — fix before pushing to GitHub.",
            err=True,
        )
        sys.exit(1)
    else:
        click.echo("  clean — no local drive-path references found.", err=True)


if __name__ == "__main__":
    cli()
