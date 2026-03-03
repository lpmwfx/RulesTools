"""Templates and install helpers used by the init/setup CLI commands."""

from __future__ import annotations

_PROBLEM_PATTERN = {
    "regexp": r"^(.+):(\d+):(\d+): (error|warning|info) ([^:]+): (.+)$",
    "file": 1, "line": 2, "column": 3,
    "severity": 4, "code": 5, "message": 6,
}


def make_tasks(py_str: str, cli_str: str, rt_str: str) -> dict:
    """Return a VSCode tasks.json structure with absolute paths baked in."""
    return {
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
                    "pattern": _PROBLEM_PATTERN,
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
                    "pattern": _PROBLEM_PATTERN,
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


RULES_MCP_MD = """\
# Rules MCP — AI context for proj/ISSUES

This project is scanned by rulestools.
Violations are written to `proj/ISSUES` after every commit
and on file change when the VSCode scanner task is running.

## Reading proj/ISSUES

Every issue line follows this format:

    path/to/file.rs:42:5: error rust/errors/no-unwrap: unwrap() in non-test code

Fields: `file:line:col: severity rule-id: message`

New issues since the last scan are marked `[NEW]`.

## Getting fix guidance via MCP

The rule ID maps to a Rules MCP file — take first two segments + .md:

    rust/errors/no-unwrap            ->  rust/errors.md
    rust/modules/no-sibling-coupling ->  rust/modules.md
    global/nesting                   ->  global/nesting.md
    uiux/state-flow/no-callback-logic ->  uiux/state-flow.md

Then call:

    mcp__rules__get_rule(file="rust/errors.md")

to get the full rule text with examples and fix guidance.

## Fix workflow

1. Open `proj/ISSUES` — look for `[NEW]` markers
2. For each rule ID, derive the MCP file and call `mcp__rules__get_rule`
3. Fix the violation
4. Run `rulestools scan` to confirm it is gone
"""
