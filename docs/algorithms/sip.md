# Algorithm: SIP / SIP+ (Simplified Policy)

## What It Does

SIP is a post-processing algorithm that simplifies MCCFR-generated strategies by pruning low-probability actions and renormalizing. SIP+ extends SIP by rounding probabilities to simple fractions (e.g., 1/2, 1/3, 2/3), producing more interpretable strategies.

The key insight: MCCFR explores many marginal actions during convergence, producing small non-zero probabilities that are strategic noise. Removing this noise simplifies the strategy without sacrificing (and sometimes improving) game-theoretic quality.

## Theoretical Basis

SIP/SIP+ are novel contributions from the thesis (Chapter 4, Sections 4.4-4.5). They are referenced as SIMCAP/SIMCAP+ in some thesis sections.

**SIP algorithm** (per info state):
1. **Filter**: keep actions with probability > epsilon
2. **Fallback**: if no actions pass, keep highest-probability action at 1.0
3. **Cap**: keep top `action_cap` actions by probability
4. **Normalize**: rescale remaining probabilities to sum to 1.0

**SIP+ additional step**:
5. For each action probability, find the closest fraction p/q (q <= frac_limit) within eta distance
6. Accept fractionized version only if ALL actions match AND fractions sum to 1.0

Reference: Bedirhan Tunckanat, MSc Thesis, University of Alberta, 2023. Chapter 4: "Self-Learning Players: Reinforcement Learning and Regret Based Methods"

## Computational Complexity

- **Time**: O(|I| * k * log(k)) where |I| = number of info states, k = max actions per state
- **Space**: O(|I| * k) for the output strategy (same as input)
- **SIP+ addition**: O(|I| * k * log(F)) where F = number of unique fractions (for binary search)
- **Practical runtime**: negligible compared to MCCFR training

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `epsilon` | f32 | 0.1 | Minimum probability threshold for an action to survive filtering |
| `action_cap` | usize | 2-8 | Maximum number of actions to keep per info state |
| `frac_limit` | usize | 20 | (SIP+ only) Maximum fraction denominator to search |
| `eta` | f32 | 0.005 | (SIP+ only) Maximum distance from probability to fraction for rounding |

**Thesis best parameters** (4x3 Dark Hex):
- Player 1 (Black): (b=8, epsilon=0.1)
- Player 2 (White): (b=2, epsilon=0.1)
- SIP+: (N=20, eta=0.005)

## How to Run

```python
from darkhex._engine import (
    MCCFRSolver, Sampling,
    simplify_policy, simplify_policy_plus,
    exploitability,
)

# Train MCCFR
solver = MCCFRSolver(2, 2, Sampling.Outcome, seed=42)
solver.solve(10_000)
strategy = solver.get_average_strategy()

# Apply SIP
sip_strategy = simplify_policy(strategy, epsilon=0.1, action_cap=2)

# Apply SIP+
sip_plus_strategy = simplify_policy_plus(
    strategy, epsilon=0.1, action_cap=2, frac_limit=20, eta=0.005
)

# Evaluate
print(f"Raw:  {exploitability(2, 2, strategy):.6f}")
print(f"SIP:  {exploitability(2, 2, sip_strategy):.6f}")
print(f"SIP+: {exploitability(2, 2, sip_plus_strategy):.6f}")
```

## Expected Output

For 2x2 Dark Hex (10k MCCFR iterations):
- Raw exploitability: ~0.001
- SIP exploitability: ~0.00 (noise removal can improve)
- SIP+ exploitability: ~0.00

Thesis results for 4x3 Dark Hex:
- Raw MCCFR: epsilon = 0.009 (abstract BR)
- SIP: epsilon = 0.009
- SIP+: epsilon = 0.002

## Implementation Notes

- **Deterministic**: no RNG used. Tiebreaking by lowest cell index (unlike legacy Python code which used `np.random.choice`).
- **Output format**: same `{info_state: [(action, prob), ...]}` as MCCFR, composable with `exploitability()`.
- **Fraction deduplication**: uses `f32::to_bits()` in a `BTreeSet` to deduplicate equivalent fractions (e.g., 1/2 = 2/4).
- **Fraction search**: binary search via `partition_point` for O(log F) nearest-fraction lookup.
- **Sum tolerance**: fractionized probabilities must sum to 1.0 within 1e-6 to be accepted.
- **Location**: `src/solver/sip.rs` (Rust), exposed via PyO3.
