#!/usr/bin/env bash
# install.sh — Install rulestools and optionally set up a project.
#
# Usage:
#   bash install.sh                   # install only
#   bash install.sh /path/to/project  # install + full project setup
#
# "Full project setup" runs: rulestools setup <project>
#   - Detects languages, writes proj/rulestools.toml
#   - Installs VSCode task (auto-starts scanner on folder open)
#   - Installs pre-commit hook (blocks commit on rule errors)
#   - Writes proj/RULES-MCP.md (AI fix-guidance context)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ── 1. Find Python 3.11+ ──────────────────────────────────────────────────────

PY=""
for candidate in python3 python python3.11 python3.12 python3.13; do
    if command -v "$candidate" &>/dev/null; then
        version=$("$candidate" -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')" 2>/dev/null || echo "0.0")
        major=${version%%.*}
        minor=${version##*.}
        if [ "$major" -ge 3 ] && [ "$minor" -ge 11 ]; then
            PY="$candidate"
            break
        fi
    fi
done

if [ -z "$PY" ]; then
    echo "Error: Python 3.11+ not found." >&2
    echo "Install from https://www.python.org/downloads/" >&2
    exit 1
fi

echo "Python $("$PY" --version) found."

# ── 2. Install rulestools ─────────────────────────────────────────────────────

echo "Installing rulestools from $SCRIPT_DIR ..."
"$PY" -m pip install -e "$SCRIPT_DIR" --quiet
echo "Done."

# Verify the command is available
if ! command -v rulestools &>/dev/null; then
    # Fallback: use python -m directly
    RT_CMD="\"$PY\" \"$SCRIPT_DIR/cli.py\""
    echo ""
    echo "Note: 'rulestools' not on PATH."
    echo "Add Python Scripts to PATH, or use:"
    echo "  $RT_CMD scan <project>"
else
    RT_CMD="rulestools"
    echo "'rulestools' command available."
fi

# ── 3. Optionally set up a project ───────────────────────────────────────────

PROJECT="${1:-}"

if [ -n "$PROJECT" ]; then
    echo ""
    echo "Setting up project: $PROJECT"
    echo "(detect languages + VSCode task + pre-commit hook + proj/RULES-MCP.md)"
    echo ""
    rulestools setup "$PROJECT"
else
    echo ""
    echo "To set up a project run:"
    echo "  rulestools setup /path/to/project"
    echo ""
    echo "This installs:"
    echo "  proj/rulestools.toml    — detected language config"
    echo "  .vscode/tasks.json      — scanner starts automatically on folder open"
    echo "  .git/hooks/pre-commit   — blocks commits that introduce rule errors"
    echo "  proj/RULES-MCP.md       — AI fix-guidance context file"
fi
