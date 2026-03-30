---
project_id: darkhex
repo_root: /Users/bedirt/Documents/Github/darkhex
vault_root: /Users/bedirt/Library/Mobile Documents/iCloud~md~obsidian/Documents/Research/darkhex
hub_note: Research/darkhex/00-Hub.md
language: en
last_sync_at: 2026-03-29T22:00:00Z
last_synced_head: 1e03d9c
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
- Run MCCFR on 4x3 (~184,000 canonical info states — thesis headline board)
- SIP/SIP+ policy simplification (thesis novel contribution)

## Completed Experiments
- EXP-001: MCCFR convergence verification (done)
- EXP-002: Info state enumeration — all thesis values confirmed (done)
- EXP-003: Exploitability convergence on 2x2 (verified: 0.55 → 0.02)

## Recent Results
- Isomorphic state reduction: 180° rotation symmetry, ~50% info state savings (2x2: 42→22, 3x2: 410→~205)
- pONE belief-space precomputation: probability-1 win state pruning (opt-in, CDH only)
- Fixed strategy-action index bug in get_average_strategy() (was returning sequential indices)
- 117 tests passing (54 Rust + 63 Python)
- Exploitability / best response: clairvoyant upper bound, memoized DFS
- 2x2 MCCFR convergence verified: exploitability 0.55 → 0.02 at 50k iters
- Ground truth: 2x2=42 (22 canonical), 3x3=12,556, 4x3=367,919 (~184k canonical)

## Recent Sync Status
- Bootstrap completed at 2026-03-29T01:57:09Z.
- Hub, Plan, Daily, Knowledge notes populated with real project context.
