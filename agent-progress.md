# DarkHex — Agent Progress

## Current Milestone
Modernize codebase and produce publishable paper (target: summer 2026)

## Last Session
- Date: 2026-03-30
- EXP-003: MCCFR on 4x3 CDH (thesis headline board)
  - Vanilla: exploitability ~0.998 at 10M iterations — NOT converged
  - 162,528 canonical info states discovered (88% of ~184k expected)
  - Throughput: ~12–15k iter/s, exploitability ~11 min/call
  - pONE condition INVALID: bug prunes entire tree (see Failed Approaches)
- pONE bug discovered: per-config minimax ≠ belief-space minimax
  - ∀config ∃strategy (implemented) vs ∃strategy ∀config (required)
  - False positives on non-square boards (rows > cols → P1 root flagged)
  - Affects 3x2, 4x3; does NOT affect 2x2, 3x3, 2x3, 3x4

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
- [x] EXP-003: 4x3 MCCFR convergence (vanilla valid, pONE invalid due to bug)

## Open PR
- #18: merged (feature/outcome-sampling → re-dev)

## Up Next
- [ ] Fix pONE bug: replace per-config minimax with belief-space search
- [ ] Scale MCCFR: 100M+ iterations on 4x3, or switch to External Sampling with variance reduction
- [ ] SIP/SIP+ policy simplification (thesis novel contribution)
- [ ] Deep CFR or ReBeL prototype (neural approach may converge faster on 4x3)

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
- pONE per-config minimax: checks ∀config ∃strategy instead of ∃strategy ∀config. False positives on non-square boards (3x2, 4x3). Prunes entire tree when P1 root is falsely flagged (2026-03-30)
- OS-MCCFR 10M iters on 4x3: exploitability stuck at ~0.998. Game tree has 31.9M states; one-path sampling gives inadequate coverage (2026-03-30)

## Decisions Made (continued)
- Isomorphic reduction always on (no opt-out), canonical = lex-smaller of original/rotated (2026-03-29)
- pONE is opt-in via set_pone_db() — game logic untouched (2026-03-29)
- pONE uses belief-space check per thesis §4.2, NOT regular Hex minimax (2026-03-29)
- Fixed strategy-action index bug: get_average_strategy returns cell indices not sequential (2026-03-29)

## Blockers
- None
