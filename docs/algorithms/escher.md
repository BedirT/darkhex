# ESCHER: Eschewing Importance Sampling in Games by Computing a History value function to Estimate Regret

**Paper**: McAleer, S., Farina, G., Lanctot, M. & Brown, N. (2023). ICLR. arXiv:2206.04122.

## Overview

ESCHER eliminates importance sampling entirely from neural CFR. Instead of
using IS-weighted terminal utilities (as DREAM and Deep CFR do), it trains
a history-value network V(h) and uses its output to compute regrets
deterministically. This reduces regret estimator variance by orders of
magnitude on Dark Hex (1.8e-1 vs 2.8e8 for DREAM on DH4).

## Why ESCHER over DREAM

DREAM's outcome sampling uses importance weights `pi_opp / pi_sample` that
grow exponentially with game depth. On 3x3 Dark Hex (~12 moves deep), these
weights can be 1e8+, making advantage network training unstable.

ESCHER avoids this by:
1. Training a value network V(h) on bounded importance-corrected returns
   (bounded because the traverser uses epsilon-greedy, so max IS ratio = 1/epsilon)
2. Computing regret as `Q(h,a) - V(h)` where Q(h,a) = V(child(h,a))
3. No IS weights in the regret computation at all

## Key Equations

### Value Network Training (bounded IS)

Trajectories sampled with epsilon-greedy. Importance correction bounded by 1/epsilon:
```
sample_policy(a) = epsilon * uniform(a) + (1-epsilon) * policy(a)
importance(a) = policy(a) / sample_policy(a)   # bounded by 1/epsilon
```

Returns computed backwards with importance correction:
```
value_target = importance * (returns + value_next)
```

### Regret Computation (no IS)

At each traverser node, query value net for ALL children:
```
Q(h, a) = V(child(h, a))    # value net prediction
V(h) = sum_a policy(a) * Q(h, a)
regret(a) = Q(h, a) - V(h)
```

This is **deterministic** given the value net — no sampling variance.

### LCFR Weighting

Regret network training uses linear CFR weighting:
```
weight = t / T    (current iteration / total iterations)
```

Policy network uses cross-entropy loss (per OpenSpiel reference).

## Architecture

```
ESCHER Solver
├── Value Networks (one per player)
│   ├── MLP: 2 × obs_dim → hidden → 1 (scalar state value)
│   ├── Input: full history (both players' canonical info states)
│   ├── Reset each iteration
│   └── Trained on bounded importance-corrected returns
├── Regret Networks (one per player)
│   ├── MLP: obs_dim → hidden → n_cells
│   ├── Input: single-player canonical info state
│   ├── Reset each iteration
│   └── Trained on value-net-derived regrets with legal action mask
├── Average Policy Network (shared)
│   ├── MLP: obs_dim → hidden → n_cells (logits, softmax at extraction)
│   ├── Cross-entropy loss with LCFR weighting
│   └── Fine-tuned continuously
└── Reservoir Buffers (6 total)
    ├── Value buffers (×2, one per player)
    ├── Regret buffers (×2, one per player)
    └── Policy buffer (×1, shared)
```

### Iteration Flow

```
For each player p:
  1. Gather value data    — epsilon-greedy rollouts, backward returns
  2. Train value net      — MSE on importance-corrected returns
  3. Gather regret data   — traverse with uniform (traverser) / policy (opponent)
                          — compute Q(h,a) = V(child) for ALL legal actions
                          — regret = Q(h,a) - V(h), no IS needed
  4. Train regret net     — masked MSE with LCFR weighting
Train average policy net  — cross-entropy with LCFR weighting
```

## Differences from DREAM

| Aspect | DREAM | ESCHER |
|--------|-------|--------|
| IS weights | Unbounded (explode with depth) | None in regret; bounded in value |
| Regret variance | ~1e8 on DH4 | ~1e-1 on DH4 |
| Networks | 2 (advantage + strategy) | 3 (value + regret + policy) |
| Regret computation | From single OS sample | From value net over ALL children |
| Traverser sampling | Epsilon-greedy | Uniform (no IS correction needed) |
| 3x3 convergence | 0.953 @ 100 iters | **0.020 @ 50 iters** |

## Verified Results

| Board | Iters | Exploitability | Time | vs DREAM |
|-------|-------|----------------|------|----------|
| 2x2 | 30 | 0.005 | 23s | 0.008 |
| 3x3 | 50 | **0.020** | 293s | 0.953 |
| 4x3 | — | Pending | — | — |

## Caveats

1. **ICLR 2026 IIG-RL-Benchmark** (Coste et al.) found ESCHER "uniformly
   noncompetitive" on 3x3 Dark Hex measured by exploitability across 7000
   runs with 350 hyperparameter configs. Generic PG methods (PPO, MMD)
   outperformed it. Our results differ — possibly due to different game
   variants (CDH vs standard) or implementation details.

2. **Training cost**: ESCHER requires training 3 networks per iteration
   vs DREAM's 2. The value network training is the main added cost.

3. **Value network accuracy**: Convergence depends on value net quality.
   Poor value estimates bias the regret estimates (Theorem 1 bound
   degrades linearly with approximation error epsilon).

## References

1. McAleer, S. et al. (2023). ESCHER. ICLR. arXiv:2206.04122.
2. Steinberger, E. et al. (2020). DREAM. arXiv:2006.10410.
3. Brown, N. et al. (2019). Deep CFR. ICML. arXiv:1811.00164.
4. Coste, N. et al. (2026). Reevaluating Policy Gradient Methods for IIG. ICLR. arXiv:2502.08938.
5. OpenSpiel reference: `open_spiel/python/pytorch/escher.py`
