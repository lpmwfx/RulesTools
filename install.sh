#!/usr/bin/env bash
# install.sh — System installer for the RulesTools ecosystem
#
# Installs ALL components from GitHub — no local repo needed at runtime.
#
# Python components (transitional — being replaced by Rust):
#   rulestools       CLI scanner + pre-commit hook
#   rulestools-mcp   MCP server for rulestools scanner
#   rules-mcp        MCP server for rule lookup
#
# Rust components (permanent):
#   file_size        standalone file-size checker
#   nesting          standalone brace-depth checker
#   secrets          standalone credential scanner
#
# Usage:
#   bash install.sh              # first-time install
#   bash install.sh update       # upgrade all components
#
# One-liner from GitHub:
#   curl -sSf https://raw.githubusercontent.com/lpmwfx/RulesTools/main/install.sh | bash

set -euo pipefail

MODE="${1:-install}"
GITHUB="https://github.com/lpmwfx"

echo "=== RulesTools installer (mode: $MODE) ==="
echo ""

# ─────────────────────────────────────────────────────────────────────────────
# 1. Python components
# ─────────────────────────────────────────────────────────────────────────────

PY=""
for candidate in python3.14 python3.13 python3.12 python3.11 python3 python; do
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
    echo "[!] Python 3.11+ not found — skipping Python components." >&2
    echo "    Install from https://www.python.org/downloads/" >&2
else
    echo "[Python] $("$PY" --version)"

    PIP_FLAGS="--quiet --force-reinstall"

    echo "  installing rulestools ..."
    "$PY" -m pip install $PIP_FLAGS "git+$GITHUB/RulesTools" 2>&1 \
        | grep -E "Resolved|Successfully" || true

    echo "  installing rulestools-mcp ..."
    "$PY" -m pip install $PIP_FLAGS "git+$GITHUB/RulesToolsMCP" 2>&1 \
        | grep -E "Resolved|Successfully" || true

    echo "  installing rules-mcp ..."
    "$PY" -m pip install $PIP_FLAGS "git+$GITHUB/RulesMCP" 2>&1 \
        | grep -E "Resolved|Successfully" || true

    echo "  [ok] Python components installed"
fi

# ─────────────────────────────────────────────────────────────────────────────
# 2. Rust components
# ─────────────────────────────────────────────────────────────────────────────

echo ""

if ! command -v cargo &>/dev/null; then
    echo "[!] cargo not found — skipping Rust components." >&2
    echo "    Install from https://rustup.rs/" >&2
else
    echo "[Rust] $(cargo --version)"
    echo "  installing file_size, nesting, secrets ..."
    cargo install --quiet --git "$GITHUB/RustScanners" --bins 2>&1 \
        | grep -E "Installed|Installing|Compiling|Finished" | tail -5 || true
    echo "  installing rustdocumenter, rustdoc-viewer ..."
    cargo install --quiet --git "$GITHUB/RustDocumenter" --bins 2>&1 \
        | grep -E "Installed|Installing|Compiling|Finished" | tail -5 || true

    # Install rustman wrapper scripts into ~/.cargo/bin/
    CARGO_BIN="${CARGO_HOME:-$HOME/.cargo}/bin"
    echo "  installing rustman wrapper → $CARGO_BIN ..."
    SCRIPTS_URL="https://raw.githubusercontent.com/lpmwfx/RustDocumenter/master/scripts"
    curl -sSf "$SCRIPTS_URL/rustman" -o "$CARGO_BIN/rustman" && chmod +x "$CARGO_BIN/rustman" || \
        echo "  [!] could not install rustman (bash)" >&2
    # Windows .bat (harmless on Linux/macOS)
    curl -sSf "$SCRIPTS_URL/rustman.bat" -o "$CARGO_BIN/rustman.bat" 2>/dev/null || true

    echo "  [ok] Rust binaries installed"
fi

# ─────────────────────────────────────────────────────────────────────────────
# 3. Register MCP servers (first install only)
# ─────────────────────────────────────────────────────────────────────────────

if [ "$MODE" != "update" ]; then
    echo ""
    if command -v claude &>/dev/null; then
        echo "[MCP] Registering servers in Claude Code ..."

        if claude mcp list 2>/dev/null | grep -q "^rules:"; then
            echo "  rules-mcp: already registered"
        else
            claude mcp add rules rules-mcp --scope user 2>/dev/null \
                && echo "  rules-mcp: registered" \
                || echo "  [!] could not register rules-mcp" >&2
        fi

        if claude mcp list 2>/dev/null | grep -q "^rulestools:"; then
            echo "  rulestools-mcp: already registered"
        else
            claude mcp add rulestools rulestools-mcp --scope user 2>/dev/null \
                && echo "  rulestools-mcp: registered" \
                || echo "  [!] could not register rulestools-mcp" >&2
        fi
    else
        echo "[MCP] Claude Code CLI not found. Register manually:"
        echo "  claude mcp add rules rules-mcp --scope user"
        echo "  claude mcp add rulestools rulestools-mcp --scope user"
    fi
fi

# ─────────────────────────────────────────────────────────────────────────────
# 4. Summary
# ─────────────────────────────────────────────────────────────────────────────

echo ""
echo "Done."
echo ""
echo "  Python: rulestools  rulestools-mcp  rules-mcp"
echo "  Rust:   file_size   nesting         secrets   rustdocumenter  rustdoc-viewer  rustman"
echo ""
if [ "$MODE" = "update" ]; then
    echo "All components updated from GitHub."
else
    echo "To set up a project:"
    echo "  mcp__rulestools__setup(\"/path/to/project\")"
fi
