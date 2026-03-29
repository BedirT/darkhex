---
project_id: darkhex
repo_root: /Users/bedirt/Documents/Github/darkhex
vault_root: /Users/bedirt/Library/Mobile Documents/iCloud~md~obsidian/Documents/Research/darkhex
hub_note: Research/darkhex/00-Hub.md
language: en
last_sync_at: 2026-03-29T01:57:09Z
last_synced_head: 011c31bb94edfe85926441b90f63ae87daa3bedf
status: active
auto_sync: true
---

# Project Memory: darkhex

## Current Question
Can we improve Dark Hex Nash equilibrium bounds beyond the thesis result (4x3: ε=0.002) using modernized MCCFR with a Rust-speed game engine?

## Hypotheses
- Rust engine gives 50-100x speedup over pyspiel, enabling more MCCFR iterations and tighter bounds
- Outcome Sampling MCCFR will converge on 2x2/3x2 boards within seconds
- Isomorphic reduction will halve effective state space and improve convergence

## Active Tasks
- Port Outcome Sampling MCCFR to Python (replace pyspiel dependency)
- Verify on 2x2 board against known equilibrium
- Add exploitability metric for convergence measurement

## Open Experiments
- EXP-001: MCCFR convergence on 2x2 Dark Hex (baseline verification)
- EXP-002: MCCFR convergence on 3x2 Dark Hex

## Recent Results
- Rust engine implemented: HexBoard + DarkHexState + union-find
- 27 tests passing (11 Rust + 16 Python)
- Full dev toolchain: make check (lint + test)

## Recent Sync Status
- Bootstrap completed at 2026-03-29T01:57:09Z.
- Hub, Plan, Daily, Knowledge notes populated with real project context.
