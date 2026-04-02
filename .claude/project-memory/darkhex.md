---
project_id: darkhex
repo_root: /Users/bedirt/Documents/Github/darkhex
vault_root: /Users/bedirt/Library/Mobile Documents/iCloud~md~obsidian/Documents/Research/darkhex
hub_note: Research/darkhex/00-Hub.md
language: en
last_sync_at: 2026-04-01T23:00:00Z
last_synced_head: 17a3be4
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
- EXP-004: Run Ab-BR vs clairvoyant comparison on 4x3 CDH to 10B iterations
- Analyze Ab-BR convergence — compare against thesis 0.002 at 1B
- Deep CFR or ReBeL prototype (neural approach)

## Completed Experiments
- EXP-001: MCCFR convergence verification (done)
- EXP-002: Info state enumeration — all thesis values confirmed (done)
- EXP-003: 4x3 MCCFR convergence — 1B iters, expl=0.989 (clairvoyant BR) (done)

## Recent Results
- Ab-BR implemented in Rust + PyO3 (2026-04-01)
  - Key finding: Ab-BR is NOT a bound on clairvoyant BR; different metric entirely
  - On trained 2x2: Ab-BR=0.082 vs clairvoyant=0.001 (Ab-BR can be higher near equilibrium)
- EXP-003: 4x3 MCCFR @ 1B iters → expl 0.989 (clairvoyant BR), 175k canonical info states
- Ground truth: 2x2=42 (22 canonical), 3x3=12,556, 4x3=367,919 (~184k canonical)

## Recent Sync Status
- Bootstrap completed at 2026-03-29T01:57:09Z.
- Hub, Plan, Daily, Knowledge notes populated with real project context.
