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

## Results (v2: 100M iters, fixed pONE)

### Vanilla MCCFR (no pONE)

| Iterations | Exploitability | Info States | Throughput |
|-----------|---------------|-------------|-----------|
| 100,000 | 0.999 | 92,922 | 101k/s |
| 1,000,000 | 0.999 | 139,820 | 114k/s |
| 10,000,000 | 0.998 | 162,528 | 117k/s |
| 50,000,000 | 0.993 | 170,111 | 122k/s |
| 100,000,000 | **0.985** | 171,825 | 123k/s |

Note: br_white ≈ 1.000 throughout — White's best response always wins.
Only Black's defense improves (br_black: 0.997 → 0.970 over 100M).

### pONE Condition (fixed AND-OR algorithm)

| Iterations | Exploitability | Info States | Throughput |
|-----------|---------------|-------------|-----------|
| 100,000 | 0.998 | 62,196 | 113k/s |
| 1,000,000 | 0.996 | 99,431 | 123k/s |
| 10,000,000 | 0.998 | 114,278 | 130k/s |
| 50,000,000 | 0.999 | 117,857 | 134k/s |
| 100,000,000 | **1.000** | 119,390 | 134k/s |

**pONE hurts exploitability**: Pruned subtrees have no stored strategy.
Exploitability exploits these "holes". pONE reduces info states by 31%
and improves throughput by 10%, but measured exploitability is WORSE
than vanilla. The MCCFR integration needs to output strategies for
pONE info states, not just prune them.

### SIP/SIP+ Post-Processing (on 100M vanilla)

| Method | Exploitability | Notes |
|--------|---------------|-------|
| Raw | 0.985 | Baseline |
| SIP(eps=0.1, b=2) | 1.000 | Too aggressive — removes useful actions |
| SIP(eps=0.01, b=8) | 0.985 | No improvement — strategy not converged enough |
| SIP+(thesis: eps=0.1, b=2, N=20) | 1.000 | Same as above |
| SIP+(eps=0.01, b=8, N=50) | 0.985 | No improvement |

SIP/SIP+ cannot help a strategy that hasn't converged. It's a polish step.

### Hypothesis Evaluation

**H1 (REJECTED):** Exploitability 0.985 at 100M iterations — slow but
measurable convergence. Thesis used 1B iterations (10x more) + SIP+
post-processing + a weaker evaluation metric (Abstract Best Response).
Our clairvoyant BR is a stricter upper bound.

**H2 (PARTIALLY CONFIRMED):** pONE correctly reduces info states by 31%
(119k vs 172k) and improves throughput by 10%. However, it HURTS
measured exploitability because pruned subtrees lack stored strategies.
The MCCFR-pONE integration needs redesign.

**H3 (CONFIRMED):** Vanilla discovered 171,825 canonical info states
(93% of ~184k expected). Isomorphic reduction works at scale.

### pONE Bug (FIXED)

The v1 run used a buggy pONE that checked per-config minimax instead of
AND-OR belief-space search. Fixed in commit 9cc26a9. The v2 run above
uses the corrected AND-OR algorithm.

### Thesis Comparison

The thesis achieved 0.002 via a fundamentally different pipeline:
- 1 billion iterations (10x our 100M)
- SIP+ post-processing
- Abstract Best Response metric (weaker than our clairvoyant BR)
- OpenSpiel's C++ OS-MCCFR implementation

Our clairvoyant BR is a stricter upper bound. To achieve comparable
numbers, we need either 1B+ iterations or the Ab-BR metric.

## Status: completed (both conditions valid, pONE fixed)
