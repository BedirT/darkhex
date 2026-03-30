# Algorithm: pONE (Probability One Win States)

## What It Does

Identifies game states where the current player can win with **probability 1** from their information state, regardless of where hidden opponent stones are placed. These states are treated as pseudo-terminals during MCCFR traversal, pruning the subtree and saving ~20% memory on 4x3.

This is a **belief-space check**, not a simple minimax on the true board. The player must have a winning strategy that works for ALL possible configurations of hidden opponent stones.

## Theoretical Basis

Defined by Bonnet (2018) and used in the thesis (Section 4.2). In CDH Dark Hex, collisions don't waste turns — the player retries until placing. A state is pONE for player P if:

1. Parse P's information state to get: own stones, discovered opponent stones, empty-appearing cells
2. Compute `hidden_count = true_opponent_stones - visible_opponent_stones`
3. For ALL `C(empty_cells, hidden_count)` placements of hidden stones among empty-appearing cells:
   - Construct the hypothetical true board
   - Check if P can force a win via regular Hex minimax
4. If P wins in EVERY placement → pONE

**Examples**:
- h=0: Player sees full board. If they have a winning move, it's pONE.
- h=1, 2 winning cells: Even if hidden stone blocks one, other works → pONE.
- h>0 complex: Multi-step strategy covering all hidden stone configurations.

Reference: Bonnet, F. (2018). "Winning Dark Hex."

## Computational Complexity

- **Precomputation time**: depends on board size and max hidden count
  - 2x2: <1ms, 3x2: ~10ms, 3x3: ~seconds, 4x3: minutes
- **Runtime overhead**: O(1) HashSet lookup per info state during traversal
- **Space**: HashSet of canonical pONE info state strings

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| rows | usize | required | Board rows |
| cols | usize | required | Board columns |

## How to Run

pONE is opt-in. Build the database, then pass it to the solver.

```python
from darkhex._engine import MCCFRSolver, PoneDb, Sampling, exploitability

# Build pONE database (one-time precomputation)
db = PoneDb(4, 3)
print(db)  # PoneDb(4x3, N pONE states)

# Use with MCCFR
solver = MCCFRSolver(4, 3, Sampling.Outcome, seed=42)
solver.set_pone_db(db)
solver.solve(100000)

# Use with exploitability (optional)
expl = exploitability(4, 3, solver.get_average_strategy(), db)

# Without pONE (backward compatible)
expl_no_pone = exploitability(4, 3, solver.get_average_strategy())
```

## Expected Output

| Board | Total Canonical | pONE States | Reduction |
|-------|----------------|-------------|-----------|
| 2x2   | 22             | ~15-18      | ~70%      |
| 3x2   | ~205           | varies      | varies    |
| 4x3   | ~184,000       | ~36,800     | ~20%      |

## Implementation Notes

- **CDH only**: pONE relies on the CDH property that collisions don't waste turns. For ADH, collisions change the true-board alternation, invalidating the regular Hex equivalence.
- **Game logic untouched**: `DarkHexState`, `rs_is_terminal()`, `HexBoard` are never modified. pONE is purely a solver-side optimization.
- **Canonical keys**: pONE stores canonical info state strings (isomorphic reduction applied).
- **Building block**: `hex_minimax` provides memoized regular Hex minimax, reused across pONE checks.
- **Legacy data**: Precomputed pickle files from the thesis exist at `darkhex/data/pone_states/`.
