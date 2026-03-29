# EXP-001: MCCFR Convergence Verification

## Hypothesis
Outcome Sampling MCCFR converges to Nash equilibrium and discovers all
imperfect-recall info states on CDH Dark Hex boards up to 3x3.

## Setup
- Algorithm: Outcome Sampling MCCFR (epsilon=0.6)
- Boards: 2x2, 3x2, 3x3 CDH
- Seed: 42
- Iteration counts: 100 to 500k
- Ground truth: exhaustive enumeration (EXP-002)

## Evaluation Plan
- Compare `solver.num_info_states()` against `enumerate_game_tree()` ground truth
- Track coverage percentage at each iteration count
- Record initial state strategies for convergence analysis

## Results

### Info State Coverage

| Board | Ground Truth | 1k iters | 10k | 100k | 500k |
|-------|-------------|----------|-----|------|------|
| 2x2 | 42 | 42 (100%) | 42 | 42 | — |
| 3x2 | 410 | 398 (97.1%) | 410 (100%) | 410 | — |
| 3x3 | 12,556 | 4,479 (35.7%) | 10,368 (82.6%) | 12,507 (99.6%) | **12,556 (100%)** |

### Speed

| Board | Throughput (it/s) |
|-------|------------------|
| 2x2 | ~44,000 |
| 3x2 | ~25,000 |
| 3x3 | ~15,000 |

### Key Findings
1. Outcome Sampling discovers ALL info states given enough iterations
2. 2x2 reaches 100% at 1k, 3x2 at 10k, 3x3 at 500k
3. Speed is board-size-dependent but practical for all tested sizes
4. Multiple Nash equilibria observed on 2x2 (different P1 strategies from External vs Outcome)

## Data
- `results/exp001_mccfr_verification/convergence.csv`
- `results/exp001_mccfr_verification/full_results.json`

## Status: done

## Next Step
Implement exploitability computation to verify convergence to actual Nash equilibrium
(not just info state coverage).
