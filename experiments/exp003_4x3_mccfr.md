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

_To be filled after running the experiment._

## Status: ready to run
