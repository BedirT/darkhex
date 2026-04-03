# DarkHex — Agent Progress

## Current Milestone
Modernize codebase and produce publishable paper (target: summer 2026)

## Last Session
- Date: 2026-04-02
- Implemented thesis-style Ab-BR + Deep CFR prototype
  - Ab-BR: three-phase algorithm matching thesis code (reach collection → bucketing → reach-weighted DFS)
  - 4x3 @ 1B: Ab-BR=0.722, clairvoyant=0.989, thesis (SIP+)=0.002
  - Gap vs thesis: raw MCCFR gives 0.722; need SIP+ post-processing for 0.002
  - Deep CFR: 2x2 expl 0.018, 3x2 expl 0.045, 3x3 expl 0.60, 4x3 ES infeasible
  - Next: DREAM (OS variant) for 4x3+, SIP+ → Ab-BR pipeline

## Completed
- [x] Rust game engine: HexBoard (union-find), DarkHexState (CDH/ADH/NDH/FDH)
- [x] External Sampling MCCFR (pure Rust, no PyO3 in hot loop)
- [x] Outcome Sampling MCCFR (corrected per OpenSpiel, 2,300x faster on 3x3)
- [x] Exhaustive info state enumeration with memoization
- [x] Ground truth: 2x2=42, 2x3=314, 3x2=410, 3x3=12,556, 4x3=367,919, 3x4=341,033
- [x] Experiment framework: experiments/ + results/ with CSV/JSON
- [x] EXP-001: MCCFR convergence verification
- [x] EXP-002: Info state enumeration (all thesis values confirmed)
- [x] src/ restructured: game/ + solver/ with separate test files
- [x] Architecture: Rust for tabular algos + VecEnv, Python for neural
- [x] Obsidian project KB with Experiments/ and Results/ notes
- [x] Harness: init.sh, AGENTS.md (git workflow, reasoning chains), hooks, rules
- [x] Exploitability / best response (clairvoyant upper bound, memoized DFS)
- [x] 2x2 MCCFR convergence verified via exploitability (0.55 → 0.02 at 50k iters)
- [x] 91 tests (37 Rust + 54 Python), all passing
- [x] Isomorphic state reduction (180° rotation, ~50% info state savings)
- [x] pONE belief-space precomputation (probability-1 win state pruning)
- [x] Fixed strategy-action index bug in get_average_strategy()
- [x] 117 tests (54 Rust + 63 Python), all passing
- [x] EXP-003: 4x3 MCCFR convergence — 1B iters, expl 0.989 (clairvoyant BR)
- [x] Deep CFR prototype: External Sampling + neural advantage/strategy nets
- [x] EXP-004: Deep CFR on 2x2 (expl 0.018), 3x2 (expl 0.045), 3x3 (expl 0.60)
- [x] Codex review fixes: LCFR counting, seeded buffers, renormalization, 3x3 config
- [x] canonical_info_state() exposed in Python bindings
- [x] PyTorch optional dependency + 22 Deep CFR tests
- [x] pONE AND-OR fix: belief-space search replaces per-config minimax
- [x] pONE MCCFR pruning disabled (causes missing strategies in subtrees)
- [x] SIP/SIP+ policy simplification (Rust, thesis §4.4–4.5)
- [x] Solver checkpointing: save/load/resume via bincode (~16 MB for 4x3)
- [x] Abstract Best Response (Ab-BR) — Rust + PyO3, thesis §4 three-phase algorithm
- [x] Ab-BR verified on 4x3 @ 1B: 0.722 (vs clairvoyant 0.989, thesis 0.002 with SIP+)
- [x] EXP-004 prepared: Ab-BR vs clairvoyant comparison, 4x3 CDH to 10B

## Open PR
- #18: merged (feature/outcome-sampling → re-dev)

## Up Next
- [ ] Apply SIP/SIP+ to 1B strategy, then compute Ab-BR (thesis pipeline: MCCFR → SIP+ → Ab-BR)
- [ ] Implement DREAM (Outcome Sampling Deep CFR) for 4x3+ boards
- [ ] Run EXP-004: 10B iterations with dual Ab-BR + clairvoyant metrics
- [ ] Optimize Ab-BR Phase 1 — opponent node memoization or DAG-based reach propagation

## Decisions Made
- Ab-BR is a heuristic, NOT a guaranteed bound on clairvoyant BR — can be higher or lower (2026-04-01)
- Clairvoyant BR (per-state optimal) for exploitability — upper bound, tight for converged strategies (2026-03-29)
- CDH default, all 4 variants supported (2026-03-28)
- External → Outcome Sampling (External too slow for 3x3) (2026-03-28)
- OS-MCCFR corrected: epsilon only at update player, raw utility at terminal (2026-03-29)
- Memoized enumeration: full game state key for dedup (2026-03-29)
- Rust for tabular algos, Python for neural via VecEnv batching (2026-03-28)
- Memory is hard constraint: f32, lazy alloc, pONE, isomorphic (2026-03-28)
- Git: re-dev = develop, feature branches PR back (2026-03-28)
- No AI co-author in commits (2026-03-28)
- Rust tests in separate _tests.rs files, never inline (2026-03-28)
- src/ organized: game/ (foundation) + solver/ (algorithms) (2026-03-28)

## Failed Approaches
- Deep CFR External Sampling on 4x3: single traversal >30s, exponential in branching factor. ES explores ALL actions at traverser nodes. 3x3 = 14.4s/traversal, 4x3 = infeasible. Need Outcome Sampling variant (DREAM) for 4x3+ (2026-04-02)
- External Sampling on 3x3: 7 iters/s, exponential in branching factor
- OS-MCCFR with epsilon at all nodes: biased importance weights (3 bugs)
- Naive DFS enumeration: 6.2h on 3x3 (9.47B terminals)
- pONE per-config minimax: checks ∀config ∃strategy instead of ∃strategy ∀config. False positives on non-square boards (3x2, 4x3). Prunes entire tree when P1 root is falsely flagged (2026-03-30)
- OS-MCCFR 10M iters on 4x3: exploitability stuck at ~0.998. Game tree has 31.9M states; one-path sampling gives inadequate coverage (2026-03-30)

## Decisions Made (continued)
- Isomorphic reduction always on (no opt-out), canonical = lex-smaller of original/rotated (2026-03-29)
- pONE is opt-in via set_pone_db() — game logic untouched (2026-03-29)
- pONE uses belief-space check per thesis §4.2, NOT regular Hex minimax (2026-03-29)
- Fixed strategy-action index bug: get_average_strategy returns cell indices not sequential (2026-03-29)

## Blockers
- None
