# DarkHex — Agent Progress

## Current Milestone
Modernize codebase and produce publishable paper (target: summer 2026)

## Last Session
- Date: 2026-03-29
- Isomorphic state reduction: 180° rotation symmetry, ~50% info state savings
  - 2x2: 42 → 22 canonical, 3x2: 410 → ~205
  - Integrated into MCCFR, exploitability, enumeration
- pONE (probability-1 win states): belief-space precomputation per thesis §4.2
  - Precomputes info states where player wins regardless of hidden stone placement
  - Optional opt-in via set_pone_db() / pone_db kwarg
  - Game logic completely unaffected
- Fixed strategy-action index bug: get_average_strategy() now returns actual cell indices
- 54 Rust + 63 Python = 117 tests passing

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

## Open PR
- #18: merged (feature/outcome-sampling → re-dev)

## Up Next
- [ ] Run MCCFR on 4x3 (~184,000 canonical info states — the thesis headline board)
- [ ] SIP/SIP+ policy simplification (thesis novel contribution)
- [ ] Deep CFR or ReBeL prototype

## Decisions Made
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

## Decisions Made (continued)
- Isomorphic reduction always on (no opt-out), canonical = lex-smaller of original/rotated (2026-03-29)
- pONE is opt-in via set_pone_db() — game logic untouched (2026-03-29)
- pONE uses belief-space check per thesis §4.2, NOT regular Hex minimax (2026-03-29)
- Fixed strategy-action index bug: get_average_strategy returns cell indices not sequential (2026-03-29)

## Blockers
- None
