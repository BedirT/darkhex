---
name: Thesis epsilon 0.002 methodology
description: How the thesis achieved epsilon 0.002 on 4x3 Dark Hex — critical for reproducing results
type: project
---

The thesis epsilon=0.002 on 4x3 Dark Hex was NOT raw MCCFR output. It was a multi-stage pipeline:

1. **Base solver**: OpenSpiel `pyspiel.OutcomeSamplingMCCFRSolver` on `dark_hex_ir` (imperfect recall variant) with `use_early_terminal=True` (pONE). Trained for 10^9 iterations (1 billion).
2. **Post-processing with SIP**: Simplified Policy — prune actions below epsilon threshold (0.1), cap branching factor (b=2 or b=8), renormalize.
3. **Post-processing with SIP+**: Further smooth probabilities to nearby simple fractions (N=20, eta=0.005).
4. **Evaluation metric was Abstract Best Response (Ab-BR)**, NOT true game exploitability. Ab-BR buckets histories into information sets and ignores action ordering. The epsilon=0.002 is in abstract space only. In the full game with Approximate Best Response (DQN-based), epsilon was 0.049.

**Why:** The 10M iteration OS-MCCFR run with exploitability ~0.998 differs from the thesis by: (a) 100x fewer iterations (10M vs 1B), (b) no SIP/SIP+ post-processing, (c) possibly different exploitability metric.

**How to apply:** To reproduce thesis results, need 10^9 iterations minimum, then apply SIP/SIP+ post-processing. The exploitability metric used in our Rust code (clairvoyant best response) is stricter than Ab-BR, so our numbers will be higher.
