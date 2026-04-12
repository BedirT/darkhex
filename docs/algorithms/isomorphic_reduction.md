# Algorithm: Isomorphic State Reduction

## What It Does

Reduces the info state space by ~50% by exploiting 180-degree rotation symmetry of the Hex board. Two info states that are related by this rotation are mapped to a single **canonical form** (the lexicographically smaller one), halving the MCCFR HashMap size and memory usage.

## Theoretical Basis

Hex has exactly one non-trivial board symmetry: 180-degree rotation. For a cell at position `pos` on a `rows x cols` board, the rotation maps it to `(rows * cols - 1) - pos`. Under this rotation:

- Black's North edge maps to South, South to North (winning condition preserved)
- White's West edge maps to East, East to West (winning condition preserved)
- Cell contents (`x`/`o`/`.`) are unchanged — no player swap needed

The grid portion of the info state string is character-reversed (excluding newline row separators). The canonical form is the lex-smaller of `(original, rotated)`.

**Action remapping**: When the rotated form is chosen as canonical, the legal action indices reverse. Since legal actions are always sorted ascending, and `n-1-a` reverses the order, the permutation is: `canonical_index i = original_index (k-1-i)`.

Reference: Standard symmetry exploitation in game solving (e.g., Schaeffer et al., Checkers is Solved).

## Computational Complexity

- **Time**: O(n) per info state computation (string comparison, n = board size)
- **Space**: ~50% reduction in info state HashMap
- **Practical runtime**: negligible overhead per MCCFR iteration

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| (none) | - | - | Always active, no configuration needed |

## How to Run

Isomorphic reduction is built into the solver — no opt-in required.

```python
from darkhex._engine import MCCFRSolver, enumerate_game_tree

# Solver automatically uses canonical keys
solver = MCCFRSolver(4, 3)
solver.solve(100000)
print(solver.num_info_states())  # ~half of 367,919

# Enumeration reports both original and canonical counts
stats = enumerate_game_tree(4, 3)
print(f"Total: {stats.total_info_states}, Canonical: {stats.canonical_info_states}")
```

## Expected Output

| Board | Total Info States | Canonical Info States | Reduction |
|-------|------------------|-----------------------|-----------|
| 2x2   | 42               | 22                   | 47.6%     |
| 3x2   | 410              | ~205                 | ~50%      |
| 3x3   | 12,556           | ~6,300               | ~50%      |
| 4x3   | 367,919          | ~184,000             | ~50%      |

## Implementation Notes

- The canonical form is computed in `DarkHexState::rs_canonical_info_state()` returning `(String, bool)` — the canonical key and whether the original was chosen
- MCCFR stores regrets/strategies under canonical keys with action remapping in both `external_sampling()` and `outcome_sampling()`
- Exploitability canonicalizes lookups in `best_response_value()` and maps probabilities back
- `get_average_strategy()` returns canonical keys with canonical action indices — consumers must canonicalize their lookups too
- The isomorphic pairs shown in the thesis (Figure 3.3) match this rotation: e.g., on 4x3, cells `(a1, c4)`, `(b1, b4)`, `(c1, a4)` are paired
