#!/usr/bin/env bash
# install.sh — System installer for RulesTools + RulesToolsMCP + RulesMCP
#
# Installs packages to site-packages (not editable).
# After running, the REPO folder is no longer needed at runtime.
#
# Usage:
#   bash install.sh            # first-time install + register MCP servers
#   bash install.sh update     # upgrade packages + re-register if needed

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RULESTOOLS_DIR="$SCRIPT_DIR"
RULESTOOLS_MCP_DIR="$SCRIPT_DIR/../RulesToolsMCP"
RULES_MCP_DIR="$SCRIPT_DIR/../RulesMCP"

MODE="${1:-install}"

# ── 1. Find Python 3.11+ ──────────────────────────────────────────────────────

PY=""
for candidate in python3.13 python3.12 python3.11 python3 python; do
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

echo "Using $("$PY" --version)"

# ── 2. Install / upgrade packages ─────────────────────────────────────────────

if [ "$MODE" = "update" ]; then
    PIP_FLAGS="--quiet --force-reinstall"
    echo "Mode: update (reinstalling all packages from REPO)"
else
    PIP_FLAGS="--quiet"
    echo "Mode: install"
fi

echo ""
echo "Installing rulestools ..."
"$PY" -m pip install $PIP_FLAGS "$RULESTOOLS_DIR"

if [ -d "$RULESTOOLS_MCP_DIR" ]; then
    echo "Installing rulestools-mcp ..."
    "$PY" -m pip install $PIP_FLAGS "$RULESTOOLS_MCP_DIR"
else
    echo "Warning: RulesToolsMCP not found at $RULESTOOLS_MCP_DIR" >&2
fi

if [ -d "$RULES_MCP_DIR" ]; then
    echo "Installing rules-mcp ..."
    "$PY" -m pip install $PIP_FLAGS "$RULES_MCP_DIR"
else
    echo "Warning: RulesMCP not found at $RULES_MCP_DIR" >&2
fi

echo ""
echo "Packages installed to site-packages."

# ── 3. Register MCP servers (install mode only — update keeps existing) ───────

if [ "$MODE" != "update" ] && command -v claude &>/dev/null; then
    echo ""
    echo "Registering MCP servers in Claude Code ..."

    if claude mcp list 2>/dev/null | grep -q "^rules:"; then
        echo "  rules MCP already registered."
    else
        claude mcp add rules rules-mcp --scope user 2>/dev/null && \
            echo "  Added: rules (rules lookup)" || \
            echo "  Warning: could not register rules MCP." >&2
    fi

    if claude mcp list 2>/dev/null | grep -q "^rulestools:"; then
        echo "  rulestools MCP already registered."
    else
        claude mcp add rulestools rulestools-mcp --scope user 2>/dev/null && \
            echo "  Added: rulestools (code scanner)" || \
            echo "  Warning: could not register rulestools MCP." >&2
    fi

elif [ "$MODE" != "update" ]; then
    echo ""
    echo "Claude Code CLI not found. Register manually:"
    echo "  claude mcp add rules rules-mcp --scope user"
    echo "  claude mcp add rulestools rulestools-mcp --scope user"
fi

# ── 4. Verify ─────────────────────────────────────────────────────────────────

echo ""
echo "Verifying ..."
"$PY" -c "from rulestools_mcp.scanner import scan_file; print('  rulestools-mcp: OK')"
"$PY" -c "from rules_mcp.registry import Registry; print('  rules-mcp: OK')"
echo ""

if [ "$MODE" = "update" ]; then
    echo "Update complete. New checks are active immediately."
else
    echo "Install complete. To set up a project, in Claude Code:"
    echo "  mcp__rulestools__setup(\"/path/to/project\")"
fi
