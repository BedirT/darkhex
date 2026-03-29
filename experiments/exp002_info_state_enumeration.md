# EXP-002: Exhaustive Info State Enumeration

## Hypothesis
The exact number of imperfect-recall info states for CDH Dark Hex can be
computed by exhaustive game tree traversal, providing ground truth for
all solver verification.

## Setup
- Method: Depth-first traversal with memoization on full game state
- Variant: CDH (Classic Dark Hex)
- Boards: 2x2, 2x3, 3x2, 3x3, 4x3, 3x4
- Code: `src/game/enumerate.rs`, `experiments/exp002_info_state_enumeration.py`

## Evaluation Plan
- Compare computed counts against thesis values (Tapkan 2022)
- Record per-player split, game states visited, max tree depth
- Measure wall-clock time to assess feasibility for larger boards

## Results

| Board | Info States | P0 | P1 | Game States | Depth | Time | Thesis | Match |
|-------|-----------|-----|-----|-------------|-------|------|--------|-------|
| 2x2 | **42** | 17 | 25 | 105 | 7 | 0.001s | 42 | MATCH |
| 2x3 | **314** | 147 | 167 | 1,797 | 11 | 0.02s | N/A | NEW |
| 3x2 | **410** | 172 | 238 | 2,469 | 11 | 0.02s | 410 | MATCH |
| 3x3 | **12,556** | 6,293 | 6,263 | 283,859 | 17 | 3.0s | 12,556 | MATCH |
| 4x3 | **367,919** | 184,024 | 183,895 | 31,949,417 | 23 | 8.5min | 367,919 | MATCH |
| 3x4 | **341,033** | 170,597 | 170,436 | 28,560,489 | 23 | 7.1min | N/A | NEW |

### Key Findings
1. All thesis values independently confirmed (2x2, 3x2, 3x3, 4x3)
2. Two new results: 2x3 = 314, 3x4 = 341,033
3. Board orientation matters: 4x3 ≠ 3x4 (367,919 vs 341,033) because
   Black connects N-S and White connects W-E
4. Memoization on full game state gives ~7,000x speedup on 3x3 (6.2h → 3s)
   and makes 4x3 feasible (8.5 min vs intractable)
5. P0/P1 split is nearly equal on larger boards (4x3: 184,024 vs 183,895)
6. Game state count grows ~100x per board size step
7. 4x4 would have ~10M+ info states — may be feasible but slow

## Optimization: Memoization
The original naive DFS cloned the full game state at every node and
traversed 9.47 billion terminal histories for 3x3 (6.2 hours). The
memoized version tracks visited (true_board, player_views, current_player)
tuples and skips duplicate subtrees, reducing 3x3 to 283,859 unique
game states (3 seconds).

## Status: done

## Data
- `results/exp002_info_state_enumeration/enumeration.csv`
- `results/exp002_info_state_enumeration/full_results.json`

## Next Step
Use these ground truth counts to validate pONE pruning (should reduce
info state count by ~20% on 4x3) and isomorphic reduction (should
roughly halve the count).
