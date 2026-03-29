# Monte Carlo Counterfactual Regret Minimization (MCCFR)

## What It Does

Computes approximate Nash equilibrium strategies for Dark Hex by iteratively
minimizing counterfactual regret at each information set. Two sampling variants
are implemented: External Sampling and Outcome Sampling.

## Theoretical Basis

CFR (Zinkevich et al. 2007) converges to Nash equilibrium in two-player
zero-sum games. MCCFR (Lanctot et al. 2009) extends CFR with Monte Carlo
sampling to avoid full tree traversal, making it practical for large games.

- **Regret matching**: At each info set, maintain cumulative regrets per action.
  Current strategy = normalize positive regrets (uniform if all non-positive).
- **Average strategy convergence**: The time-averaged strategy converges to Nash.

### External Sampling

- At update player's nodes: try ALL actions
- At opponent's nodes: sample ONE action from current strategy
- Unbiased regret estimates without importance weights
- Cost per iteration: O(|A|^d) where |A| = branching, d = tree depth

### Outcome Sampling

- At ALL nodes: sample ONE action with epsilon-greedy exploration
- q(a) = eps/|A| + (1-eps)*sigma(a), ensuring every action has nonzero probability
- Uses importance sampling weights: u(z)/pi_sample(z)
- Cost per iteration: O(d) — single path from root to terminal
- Higher variance per iteration but massively cheaper

### References

- Zinkevich, M. et al. (2007). "Regret Minimization in Games with Incomplete Information." NeurIPS.
- Lanctot, M. et al. (2009). "Monte Carlo Sampling for Regret Minimization in Extensive Games." NeurIPS.
- Neller, T. & Lanctot, M. (2013). "An Introduction to Counterfactual Regret Minimization."

## Computational Complexity

| Variant | Time/iteration | Space | Info state coverage |
|---------|---------------|-------|---------------------|
| External | O(\|A\|^d) | O(\|I\|) | Misses zero-prob opponent branches |
| Outcome | O(d) | O(\|I\|) | High (epsilon at update player only; asymptotically full) |

Where |A| = max actions, d = game depth, |I| = info set count.

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `rows` | int | — | Board rows |
| `cols` | int | — | Board columns |
| `sampling` | Sampling | Outcome | External or Outcome |
| `epsilon` | f32 | 0.6 | Exploration for Outcome Sampling |
| `seed` | u64 | 42 | RNG seed for reproducibility |

## How to Run

```python
from darkhex._engine import MCCFRSolver, Sampling

# Outcome Sampling (default, recommended)
solver = MCCFRSolver(3, 3, seed=42)
solver.solve(100000)

# External Sampling (small boards only)
solver = MCCFRSolver(2, 2, Sampling.External, seed=42)
solver.solve(10000)

# Get converged strategy
strategy = solver.get_average_strategy()
for info_state, probs in sorted(strategy.items())[:5]:
    print(f"{info_state}: {probs}")
```

## Ground Truth Verification

Info state counts verified by exhaustive game tree enumeration (`enumerate_game_tree`):

| Board | Exact Count | Method | Time |
|-------|-------------|--------|------|
| 2x2 | **42** (P0=17, P1=25) | Exhaustive DFS | 0.001s |
| 3x2 | **410** (P0=172, P1=238) | Exhaustive DFS | 0.23s |
| 3x3 | **12,556** (P0=6293, P1=6263) | Exhaustive DFS | 6.2h |

All three verified by exhaustive enumeration. 3x3 has 9.47 billion terminal
histories (max depth 17 due to CDH collision retries), explaining the 6.2h
runtime.

## Verified Results (Corrected OS-MCCFR)

Note: Outcome Sampling applies epsilon-greedy exploration ONLY at the update
player's nodes (per OpenSpiel). Opponent nodes sample from sigma directly.
This means coverage is high but not 100% — info states behind zero-probability
opponent actions are not visited. This is correct: those states don't affect
Nash equilibrium computation.

### 2x2 CDH (Outcome Sampling, 100k iterations, 2.3s)

- **Info states**: 42/42 (100% — small enough for full coverage)
- **Black opening**: 50/50 on cells 1 and 2 (anti-diagonal)
- Converged by ~1k iterations

### 3x2 CDH (Outcome Sampling, 100k iterations, 4.2s)

- **Info states**: 396/410 (96.6%)
- **Speed**: ~25k iters/s (31x faster than External's 794 iters/s)

### 3x3 CDH (Outcome Sampling, 500k iterations, 31s)

- **Info states**: 11,308/12,556 (90.1%)
- **Speed**: ~15k iters/s (2,300x faster than External's 6.5 iters/s)
- **3x3 is now practical**: 500k iterations in 31 seconds

### Speed Comparison

| Board | External (it/s) | Outcome (it/s) | Speedup |
|-------|-----------------|----------------|---------|
| 2x2 | 10,800 | 44,000 | 4x |
| 3x2 | 794 | 25,000 | 31x |
| 3x3 | 6.5 | 15,000 | 2,300x |

External Sampling's O(|A|^d) cost is catastrophic on 3x3 (branching 9, depth 9).
Outcome Sampling's O(d) cost makes 3x3 practical.

## Implementation Notes

### Why Outcome Sampling was needed

External Sampling revealed two limitations on 3x3:
1. **Speed**: 7 iters/s — tries all 9 actions at update player root, exponential branching
2. **Coverage**: Missed 3 info states on 2x2 (39 vs 42) because zero-probability
   actions are never sampled at opponent nodes

Outcome Sampling solves both: O(depth) per iteration, and epsilon-greedy
exploration guarantees all actions have nonzero sampling probability.

### Multiple Nash Equilibria

On 2x2, External and Outcome converge to the same Black strategy (unique)
but different White strategies (multiple equilibria). This is expected —
the Nash equilibrium is a set, and different sampling paths find different
points in that set. Both are equally valid.

### Memory

Info state storage: one `Vec<f32>` for regret sums + one for strategy sums per info set.
At 12,556 info states (3x3) with ~9 actions each: ~12,556 * 9 * 2 * 4 bytes ≈ 900 KB.
Memory is not the bottleneck until 4x3+ boards.
