"""Write issues to stdout and proj/ISSUES.

stdout format is VSCode problem-matcher compatible:
  path:line:col: severity rule: message

proj/ISSUES format adds a header with timestamp and issue counts.
"""

from __future__ import annotations
import sys
from datetime import datetime, timezone
from pathlib import Path

from .issue import Issue, Severity


def print_issues(issues: list[Issue], errors_only: bool = False) -> None:
    """Print issues to stdout/stderr.

    errors_only=True  — only print errors (used by pre-commit hook).
    errors_only=False — print everything; errors → stderr, warnings → stdout.
    """
    for issue in sorted(issues):
        if errors_only and issue.severity != Severity.ERROR:
            continue
        line = str(issue)
        if issue.severity == Severity.ERROR:
            print(line, file=sys.stderr)
        else:
            print(line)


def write_issues_file(issues: list[Issue], project_root: Path) -> Path:
    """Write sorted issues to proj/ISSUES. Returns the path written."""
    issues_dir = project_root / "proj"
    issues_dir.mkdir(exist_ok=True)
    issues_path = issues_dir / "ISSUES"

    errors   = [i for i in issues if i.severity == Severity.ERROR]
    warnings = [i for i in issues if i.severity == Severity.WARNING]
    infos    = [i for i in issues if i.severity == Severity.INFO]

    ts = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")

    lines: list[str] = [
        f"# ISSUES — {ts}",
        f"# {len(errors)} errors · {len(warnings)} warnings · {len(infos)} info",
        "",
    ]

    if not issues:
        lines.append("# clean — no issues found")
    else:
        current_file: Path | None = None
        for issue in sorted(issues):
            if issue.file != current_file:
                current_file = issue.file
                lines.append(f"\n## {issue.file}")
            lines.append(str(issue))

    issues_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return issues_path
