# Algorithm: pONE (Probability One Win States)

## What It Does

Identifies game states where the current player can win with **probability 1** from their information state, regardless of where hidden opponent stones are placed. These states are treated as pseudo-terminals during MCCFR traversal, pruning the subtree and saving memory.

This is a **belief-space check**, not a simple minimax on the true board. The player must have a **single** winning strategy that works for ALL possible configurations of hidden opponent stones simultaneously.

## Theoretical Basis

Defined by Bonnet (2018) and used in the thesis (Section 4.2). The correct pONE check uses an AND-OR tree search (Russell & Wolfe, IJCAI 2005):

**When h=0** (no hidden stones): Player sees the full board. Standard perfect-information minimax determines if they can force a win.

**When h>0** (hidden stones exist): For each empty-appearing cell y the player considers (OR node):
- **AND branch 1 (collision):** Assume y holds a hidden opponent stone. The collision reveals it: player's view now shows the opponent stone at y, h decreases by 1. Recurse.
- **AND branch 2 (success):** Assume y is truly empty. Player's stone is placed at y. Recurse.
- **Both branches must succeed** for action y to be viable.
- If ANY action y satisfies both branches → pONE.

**Opponent turns:** When it's the opponent's turn (after a successful placement), h increments because the opponent places a hidden stone.

This enforces the **strong condition**: ∃ strategy, ∀ config: strategy wins. The player commits to action y before knowing whether it results in collision or success. The AND node ensures the same choice works in both scenarios.

### Historical Note

An earlier implementation (pre-2026-03-30) used per-configuration minimax, which checks the **weak condition**: ∀ config, ∃ strategy that wins. This produced false positives on non-square boards (e.g., P1 root on 4x3), causing the MCCFR solver to prune the entire game tree. The AND-OR fix corrects this.

References:
- Bonnet, F. (2018). "Winning strategies in DarkHex." ICGA Journal.
- Russell, S. & Wolfe, J. (2005). "Efficient belief-state AND-OR search." IJCAI.

## Computational Complexity

- **Precomputation time**: depends on board size and max hidden count
  - 2x2: <1ms, 3x2: ~10ms, 3x3: ~seconds, 4x3: ~1-2 minutes
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
print(db)  # PoneDb(4x3, 57485 pONE states)

# Use with MCCFR
solver = MCCFRSolver(4, 3, Sampling.Outcome, seed=42)
solver.set_pone_db(db)
solver.solve(100000)

# Use with exploitability (optional — does NOT bias the measurement)
expl = exploitability(4, 3, solver.get_average_strategy(), db)

# Without pONE (backward compatible)
expl_no_pone = exploitability(4, 3, solver.get_average_strategy())
```

## Expected Output

| Board | Canonical Info States | pONE States | MCCFR Info State Reduction |
|-------|----------------------|-------------|---------------------------|
| 2x2   | 22                   | 12          | ~55%                      |
| 3x2   | ~205                 | 102         | ~50%                      |
| 3x3   | ~6,278               | 2,314       | ~37%                      |
| 4x3   | ~184,000             | 57,485      | ~33% (measured at 100k iters) |

## Implementation Notes

- **AND-OR tree search**: `pone_andor()` in `src/solver/pone.rs`. Memoized on (view, h) keys.
- **CDH only**: pONE relies on CDH property that collisions don't waste turns.
- **Game logic untouched**: `DarkHexState`, `rs_is_terminal()`, `HexBoard` are never modified.
- **Canonical keys**: pONE stores canonical info state strings (isomorphic reduction applied).
- **Legacy data**: Precomputed pickle files from the thesis exist at `darkhex/data/pone_states/`. Note: these use a different representation (flat view strings + h) and include states for a specific player, so counts differ from our canonical info state counts.
