# DarkHex — Agent Progress

## Current Milestone
Modernize codebase and produce publishable paper (target: summer 2026)

## Last Session
- Date: 2026-03-28
- Implemented Rust game engine (CDH + all 4 variants) + External Sampling MCCFR
- Investigated results on 2x2/3x2/3x3: External Sampling too slow for 3x3 (7 iters/s)
- Root cause: tries all actions at update player nodes → exponential branching
- Also: misses info states behind zero-probability actions (39 vs 42 on 2x2)
- Both issues resolved by Outcome Sampling MCCFR → implementing now

## Completed (recent)
- [x] Rust game engine: HexBoard (union-find), DarkHexState (CDH/ADH/NDH/FDH)
- [x] External Sampling MCCFR in pure Rust (no PyO3 in hot loop)
- [x] Verified 2x2 (39 info states, converges by 1k), 3x2 (385, ~800 iters/s)
- [x] Identified External Sampling limitation on 3x3 (7 iters/s, missing info states)
- [x] Architecture: Rust for tabular algos + VecEnv, Python for neural
- [x] Memory optimization as hard constraint (f32, integer keys, lazy alloc)
- [x] Obsidian project KB + full documentation (ARCHITECTURE, DECISIONS, algorithm docs)
- [x] Harness: init.sh, hooks, rules, git branching strategy (re-dev → feature/)
- [x] Thesis deep-dive + literature survey (ReBeL, AlphaZe**, Deep CFR, etc.)

## In Progress
- [ ] Outcome Sampling MCCFR (feature/outcome-sampling branch)
  - O(depth) per iteration instead of O(branching^depth)
  - Epsilon-greedy exploration ensures full info state coverage
  - Should hit thesis numbers: 42 (2x2), 410 (3x2), 12,556 (3x3)

## Up Next
- [ ] Exploitability / best response computation
- [ ] Verify MCCFR on 2x2 against known Nash equilibrium
- [ ] SIP/SIP+ policy simplification
- [ ] pONE sure-win state database
- [ ] VecEnv for batched game stepping
- [ ] Deep CFR or ReBeL prototype

## Decisions Made
- CDH is the default Dark Hex variant — player retries after collision (2026-03-28)
- External Sampling MCCFR: too slow for 3x3 → need Outcome Sampling (2026-03-28)
- Info state gap (39 vs 42) caused by zero-prob actions in External Sampling (2026-03-28)
- Rust for tabular algos, Python for neural (via VecEnv batching) (2026-03-28)
- Memory is a hard constraint: f32 regrets, lazy alloc, pONE pruning (2026-03-28)
- Git: re-dev = develop, feature branches PR back, reasoning-chain commits (2026-03-28)
- Thesis algorithms: MCCFR, NFSP, SIP/SIP+, pONE, Ab-BR, ABR (2026-03-28)
- Top candidates beyond thesis: ReBeL, AlphaZe**, Deep CFR, R-NaD (2026-03-28)

## Failed Approaches
- External Sampling MCCFR on 3x3: 7 iters/s, exponential in branching factor

## Blockers
- None
