# DarkHex — Agent Progress

## Current Milestone
Modernize codebase and produce publishable paper (target: summer 2026)

## Last Session
- Date: 2026-04-02
- Implemented thesis-style Abstract Best Response (Ab-BR) in Rust + PyO3
  - Three-phase algorithm: (1) collect opponent reach probs, (2) bucket by info set, (3) reach-weighted value DFS
  - Memoized on info_state(P0)+info_state(P1) for Phase 3, state_key for Phase 1 BR nodes
  - 4x3 @ 1B iters: **Ab-BR=0.722** vs clairvoyant=0.989 (71 min, ~10 GB peak)
  - 2x2: Ab-BR=0.231 vs clairvoyant=0.001
  - Thesis reported 0.009 (SIP) / 0.002 (SIP+) — our raw MCCFR gives 0.722, needs SIP+ post-processing
- Key discoveries:
  - Clairvoyant BR (our original) gives the BR player full board visibility — valid upper bound but not thesis metric
  - Ab-BR is a heuristic with reach weighting from thesis §4 — NOT a formal lower bound
  - True BR (info-set-consistent) requires history enumeration — infeasible for 4x3
  - The thesis 0.002 was SIP+ post-processed, not raw MCCFR output
- Fixed pre-existing duplicate methods in mccfr.rs, lint errors in exp003

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
- [ ] Run EXP-004: 10B iterations with dual Ab-BR + clairvoyant metrics
- [ ] Optimize Ab-BR Phase 1 — opponent node memoization or DAG-based reach propagation
- [ ] Deep CFR or ReBeL prototype (neural approach may converge faster on 4x3)
- [ ] PR: merge Ab-BR + EXP-004 → re-dev

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
