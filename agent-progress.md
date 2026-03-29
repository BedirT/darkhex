# DarkHex — Agent Progress

## Current Milestone
Modernize codebase and produce publishable paper (target: summer 2026)

## Last Session
- Date: 2026-03-28/29
- Built Rust game engine, MCCFR (External + Outcome Sampling), exhaustive enumeration
- Found and fixed 3 OS-MCCFR importance weight bugs via PR review + literature verification
- Memoized enumeration: 7,000x speedup, 4x3 now feasible (8.5min)
- All 4 thesis info state counts independently confirmed + 2 new board sizes
- PR #18 open: feature/outcome-sampling → re-dev (9 commits, 76 tests)

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
- [x] 76 tests (30 Rust + 46 Python), all passing

## Open PR
- #18: feature/outcome-sampling → re-dev (9 commits, 18 review comments addressed)

## Up Next
- [ ] Exploitability / best response computation (needed to verify Nash convergence)
- [ ] Verify MCCFR strategies against exploitability on 2x2
- [ ] SIP/SIP+ policy simplification (thesis novel contribution)
- [ ] pONE sure-win state database (20% memory savings on 4x3)
- [ ] Isomorphic state reduction (~halves info state count)
- [ ] Run MCCFR on 4x3 (367,919 info states — the thesis headline board)
- [ ] Deep CFR or ReBeL prototype

## Decisions Made
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

## Blockers
- None
