# Algorithm: Deep CFR

## What It Does

Deep Counterfactual Regret Minimization replaces CFR's tabular regret and average
strategy tables with neural networks. Instead of storing cumulative regrets for
every information state, an advantage network learns to predict regrets from the
info state representation. A separate strategy network learns the average policy
across all CFR iterations.

This enables generalization across similar information states and scales to games
where tabular CFR would require too much memory.

## Theoretical Basis

**Foundation**: CFR with External Sampling (Lanctot et al., 2009) combined with
neural function approximation.

**Key idea**: At each CFR iteration, traverse the game tree via External Sampling.
At traverser nodes, compute instantaneous advantages (regrets) for all actions and
store them in a reservoir-sampled buffer. At opponent nodes, store the current
strategy. Periodically retrain advantage networks from scratch on the buffer, using
LCFR (Linear CFR) weighting where iteration t samples carry weight t.

**Convergence**: Converges to an ε-Nash equilibrium where ε depends on the function
approximation error. As network capacity increases and approximation error → 0,
Deep CFR converges to exact Nash equilibrium at O(1/√T) rate (same as tabular MCCFR).

Theorem 1 (Brown et al., 2019): Total regret bounded by
R^T ≤ O(|I|√(|A|T)) + O(T|I|√(|A|ε_L)) where ε_L is the advantage network's
approximation error.

**References**:
- Brown, Lerer, Gross, & Sandholm. "Deep Counterfactual Regret Minimization."
  ICML 2019. arXiv:1811.00164
- Steinberger. "Single Deep Counterfactual Regret Minimization." arXiv:1901.07621
- Steinberger, Lerer, & Brown. "DREAM." arXiv:2006.10410

## Computational Complexity

- **Time per CFR iteration**: O(K × |A|^d) for External Sampling traversals +
  O(train_steps × batch_size × params) for network training
  - K = traversals per iteration, |A| = max branching factor, d = max depth
- **Space**: O(buffer_size × (input_dim + max_actions)) for reservoir buffers +
  O(params) for networks
- **Practical runtime**:
  - 2x2 CDH: ~0.5s per CFR iteration (100 traversals, 64-hidden MLP)
  - 4x3 CDH: ~TBD

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| rows | int | — | Board rows |
| cols | int | — | Board columns |
| hidden_sizes | tuple | (64, 64) | Hidden layer sizes for advantage/strategy nets |
| lr | float | 1e-3 | Adam learning rate |
| buffer_size | int | 1,000,000 | Reservoir buffer capacity |
| batch_size_advantage | int | 256 | Mini-batch size for advantage training |
| batch_size_strategy | int | 256 | Mini-batch size for strategy training |
| advantage_train_steps | int | 375 | SGD steps per advantage training round |
| strategy_train_steps | int | 2500 | SGD steps per strategy training round |
| num_cfr_iters | int | 100 | Outer CFR iterations |
| num_traversals | int | 375 | Traversals per player per CFR iteration |
| seed | int | 42 | Random seed |
| reinitialize_advantage_networks | bool | True | Reset advantage nets each iteration |

## How to Run

```bash
# Install neural dependencies
uv sync --extra neural

# Build Rust engine
make dev

# Quick 2x2 verification
uv run python -c "
from darkhex.algorithms.deep_cfr import DeepCFR, DeepCFRConfig
cfg = DeepCFRConfig(rows=2, cols=2, num_cfr_iters=20, num_traversals=100)
solver = DeepCFR(cfg)
solver.solve()
print(f'Exploitability: {solver.exploitability():.4f}')
"

# Full experiment
uv run python experiments/exp004_deep_cfr_prototype.py --board 2x2
uv run python experiments/exp004_deep_cfr_prototype.py --board 4x3
```

## Expected Output

**2x2 CDH** (100 CFR iterations, K=200):
- Exploitability drops from 1.0 to **0.018** (94s total)
- Tabular OS-MCCFR reaches 0.0001 at 100k iters — neural is less precise but converges

**3x2 CDH** (50 CFR iterations, K=200):
- Exploitability drops from 1.0 to **0.045** (324s total)

**3x3 CDH** (30 CFR iterations, K=5):
- Exploitability drops from 1.0 to **0.60** — K=5 too few for 6,334 canonical info states
- Each ES traversal takes ~14s; needs DREAM (Outcome Sampling) for efficient convergence

**4x3 CDH**: External Sampling infeasible (>30s per traversal). DREAM required.

## Implementation Notes

- **Advantage net reinitialization**: Networks are retrained from scratch each CFR
  iteration (not fine-tuned). Paper ablation shows this avoids catastrophic forgetting
  and produces ~50% lower exploitability.
- **Argmax tiebreaker**: When all predicted advantages are ≤ 0, the action with the
  highest raw advantage is played deterministically (not uniform). Paper ablation
  shows this handles approximation error better.
- **LCFR weighting**: Implemented via sqrt(t) trick — multiplying both prediction and
  target by sqrt(iteration) before MSE gives effective weight t on each sample.
- **Reservoir sampling**: Critical choice over sliding window. Paper ablation shows
  sliding window stops converging once buffer is full.
- **Isomorphic reduction**: Info states are canonicalized via 180° rotation before
  encoding, halving the effective state space.
- **Strategy samples at opponent nodes**: During traversal, strategy vectors are
  collected at opponent nodes (not a separate pass).
- **LayerNorm**: Applied on last hidden layer before output (per paper and OpenSpiel).
