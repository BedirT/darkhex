#!/bin/bash
# Agent session initialization — run at the start of every session.
set -euo pipefail

echo "=== DarkHex Session Init ==="

# Progress file
if [ -f agent-progress.md ]; then
  echo ""
  echo "=== Previous Progress ==="
  cat agent-progress.md
  echo ""
else
  echo "No progress file found (first session)"
fi

# Verify Rust engine is built
echo "=== Build Status ==="
if python3 -c "import darkhex._engine; print('Rust engine: v' + darkhex._engine.__version__)" 2>/dev/null; then
  :
else
  echo "Rust engine NOT built. Run: make dev"
fi

# Git status
echo ""
echo "=== Git Status ==="
git status --short
echo ""
echo "=== Recent Commits ==="
git log --oneline -5

# Obsidian KB check
echo ""
echo "=== Knowledge Base ==="
if [ -f .claude/project-memory/registry.yaml ]; then
  echo "Obsidian KB: bound (iCloud: ~/Library/Mobile Documents/iCloud~md~obsidian/Documents/Research/darkhex/)"
else
  echo "Obsidian KB: not bound"
fi

# Available commands
echo ""
echo "=== Makefile Targets ==="
echo "  make dev        - build Rust extension (debug)"
echo "  make build      - build Rust extension (release)"
echo "  make test       - run all tests (Rust + Python)"
echo "  make check      - lint + all tests"
echo "  make clean      - remove build artifacts"

echo ""
echo "=== Ready to Work ==="
