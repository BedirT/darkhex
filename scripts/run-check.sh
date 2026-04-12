#!/bin/bash
# Back-pressure wrapper: silent on success, full output on failure.
# This keeps the agent's context clean - passing tests don't waste tokens.
#
# Usage: ./scripts/run-check.sh <command> [args...]
# Example: ./scripts/run-check.sh uv run pytest -x
# Example: ./scripts/run-check.sh uv run ruff check .

OUTPUT=$("$@" 2>&1)
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
  echo "✓ $*"
else
  echo "✗ $*"
  echo "$OUTPUT"
  exit $EXIT_CODE
fi
