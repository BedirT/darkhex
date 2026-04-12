# DREAM: Deep Regret minimization with Advantage baselines and Model-free learning

**Paper**: Steinberger, E., Lerer, A. & Brown, N. (2020). arXiv:2006.10410.

## Overview

DREAM is a neural CFR variant that replaces External Sampling with Outcome
Sampling, reducing per-traversal cost from O(|A|^depth) to O(depth). This
makes neural function approximation feasible for large imperfect-information
games where External Sampling (Deep CFR) is computationally infeasible.

## Key Equations

### Exploration Policy (traverser)

```
q(s_i, a) = ε/|A(s_i)| + (1-ε) · σ^t(s_i, a)
```

Non-traverser samples from current policy σ^t directly.

### Baseline-Corrected Child Values (Eq. 9, Schmid et al. '19)

For sampled action a' with sampling probability q(a'):
```
v(h, a) = {
    baseline(s*, a) + [v_child - baseline(s*, a)] / q(a)   if a = a'  (sampled)
    baseline(s*, a)                                         if a ≠ a'  (not sampled)
}
```

Without baseline (Phase 1): baseline = 0, so non-sampled actions contribute 0.

### Counterfactual Advantage

```
d(s_i, a) = cf_prefix · v(h, a) - cf_prefix · Σ_a' σ(a') · v(h, a')
```

where `cf_prefix = π_opp / π_sample` (importance weight).

### LCFR Weighting

Network training uses linear CFR weighting via the sqrt(t) trick:
```
L = MSE(√t · f(x), √t · y) = t · MSE(f(x), y)
```

## Architecture

```
DREAM Solver
├── Advantage Networks (one per player)
│   ├── MLP: input_dim → hidden → n_cells
│   ├── Regret matching on output
│   └── Reset every N iterations (default: 10)
├── Strategy Network (shared)
│   ├── MLP: input_dim → hidden → n_cells + Softmax
│   └── Fine-tuned continuously (no reset)
├── Reservoir Buffers
│   ├── Advantage buffers (one per player)
│   └── Strategy buffer (shared)
└── Q-Baseline Networks (optional, one per player)
    ├── MLP: 2 × input_dim → hidden → n_cells
    ├── Input: concatenated info states of both players
    └── Trained with expected SARSA targets
```

## Differences from Deep CFR

| Aspect | Deep CFR | DREAM |
|--------|----------|-------|
| Sampling | External (ALL traverser actions) | Outcome (ONE action at ALL nodes) |
| Complexity/traversal | O(\|A\|^depth) | O(depth) |
| Importance weights | Not needed | Required (π_opp/π_sample) |
| Adv net reset | Every iteration | Every N iterations |
| Variance | Low (full-width) | High (single sample) → mitigated by baseline |
| 4x3 feasibility | No (>30s/traversal) | Yes (~ms/traversal) |

## Implementation

File: `darkhex/algorithms/dream.py`

Reuses from `darkhex/algorithms/deep_cfr.py`:
- `encode_info_state()` — player-relative board encoding
- `ReservoirBuffer` — Vitter reservoir sampling
- `MLP` — network architecture with LayerNorm

## References

1. Steinberger et al. (2020). DREAM. arXiv:2006.10410.
2. Brown et al. (2019). Deep CFR. ICML. arXiv:1811.00164.
3. Lanctot et al. (2009). Monte Carlo Sampling for Regret Minimization. NeurIPS.
4. Schmid et al. (2019). Variance Reduction in Monte Carlo CFR. AAAI.
5. McAleer et al. (2022). ESCHER. arXiv:2206.04122.
