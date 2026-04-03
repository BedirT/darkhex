# EXP-004: Ab-BR vs Clairvoyant BR Comparison on 4x3 CDH

## Hypothesis

**H1 (Primary):** Ab-BR exploitability on 4x3 CDH at 1B iterations will be
close to the thesis-reported 0.002, confirming the EXP-003 gap (0.989 vs
0.002) was a metric difference, not a convergence failure.

**H3 (Extended):** Training to 10B iterations (10x thesis) will further
tighten Ab-BR exploitability, potentially below 0.001.

**H2 (Metric relationship):** Ab-BR exploitability will be substantially
lower than clairvoyant BR exploitability at every checkpoint, because Ab-BR
constrains the best-response player to act consistently within information
sets.

## Background

EXP-003 showed clairvoyant BR exploitability plateauing at 0.989 on 4x3 CDH
after 1B MCCFR iterations. The thesis reports 0.002 using Abstract Best
Response (Ab-BR), which memoizes on info states instead of full game states.
This experiment computes both metrics side-by-side for direct comparison.

## Setup

- Algorithm: Outcome Sampling MCCFR (epsilon=0.6, seed=42)
- Board: 4x3 CDH (Classic Dark Hex)
- Checkpoints: 1M, 10M, 100M, 500M, 1B, 2B, 5B, 10B iterations
- Metrics: Ab-BR exploitability + clairvoyant BR exploitability at each checkpoint
- Solver checkpoints saved for future reuse

## Known Ground Truth

| Property | Value | Source |
|----------|-------|--------|
| Thesis Ab-BR exploitability | 0.002 | Thesis §4, 1B iters |
| Thesis Ab-BR baseline | 0.156 | Thesis §4, handcrafted |
| EXP-003 clairvoyant BR @ 1B | 0.989 | EXP-003 |

## How to Run

```bash
# Build Rust engine (release mode for performance)
make build

# Fresh run (all checkpoints to 1B)
uv run python experiments/exp004_ab_br_comparison.py

# Resume from a checkpoint
uv run python experiments/exp004_ab_br_comparison.py --resume results/exp004_ab_br/solver_100M.bin

# Custom iteration target
uv run python experiments/exp004_ab_br_comparison.py --max-iters 100000000
```

## Evaluation Plan

1. At each checkpoint, verify both metrics are finite
2. Compare Ab-BR exploitability trajectory to thesis reference (0.002)
3. Compare the gap between Ab-BR and clairvoyant BR across checkpoints
4. Generate dual-line convergence plot for the paper

## Resource Estimates (M-series Mac)

| Phase | Time (est.) |
|-------|------------|
| 1B MCCFR solve | ~2.2 hours |
| 10B MCCFR solve | ~22 hours |
| Ab-BR eval/call | ~2 min |
| Clairvoyant eval/call | ~2 min |
| Full experiment (8 checkpoints) | ~24 hours |
| Checkpoint file size | ~16 MB |

## Outputs

- `convergence.csv`: Per-checkpoint data (both metrics + timing)
- `full_results.json`: Full data including config
- `convergence.png` / `.pdf`: Dual-line log-log convergence plot
- `solver_*.bin`: Solver checkpoints (resume or strategy extraction)

## Preliminary Results (4x3 @ 1B, raw MCCFR strategy)

| Metric | Value | Time |
|--------|-------|------|
| Ab-BR exploitability | 0.722 | 71 min |
| Ab-BR br_black | 0.527 | |
| Ab-BR br_white | 0.918 | |
| Clairvoyant exploitability | 0.989 | 2.9 min |
| Thesis Ab-BR (after SIP) | 0.009 | — |
| Thesis Ab-BR (after SIP+) | 0.002 | — |

Gap vs thesis: our raw MCCFR gives 0.722 under Ab-BR. The thesis applied
SIP+ post-processing BEFORE computing Ab-BR, yielding 0.002. Next step:
apply SIP/SIP+ to our 1B strategy, then recompute Ab-BR.

## Status: running
