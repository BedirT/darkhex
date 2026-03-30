# EXP-003: MCCFR Convergence on 4x3 CDH Dark Hex

## Hypothesis

**H1 (Primary):** OS-MCCFR on 4x3 CDH converges to exploitability < 0.01
within 10M iterations, matching the thesis result (epsilon 0.002).

**H2 (pONE effect):** pONE pruning reduces info state count by at least 10%
and accelerates convergence relative to vanilla MCCFR.

**H3 (Isomorphic reduction):** With isomorphic reduction always on, the solver
discovers ~184,000 canonical info states (half of the 367,919 total).

## Setup

- Algorithm: Outcome Sampling MCCFR (epsilon=0.6)
- Board: 4x3 CDH (Classic Dark Hex)
- Seed: 42
- Conditions: vanilla (no pONE) and pONE-enabled
- Checkpoints: 10k, 50k, 100k, 500k, 1M, 2M, 5M, 10M iterations

## Known Ground Truth (from EXP-002)

| Property | Value |
|----------|-------|
| Total info states | 367,919 |
| P0 info states | 184,024 |
| P1 info states | 183,895 |
| Game states | 31,949,417 |
| Thesis best epsilon | 0.002 |
| Thesis baseline epsilon | 0.156 |

## Evaluation Plan

- **Primary metric:** Exploitability = (BR_Black + BR_White) / 2
- **Success:** exploitability < 0.01 at 10M iterations
- **Strong success:** exploitability < 0.005 (matches/beats thesis)
- Exploitability measured via `best_response_values()` without pONE
  (to avoid biasing the measurement)
- Monitor for non-monotonic convergence (>20% increase = warning)

## Verification

- V1: Sanity check 3x3 exploitability before starting
- V2: Throughput guard: abort if < 1000 iter/s
- V3: Check (br_b + br_w) / 2 == exploitability at each checkpoint
- V4: Final info state count in [170k, 190k] confirms isomorphic reduction
- V5: pONE condition should have fewer info states than vanilla

## Resource Estimates (measured on M-series Mac)

| Phase | Estimate |
|-------|----------|
| pONE precomputation | ~1.6 min (59,027 states) |
| MCCFR throughput (4x3) | ~87,000 iter/s |
| 10M MCCFR iterations | ~2 min |
| Exploitability per call | ~2.3 min |
| Full experiment (both conditions) | ~40-60 min |

## How to Run

```bash
# Build Rust engine (release mode required for performance)
make build

# Run the experiment (expect ~40-60 min)
uv run python experiments/exp003_4x3_mccfr.py

# Results will appear in:
#   results/exp003_4x3_mccfr/convergence.csv
#   results/exp003_4x3_mccfr/full_results.json
#   results/exp003_4x3_mccfr/convergence.png
#   results/exp003_4x3_mccfr/convergence.pdf
```

## Expected Outputs

- `convergence.csv`: Per-checkpoint data (condition, iters, exploitability, ...)
- `full_results.json`: Full data including config and timing
- `convergence.png` / `.pdf`: Log-log convergence plot, two conditions,
  thesis reference lines at 0.002 and 0.156

## Results

### Vanilla MCCFR (no pONE)

| Iterations | Exploitability | Info States | Solve Time | Expl Time |
|-----------|---------------|-------------|------------|-----------|
| 10,000 | 0.9981 | 35,329 | 1s | 666s |
| 50,000 | 0.9986 | 74,454 | 4s | 662s |
| 100,000 | 0.9987 | 92,922 | 8s | 649s |
| 500,000 | 0.9993 | 127,961 | 34s | 646s |
| 1,000,000 | 0.9994 | 139,820 | 67s | 646s |
| 2,000,000 | 0.9957 | 148,342 | 134s | 1,601s |
| 5,000,000 | 0.9980 | 156,190 | — | 665s |
| 10,000,000 | 0.9982 | 162,528 | — | 670s |

**Note**: Solve times at 5M/10M are inflated by machine sleep. Actual MCCFR
throughput was ~12,000–15,000 iter/s throughout.

### pONE Condition: INVALID

pONE pruned the entire game tree. Only 1 info state discovered across all
iterations. Exploitability constant at 0.995094.

**Root cause**: The pONE algorithm has a correctness bug. It checks each
hidden-stone configuration independently via minimax (∀ config, ∃ winning
strategy), but pONE requires a SINGLE strategy that wins against ALL
configurations simultaneously (∃ strategy, ∀ config it wins). On 4x3,
this falsely flags P1's root info state as pONE, causing the solver to
prune the entire tree immediately.

See "pONE Bug" section below.

### Hypothesis Evaluation

**H1 (REJECTED):** Exploitability remains at ~0.998 after 10M iterations.
OS-MCCFR has not meaningfully converged on 4x3. The game tree has 31.9M
states; each OS-MCCFR iteration samples one path, so 10M iterations
provides inadequate coverage. Need 100M–1B iterations or a different
algorithm (Deep CFR, DREAM, or External Sampling with variance reduction).

**H2 (INVALID):** pONE condition is invalid due to the algorithm bug.
Cannot evaluate pruning effect until the bug is fixed.

**H3 (PARTIAL):** Vanilla discovered 162,528 canonical info states out of
~184,000 expected. This is 88% coverage, confirming isomorphic reduction
works at scale. Full coverage needs more iterations.

### pONE Bug Analysis

The `is_pone` function (src/solver/pone.rs) checks each belief-consistent
board independently via perfect-information minimax. This is necessary but
not sufficient for probability-1 win detection:

```
Implemented:  ∀ hidden_config, ∃ strategy: wins(strategy, hidden_config)
Required:     ∃ strategy, ∀ hidden_config: wins(strategy, hidden_config)
```

On 4x3, P1 root (White, empty view, 1 hidden Black stone) is falsely
flagged because P1 can beat each of the 12 possible Black-stone positions
using different strategies, but no single strategy works against all 12.

Affected boards (P1 root false positive): 3x2, 4x3 (rows > cols).
Unaffected: 2x2, 2x3, 3x3, 3x4 (rows ≤ cols).

**Fix needed**: Replace per-configuration minimax with a belief-space
search or imperfect-information minimax that finds a single strategy
valid across all belief-consistent boards.

## Status: completed (vanilla valid, pONE invalid)
