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
- Run ESCHER on 4x3 boards (the main feasibility test, running now)
- Write ESCHER tests (test_escher.py)
- Create ESCHER experiment harness (exp006_escher.py)
- Implement Abstract Best Response (Ab-BR) for apples-to-apples thesis comparison
- Run 10B tabular OS-MCCFR iterations (resume from 1B checkpoint)

## Completed Experiments
- EXP-001: MCCFR convergence verification (done)
- EXP-002: Info state enumeration — all thesis values confirmed (done)
- EXP-003: 4x3 MCCFR convergence — 1B iters, expl 0.989 clairvoyant BR (done)
- EXP-004 (repo): Deep CFR prototype — 2x2 (expl 0.018), 3x2 (expl 0.045), 4x3 ES infeasible (done)
- EXP-005: DREAM — 2x2 (expl 0.008), 3x2 (0.197), 3x3 (0.953 — OS variance too high), 4x3 running
- EXP-006 (pending): ESCHER — 2x2 (expl 0.005), 3x3 (**0.020** — breakthrough), 4x3 running

## Recent Results (2026-04-02)
- ESCHER implemented: IS-free neural CFR with 3-network architecture (darkhex/algorithms/escher.py)
- ESCHER 3x3 CDH: exploitability 1.0 → **0.020** in 50 iters (293s) — breakthrough result
- ESCHER 2x2 CDH: exploitability 1.0 → **0.005** in 30 iters (23s)
- ESCHER eliminates IS weights entirely; regret variance ~1e-1 vs DREAM's ~1e8 on DH4
- DREAM implemented but OS variance too high for 3x3+: 3x3 expl stuck at 0.953
- DREAM 2x2 CDH: expl 0.008 (OK), 3x2: 0.197, 3x3: 0.953 (bad)
- Deep CFR (ES): 2x2 (expl 0.018), 3x2 (expl 0.045), 4x3 infeasible
- Code reviewed by internal agent + Codex; multiple correctness bugs caught and fixed
- ESCHER 4x3 experiment running — the decisive test

## Previous Results (2026-03-30)
- Isomorphic state reduction: 180° rotation symmetry, ~50% info state savings (2x2: 42→22, 3x2: 410→~205)
- pONE belief-space precomputation: probability-1 win state pruning (opt-in, CDH only)
- SIP/SIP+ policy simplification (Rust, thesis §4.4-4.5)
- Exploitability / best response: clairvoyant upper bound, memoized DFS
- 2x2 MCCFR convergence verified: exploitability 0.55 → 0.02 at 50k iters
- Ground truth: 2x2=42 (22 canonical), 3x3=12,556, 4x3=367,919 (~184k canonical)

## Recent Sync Status
- Bootstrap completed at 2026-03-29T01:57:09Z.
- Hub, Plan, Daily, Knowledge notes populated with real project context.
- Updated 2026-04-02 with Deep CFR prototype results.
