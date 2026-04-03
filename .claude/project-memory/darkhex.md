---
project_id: darkhex
repo_root: /Users/bedirt/Documents/Github/darkhex
vault_root: /Users/bedirt/Library/Mobile Documents/iCloud~md~obsidian/Documents/Research/darkhex
hub_note: Research/darkhex/00-Hub.md
language: en
last_sync_at: 2026-04-02T08:00:00Z
last_synced_head: HEAD
status: active
auto_sync: true
---

# Project Memory: darkhex

## Current Question
Can we improve Dark Hex Nash equilibrium bounds beyond the thesis result (4x3: ε=0.002) using neural approaches (Deep CFR, DREAM) with a Rust-speed game engine?

## Hypotheses
- Rust engine gives 50-100x speedup over pyspiel, enabling more MCCFR iterations and tighter bounds
- Deep CFR neural approximation converges faster than tabular MCCFR by generalizing across similar info states
- External Sampling is infeasible for 4x3+ boards — Outcome Sampling variant (DREAM) needed
- Isomorphic reduction halves effective state space for both tabular and neural approaches

## Active Tasks
- Apply SIP/SIP+ to 1B strategy, then compute Ab-BR (thesis pipeline)
- EXP-004: Run Ab-BR vs clairvoyant comparison on 4x3 CDH to 10B iterations
- Implement DREAM (Outcome Sampling Deep CFR) for 4x3+ boards

## Completed Experiments
- EXP-001: MCCFR convergence verification (done)
- EXP-002: Info state enumeration — all thesis values confirmed (done)
- EXP-003: 4x3 MCCFR convergence — 1B iters, expl 0.989 clairvoyant BR (done)
- EXP-004 (repo): Deep CFR prototype — 2x2 (expl 0.04), 3x2 (expl 0.08), 4x3 ES infeasible (done)

## Recent Results (2026-04-02)
- Ab-BR implemented: thesis three-phase algorithm (reach collection → bucketing → reach-weighted DFS)
  - 4x3 @ 1B: Ab-BR=0.722, clairvoyant=0.989, thesis (SIP+)=0.002
  - Gap vs thesis: raw MCCFR gives 0.722; need SIP+ post-processing for 0.002
- Deep CFR implemented: External Sampling + neural advantage/strategy nets
- 2x2 CDH: exploitability 1.0 → **0.018** in 100 CFR iterations (94s, K=200)
- 3x2 CDH: exploitability 1.0 → **0.045** in 50 CFR iterations (324s, K=200)
- 3x3 CDH: exploitability 1.0 → **0.60** in 30 CFR iterations (668s, K=5)
- 4x3 CDH: External Sampling infeasible — need DREAM (Outcome Sampling variant)
- 100+ tests passing (54 Rust + Python)

## Previous Results (2026-03-30)
- Isomorphic state reduction: 180° rotation symmetry, ~50% info state savings
- pONE belief-space precomputation, SIP/SIP+ policy simplification
- Exploitability / best response: clairvoyant upper bound, memoized DFS
- 2x2 MCCFR convergence verified: exploitability 0.55 → 0.02 at 50k iters
- Ground truth: 2x2=42 (22 canonical), 3x3=12,556, 4x3=367,919 (~184k canonical)

## Recent Sync Status
- Bootstrap completed at 2026-03-29T01:57:09Z.
- Hub, Plan, Daily, Knowledge notes populated with real project context.
- Updated 2026-04-02 with Deep CFR prototype results.
