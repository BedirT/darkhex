# EXP-002: Exhaustive Info State Enumeration

## Hypothesis
The exact number of imperfect-recall info states for CDH Dark Hex can be
computed by exhaustive game tree traversal, providing ground truth for
all solver verification.

## Setup
- Method: Depth-first traversal of complete game tree
- Variant: CDH (Classic Dark Hex)
- Boards: 2x2, 2x3, 3x2, 3x3

## Evaluation Plan
- Compare computed counts against thesis values (Tapkan 2022)
- Record per-player split, terminal count, max tree depth
- Measure wall-clock time to assess feasibility for larger boards

## Results

| Board | Info States | P0 | P1 | Terminals | Max Depth | Time | Thesis | Match |
|-------|-----------|-----|-----|-----------|-----------|------|--------|-------|
| 2x2 | **42** | 17 | 25 | 216 | 7 | 0.001s | 42 | MATCH |
| 2x3 | **314** | 147 | 167 | 53,920 | 11 | 0.12s | N/A | NEW |
| 3x2 | **410** | 172 | 238 | 95,760 | 11 | 0.23s | 410 | MATCH |
| 3x3 | **12,556** | 6,293 | 6,263 | 9,469,697,760 | 17 | 6.2h | 12,556 | MATCH |

### Key Findings
1. All thesis values independently confirmed by our implementation
2. 2x3 (314 info states) is a new result not in the thesis
3. Terminal state explosion: 216 → 95,760 → 9.47B from 2x2 to 3x3
4. Max depth increases with CDH collision retries (7 → 11 → 17)
5. P1 (White) has slightly more info states than P0 on small boards
   (25 vs 17 on 2x2, 238 vs 172 on 3x2), but nearly equal on 3x3
6. 4x3 enumeration is infeasible (~10^17 IR info states per thesis)

## Data
- `results/exp002_info_state_enumeration/enumeration.csv`
- `results/exp002_info_state_enumeration/full_results.json`

## Status: done

## Next Step
Use these ground truth counts to validate any future solver or pruning
technique (pONE, isomorphic reduction).
