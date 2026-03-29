# Dark Hex Game Engine

## What It Does

Implements the complete game logic for Dark Hex — an imperfect-information variant of the board game Hex. The engine handles board state management, move execution with collision detection, win detection, and information state computation for each player.

This is the foundation that all algorithms (MCCFR, best response, etc.) build on.

## Theoretical Basis

### Hex

Hex is a two-player connection game played on a rhombus-shaped board of hexagonal cells. Player 1 (Black) connects North-South, Player 2 (White) connects West-East. Hex has several important properties:

- **Determinacy**: Hex cannot end in a draw (any full board has exactly one winner). This follows from the Hex theorem, which is equivalent to the Brouwer fixed-point theorem.
- **First-player advantage**: Black (first mover) has a winning strategy on any board size, proved by strategy-stealing argument (Nash, 1952).

### Dark Hex

Dark Hex (also called Phantom Hex or Kriegspiel Hex) adds imperfect information:

- Players **cannot see** opponent's stones
- When a player tries to place on an opponent-occupied cell (**collision**), their turn is wasted but they learn the cell is occupied
- Each player maintains a **partial observation** of the board

This transforms Hex from a perfect-information game (solvable by minimax) into an imperfect-information extensive-form game requiring game-theoretic solution concepts (Nash equilibrium via CFR).

### Information States

Two variants:

1. **Imperfect recall**: Player's info state = their current board view. Forgets the order of their own moves. Compact but loses strategic information.
2. **Perfect recall**: Info state = board view + ordered history of own actions. Standard for CFR but larger state space.

### References

- Nash, J. (1952). "Some Games and Machines for Playing Them." RAND Corporation.
- Gale, D. (1979). "The game of Hex and the Brouwer fixed-point theorem." The American Mathematical Monthly.
- Lanctot, M. et al. (2009). "Monte Carlo Sampling for Regret Minimization in Extensive Games." NeurIPS.
- Tapkan, B. (2024). MSc Thesis — Dark Hex equilibrium bounds. University of Alberta.

## Computational Complexity

### Board Operations

| Operation | Time | Space |
|-----------|------|-------|
| Place stone | O(α(n)) amortized | O(1) |
| Win detection | O(α(n)) | O(1) |
| Legal actions | O(n) | O(n) |
| Info state string | O(n) | O(n) |
| State clone | O(n) | O(n) |
| Neighbour lookup | O(1) | O(1) |

Where n = rows × cols and α is the inverse Ackermann function (effectively constant).

### Game Tree Size

| Board | States | Info sets (est.) |
|-------|--------|-----------------|
| 2×2 | ~24 | ~40 |
| 3×2 | ~720 | ~2,000 |
| 3×3 | ~362,880 | ~50,000 |
| 4×4 | ~2×10¹³ | intractable exact |

Dark Hex has more info sets than Hex game states because each player has multiple possible observations for the same true board state.

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `rows` | int | — | Number of board rows |
| `cols` | int | — | Number of board columns |

The engine is parameterized only by board dimensions. All other game rules are fixed.

## How to Run

```python
from darkhex._engine import DarkHexState, Player

# Create a new game
state = DarkHexState(rows=3, cols=3)

# Game loop
while not state.is_terminal():
    player = state.current_player()
    actions = state.legal_actions()
    # Choose action (e.g., random, or from MCCFR policy)
    state.apply_action(actions[0])

# Result
print(state.winner())       # Player.Black or Player.White
print(state.returns())      # [1.0, -1.0] or [-1.0, 1.0]

# Information states (for CFR)
info = state.info_state_string(Player.Black)
info_pr = state.info_state_string_perfect_recall(Player.Black)
```

```bash
# Run tests
make check       # lint + Rust tests + Python tests
cargo test       # Rust unit tests only
uv run pytest -x # Python integration tests only
```

## Expected Output

### 2x2 Board Verification

```python
# Black wins by connecting column 0
s = DarkHexState(2, 2)
s.apply_action(0)  # Black (0,0) — North edge
s.apply_action(1)  # White (0,1)
s.apply_action(2)  # Black (1,0) — South edge
assert s.winner() == Player.Black

# Info state format
s2 = DarkHexState(2, 2)
s2.apply_action(0)  # Black at (0,0)
s2.apply_action(3)  # White at (1,1) — invisible to Black
assert s2.info_state_string(Player.Black) == "P0\nx.\n.."
assert s2.info_state_string(Player.White) == "P1\n..\n.o"
```

## Implementation Notes

### Design Decision: Union-Find vs Flood-Fill

The original Python implementation used flood-fill for win detection, which is O(n) per check. We switched to union-find (disjoint set with path halving + union by rank), giving O(α(n)) ≈ O(1) amortized per stone placement.

Four virtual nodes represent board edges. Win = connected(North, South) for Black or connected(West, East) for White.

### Design Decision: Per-Player Views

Rather than recomputing what each player can see from action histories (expensive), we maintain `player_views[2][n]` arrays updated incrementally:
- On successful placement: mark cell as own stone in own view
- On collision: mark cell as opponent stone in own view

This makes `legal_actions()` and `info_state_string()` simple array scans — no history replay needed.

### Design Decision: String Info States

Info states are formatted as strings (`"P0\nx.\n.."`) rather than structured objects. This is deliberate: MCCFR algorithms use info states as dictionary keys, and strings are hashable, comparable, and serializable without extra work. The format matches the original thesis implementation for result comparison.

### Edge Case: Collision Cascades

In Dark Hex, a player can collide multiple times on different cells. Each collision reveals one opponent stone. The game cannot loop infinitely because:
1. Each collision reveals new information (shrinks the player's apparent action space)
2. Hex on a full board always has a winner (Hex theorem)
3. Therefore the game must terminate

### Known Limitation: No Isomorphic Reduction

The current engine does not exploit board symmetry (180° rotation). The old code (`darkhex/utils/isomorphic.py`) had this. Planned for later — halves the effective info set count.

### What's NOT in the Engine

The engine is purely game logic. It does NOT include:
- Algorithm implementations (MCCFR, best response) — those are Python
- Policy storage or serialization — algorithms handle that
- Visualization — separate NiceGUI layer
- OpenSpiel compatibility — clean break from pyspiel dependency
