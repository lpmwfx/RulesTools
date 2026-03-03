"""Write issues to stdout and proj/ISSUES.

stdout format is VSCode problem-matcher compatible:
  path:line:col: severity rule: message

proj/ISSUES format adds a header with timestamp and issue counts.

Delta comparison uses (file, rule, message) as identity key —
line numbers are ignored so that code movement doesn't look like new issues.
"""

from __future__ import annotations
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

from .issue import Issue, Severity

# Matches a line written by this module:  path:line:col: sev rule: msg
_ISSUE_LINE = re.compile(
    r"^(.+):(\d+):(\d+): (error|warning|info) ([^:]+): (.+)$"
)


def _issue_key(issue: Issue) -> tuple:
    """Identity that survives line-number shifts."""
    return (str(issue.file), issue.rule, issue.message)


def _load_previous(issues_path: Path) -> set[tuple]:
    """Parse an existing ISSUES file and return a set of identity keys."""
    if not issues_path.exists():
        return set()
    keys: set[tuple] = set()
    for line in issues_path.read_text(encoding="utf-8", errors="replace").splitlines():
        if m := _ISSUE_LINE.match(line):
            file_str, _, _, _, rule, message = m.groups()
            keys.add((file_str, rule, message))
    return keys


def _banner(lines: list[str]) -> None:
    """Print a framed banner to stderr."""
    width = max(len(l) for l in lines) + 4
    border = "-" * width
    print(border, file=sys.stderr)
    for l in lines:
        print(f"  {l}", file=sys.stderr)
    print(border, file=sys.stderr)


def print_issues(issues: list[Issue], errors_only: bool = False) -> None:
    """Print issues to stdout/stderr.

    errors_only=True  — only errors (pre-commit mode).
    errors_only=False — all issues; errors to stderr, warnings to stdout.
    """
    for issue in sorted(issues):
        if errors_only and issue.severity != Severity.ERROR:
            continue
        line = str(issue)
        if issue.severity == Severity.ERROR:
            print(line, file=sys.stderr)
        else:
            print(line)


def write_issues_file(issues: list[Issue], project_root: Path) -> "Delta":
    """Write sorted issues to proj/ISSUES. Returns a Delta with new/resolved counts."""
    issues_dir  = project_root / "proj"
    issues_dir.mkdir(exist_ok=True)
    issues_path = issues_dir / "ISSUES"

    # ── delta against previous scan ──────────────────────────────────────────
    prev_keys    = _load_previous(issues_path)
    current_keys = {_issue_key(i) for i in issues}
    new_keys     = current_keys - prev_keys
    resolved_keys = prev_keys - current_keys

    # ── write file ───────────────────────────────────────────────────────────
    errors   = [i for i in issues if i.severity == Severity.ERROR]
    warnings = [i for i in issues if i.severity == Severity.WARNING]
    infos    = [i for i in issues if i.severity == Severity.INFO]

    ts = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")

    file_lines: list[str] = [
        f"# ISSUES — {ts}",
        f"# {len(errors)} errors · {len(warnings)} warnings · {len(infos)} info",
        f"# delta: +{len(new_keys)} new  -{len(resolved_keys)} resolved",
        "# Rule lookup: mcp__rules__get_rule(file=\"<lang>/<topic>.md\")  (first two rule-ID segments + .md)",
        "",
    ]

    if not issues:
        file_lines.append("# clean — no issues found")
    else:
        current_file: Path | None = None
        for issue in sorted(issues):
            marker = " [NEW]" if _issue_key(issue) in new_keys else ""
            if issue.file != current_file:
                current_file = issue.file
                file_lines.append(f"\n## {issue.file}")
            file_lines.append(str(issue) + marker)

    issues_path.write_text("\n".join(file_lines) + "\n", encoding="utf-8")

    return Delta(
        total    = len(issues),
        errors   = len(errors),
        warnings = len(warnings),
        new      = len(new_keys),
        resolved = len(resolved_keys),
        first_scan = not bool(prev_keys),
    )


class Delta:
    def __init__(
        self, total: int, errors: int, warnings: int,
        new: int, resolved: int, first_scan: bool,
    ) -> None:
        self.total     = total
        self.errors    = errors
        self.warnings  = warnings
        self.new       = new
        self.resolved  = resolved
        self.first_scan = first_scan

    def print_banner(self, pre_commit: bool = False) -> None:
        """Print a clear status banner to stderr."""
        if self.total == 0:
            _banner(["CLEAN — no issues found"])
            return

        if pre_commit:
            # Compact — just the delta line, banner only on errors
            if self.errors:
                _banner([
                    f"COMMIT BLOCKED — {self.errors} errors",
                    f"+{self.new} new  -{self.resolved} resolved",
                    "See proj/ISSUES for details",
                ])
            return

        # Normal scan banner
        if self.first_scan:
            delta_str = "first scan"
        else:
            parts = []
            if self.new:
                parts.append(f"+{self.new} new")
            if self.resolved:
                parts.append(f"-{self.resolved} resolved")
            delta_str = "  ".join(parts) if parts else "no change"

        if self.errors == 0:
            status = "OK — no errors"
        else:
            status = f"{self.errors} ERRORS"

        lines = [
            f"{status}  ({self.warnings} warnings)",
            delta_str,
        ]
        if self.errors:
            lines.append("proj/ISSUES updated — [NEW] markers on new issues")

        _banner(lines)
