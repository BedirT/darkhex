# Experiment: EXP-005 — DREAM (Outcome Sampling Neural CFR)

## Hypothesis

**H1**: DREAM (OS traversal + neural function approximation) converges on
2x2 and 3x2 CDH Dark Hex, achieving exploitability comparable to Deep CFR
(ES variant) within matched wall-clock time.

**H2**: DREAM runs to completion on 4x3 CDH (where ES-based Deep CFR is
infeasible) and achieves exploitability < 0.9 within 200 CFR iterations.

**H3**: Q-baseline variance reduction improves convergence rate on 3x3+
boards compared to vanilla DREAM (no baseline).

**Motivation**: Deep CFR with External Sampling cannot complete even one
traversal on 4x3 (>30s). DREAM replaces ES with Outcome Sampling (O(depth)
per traversal), making neural CFR feasible for large boards.

## Setup

- Algorithm: DREAM (Steinberger et al., arXiv:2006.10410, 2020)
- Board sizes: 2x2 (verification), 3x2, 3x3, 4x3 (target)
- Variant: CDH (Classic Dark Hex)
- Parameters per board: see CONFIGS in exp005_dream.py
- Baseline comparison: Tabular OS-MCCFR (EXP-003), Deep CFR ES (EXP-004)

## Conditions

| Condition | Baseline | Description |
|-----------|----------|-------------|
| A: DREAM  | off      | Outcome Sampling + neural nets, no Q-baseline |
| B: DREAM+BL | on    | + learned Q-baseline for variance reduction |

## Evaluation Plan

1. Track exploitability (clairvoyant BR) at each CFR iteration
2. Compare convergence against Deep CFR (ES) at matched iteration counts
3. Compare against tabular OS-MCCFR at matched wall-clock time
4. Measure traversal throughput (traversals/second)
5. Success criteria:
   - 2x2: exploitability < 0.1 within 50 iterations
   - 3x3: exploitability < 0.5 within 100 iterations
   - 4x3: runs to completion AND exploitability decreases monotonically

## Run Instructions

```bash
# Build Rust engine
make dev

# Run per board
uv run python experiments/exp005_dream.py --board 2x2
uv run python experiments/exp005_dream.py --board 3x2
uv run python experiments/exp005_dream.py --board 3x3
uv run python experiments/exp005_dream.py --board 4x3

# Run with Q-baseline
uv run python experiments/exp005_dream.py --board 3x3 --baseline
```

## Results

_(To be filled after experiment runs)_

## References

- Steinberger, E., Lerer, A. & Brown, N. (2020). DREAM. arXiv:2006.10410.
- Brown, N. et al. (2019). Deep CFR. ICML. arXiv:1811.00164.
- McAleer, S. et al. (2022). ESCHER. arXiv:2206.04122.
- Lanctot, M. et al. (2009). Monte Carlo Sampling for Regret Minimization. NeurIPS.
