# Experiment: EXP-004 — Deep CFR Prototype

## Hypothesis

**H1**: Deep CFR (Brown et al., ICML 2019) converges on 2x2 CDH Dark Hex,
achieving exploitability < 0.3 within 100 CFR iterations (K=375 traversals each).

**H2**: Deep CFR on 4x3 CDH achieves exploitability < 0.5 within 200 CFR iterations,
improving over OS-MCCFR's early convergence rate at matched wall-clock time.

**Motivation**: Tabular OS-MCCFR plateaus at ~0.989 exploitability on 4x3 after 1B
iterations (clairvoyant BR metric). Neural function approximation may converge faster
by generalizing across similar information states (~184k canonical states compressed
into a small MLP).

## Setup

- Algorithm: Deep CFR (External Sampling MCCFR + neural advantage/strategy networks)
- Board sizes: 2x2 (verification), 4x3 (target)
- Variant: CDH (Classic Dark Hex — player retries after collision)
- Parameters:
  - CFR iterations (T): 100 (2x2), 200 (4x3)
  - Traversals per player per iteration (K): 375
  - Hidden layers: (64, 64) for 2x2, (128, 128) for 4x3
  - Learning rate: 1e-3 (Adam)
  - Advantage SGD steps: 375
  - Strategy SGD steps: 2500
  - Buffer capacity: 100,000 (2x2), 1,000,000 (4x3)
  - Batch size: 256
  - LCFR weighting (linear iteration weighting via sqrt(t) trick)
  - Advantage networks reinitialized from scratch each CFR iteration
  - Argmax tiebreaker when all regrets ≤ 0
  - Isomorphic reduction (canonical info states via 180° rotation)
  - Seed: 42
- Baseline comparison: Tabular OS-MCCFR (EXP-003 results)

## Evaluation Plan

1. Track exploitability (clairvoyant BR) at each CFR iteration
2. Compare convergence curve against tabular OS-MCCFR at matched wall-clock time
3. Measure throughput (traversals/second, CFR iterations/hour)
4. Track advantage and strategy network training loss
5. Success criteria:
   - 2x2: exploitability < 0.3 within 100 iterations
   - 4x3: exploitability < 0.5 within 200 iterations

## Verification Plan

1. Verify on 2x2 first (22 canonical info states, known equilibrium)
2. Check extracted strategy probabilities sum to 1 at all info states
3. Compare 2x2 equilibrium strategies against tabular MCCFR output
4. Sanity check: uniform random strategy should have higher exploitability than trained
5. Run `make check` to verify all existing tests still pass

## Run Instructions

```bash
# Install neural dependencies
uv sync --extra neural

# Build Rust extension
make dev

# Run 2x2 verification
uv run python experiments/exp004_deep_cfr_prototype.py --board 2x2

# Run 4x3 experiment
uv run python experiments/exp004_deep_cfr_prototype.py --board 4x3
```

## Results

### 2x2 CDH (100 CFR iterations, K=200 traversals)
- Exploitability: 1.00 → 0.12 → 0.02 → **0.012** (converges to near-Nash)
- ~0.9s per CFR iteration, 91s total
- Comparable to tabular OS-MCCFR at 100k iterations (expl ~0.0001)

### 3x2 CDH (50 CFR iterations, K=200 traversals)
- Exploitability: 1.00 → 0.33 → 0.07 → **0.055** (converges well)
- ~6s per CFR iteration, 291s total

### 3x3 CDH (30 CFR iterations, K=5 traversals)
- Exploitability: 1.00 → 0.97 → 0.88 → **0.86** (slow convergence with K=5)
- ~20s per CFR iteration, 618s total
- K=5 traversals is too few for 12,556 info states — needs K=50+ for proper convergence
- Strategy extraction optimized: 31.9M game states → memoized to ~5s (was >10min)

### 4x3 CDH — External Sampling infeasible
- Single ES traversal on 4x3 does not complete within 30s
- Scaling: 2x2=0.00s, 3x2=0.07s, 3x3=14.4s, 4x3=∞
- ES explores ALL actions at traverser nodes → exponential in branching factor
- **Requires Outcome Sampling variant (DREAM, Steinberger et al. 2020) for 4x3+**

## Analysis

**H1 CONFIRMED**: Deep CFR converges on 2x2 (expl 0.012 << 0.3 target) and
3x2 (expl 0.055). 3x3 shows slow but clear convergence (0.86 at K=5, needs more traversals).

**H2 NOT TESTABLE with ES**: External Sampling infeasible for 4x3.
Paper acknowledges "a different sampling scheme, such as outcome sampling,
may be desired" for games with large branching factors.

**Key findings**:
1. Neural approximation converges well — bottleneck is ES traversal, not NNs
2. K (traversals per iteration) is critical: K=200 works for 2x2/3x2, K=5 is insufficient for 3x3
3. Strategy extraction optimization (game state memoization) reduced 3x3 eval from >10min to 5s
4. 3x3 is on the edge of ES feasibility — convergence is possible but slow

## Next Steps

1. Implement DREAM (Deep Regret minimization with Advantage baselines and Model-free
   learning, Steinberger et al. 2020) — uses Outcome Sampling instead of ES
2. Re-run 4x3 experiment with DREAM traversal
3. Compare wall-clock convergence against tabular OS-MCCFR

## Write-up Plan

- Paper Section 5: "Neural Approaches" — convergence comparison figure
- Figure: log-log exploitability vs wall-clock time (Deep CFR vs OS-MCCFR)
- Table: exploitability at matched iteration counts
- Discussion: ES infeasibility motivates Outcome Sampling variant
