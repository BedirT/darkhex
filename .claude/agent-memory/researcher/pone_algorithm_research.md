---
name: pONE Algorithm Research — Weak vs Strong Condition
description: Complete analysis of pONE bug (weak vs strong quantifier ordering), thesis algorithm, old Python code (ryan_alg AND-OR), Russell-Wolfe belief-state AND-OR framework, and correct fix approach
type: project
---

## The Bug

Current `is_pone` in `src/solver/pone.rs` checks the WEAK condition:
  for all hidden_config, exists strategy: wins(strategy, config)
The STRONG (correct) condition is:
  exists strategy, for all hidden_config: wins(strategy, config)

Falsely flags P1 root on non-square boards where rows > cols (3x2, 4x3).

## What the Thesis Says

Section 4.2 (players.tex) defines pONE informally: "sometimes a player knows they can win with probability 1." The description in the thesis text does not explicitly state the quantifier ordering; it uses examples where the distinction doesn't matter (h=0 where they are equivalent, or h=1 with 2 winning cells where they happen to coincide).

The thesis pseudocode (pseudocodes/pone.tex and pseudocodes/system.tex) is incomplete -- the h>0 branch is left blank.

## What the Old Python Code Does (ryan_alg)

The old code (PONE/pone.py, commit b1ef696) implements `ryan_alg` which IS an AND-OR tree search -- the correct approach:

```python
if h == 0:
    # OR node: try each legal action, return True if ANY works
    for x in vm:
        n_state = update(state, x, color, h)
        if ryan_alg(n_state, h):
            return True

elif h > 0:
    # AND-OR node: for each cell y, BOTH branches must win:
    #   1. collision branch (hit hidden stone): ryan_alg(state_with_opp_at_y, h-1)
    #   2. success branch (placed own stone): ryan_alg(state_with_own_at_y, h)
    for y in vm:
        if ryan_alg(n_state_hW, h-1) AND ryan_alg(n_state_B, h):
            return True
```

This is the correct AND-OR structure:
- Player's action choice = OR (try cells, succeed if ANY works)
- Outcome of action = AND (must handle BOTH collision and success)
- Collision reduces h by 1 (one fewer hidden stone)
- Success keeps h the same (hidden stones unchanged)

**Why:** When h>0 and the player tries cell y:
- If y has a hidden opponent stone -> collision (probability proportional to hidden count). Player sees it, h decreases.
- If y is truly empty -> success, stone placed, h unchanged.
- A winning strategy must work in BOTH cases for the same action y.

This AND-branching is what makes it the strong condition. The player commits to action y BEFORE knowing whether collision or success occurs, so the same action must lead to a win in both branches.

## The Rust Implementation Error

The Rust code replaced this AND-OR search with per-config minimax:
1. Enumerate all C(empty, h) placements of hidden stones
2. For each placement, run perfect-information minimax
3. If player wins ALL placements -> pONE

This is wrong because each minimax call finds a DIFFERENT strategy per placement. It's the weak condition.

## Correct Algorithm: AND-OR Tree Search (a la Ryan/Russell-Wolfe)

**Russell & Wolfe (IJCAI 2005)** formalized this exactly for Kriegspiel:

Node types:
- **OR-nodes**: Player's choice. Proven if at least one child is proven.
- **AND-nodes**: Nature/uncertainty. Contains a partition of physical states. Proven iff ALL children are proven.

For Dark Hex pONE:
- OR-node: Player picks cell y. Proven if any y leads to win.
- AND-node: For chosen y, two branches exist:
  - Collision (y had hidden stone): new state with revealed stone, h-1
  - Success (y was empty): new state with own stone placed, same h
  - Both must be won.

The old Python `ryan_alg` already implements this correctly. The Rust rewrite lost the AND-branching.

## Key Insight: CDH Simplification

In CDH (Classic Dark Hex), collisions don't waste turns -- the player retries. This means:
- After collision on y, the player has gained info (sees opponent stone at y) but hasn't placed a stone. It's still their turn.
- The game continues from a state where one hidden stone is now visible.
- This is why h decreases by 1 on collision: one fewer hidden stone.

## Recommended Fix

Replace `is_pone` with an AND-OR search:

```rust
fn is_pone_andor(view, player, h, rows, cols, memo) -> bool {
    if h == 0:
        // Perfect information: just minimax
        return hex_minimax(view, rows, cols, player, memo) == player

    // OR: try each empty-appearing cell
    for y in empty_appearing_cells(view):
        // AND: both branches must win

        // Branch 1: collision (hidden stone was at y)
        let view_collision = view with opponent revealed at y
        let collision_wins = is_pone_andor(view_collision, player, h-1, ...)

        // Branch 2: success (cell was truly empty)
        let view_success = view with own stone at y
        let success_wins = is_pone_andor(view_success, player, h, ...)

        if collision_wins && success_wins:
            return true  // Found action that wins in both cases

    return false  // No action works for both cases
}
```

## Complexity Note

The AND-OR search is more expensive than per-config minimax but still tractable for small boards. The branching factor at each OR node is |empty_cells|, and the AND branching is always 2 (collision/success). Total depth is bounded by the number of remaining moves.

## References

- Bonnet, F. (2018). "Winning strategies in DarkHex: Hex with hidden stones." ICGA Journal, 40(3), 234-245.
- Russell, S. & Wolfe, J. (2005). "Efficient belief-state AND-OR search, with application to Kriegspiel." IJCAI 2005, 278-285.
- Thesis Section 4.2 (players.tex, line 82-160)
- Old Python: commit b1ef696 PONE/pone.py `ryan_alg`
- Current Rust: src/solver/pone.rs `is_pone` (lines 279-335)
