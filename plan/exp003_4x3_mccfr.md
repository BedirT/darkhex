# EXP-003: MCCFR Convergence on 4x3 Dark Hex

**Paper target**: Main result table + convergence figure (Section: Experiments).
This is the thesis headline board. The thesis reports epsilon improved from
0.156 to 0.002 on 4x3 CDH. This experiment validates that result with the
new Rust engine and quantifies the effect of pONE pruning.

---

## Hypotheses

### H1 (Primary): Convergence to near-Nash
Outcome Sampling MCCFR on 4x3 CDH converges to exploitability < 0.01
within 10M iterations, matching the thesis result (epsilon 0.002).

**Falsifiable**: If exploitability plateaus above 0.01 at 10M iterations,
the hypothesis is rejected. Root cause candidates: OS-MCCFR variance,
insufficient exploration, or a bug in canonical action mapping at this scale.

### H2 (pONE effect): Pruning accelerates convergence
pONE pruning reduces both memory (fewer info states stored) and
convergence time (fewer iterations to reach the same exploitability)
relative to vanilla MCCFR.

**Falsifiable**: If pONE does not reduce info state count by at least 10%
or does not improve exploitability at matched iteration counts, the
hypothesis is rejected.

### H3 (Isomorphic reduction): ~50% info state savings confirmed at scale
With isomorphic reduction always on, the solver should discover
~184,000 canonical info states (half of the 367,919 total).

**Falsifiable**: If the final canonical count deviates by more than 5%
from 184,000, something is wrong with the canonicalization.

---

## Evaluation Metric

**Exploitability** = (BR_Black + BR_White) / 2, computed via clairvoyant
best response (upper bound, tight for well-converged strategies).

Success criteria:
- exploitability < 0.01 (matches thesis order of magnitude)
- exploitability < 0.005 (matches/beats thesis result of 0.002)

---

## Known Ground Truth

From EXP-002 and the thesis:

| Property | Value | Source |
|----------|-------|--------|
| Total info states | 367,919 | EXP-002 (confirmed) |
| P0 info states | 184,024 | EXP-002 |
| P1 info states | 183,895 | EXP-002 |
| Game states (for exploitability memo) | 31,949,417 | EXP-002 |
| Max tree depth | 23 | EXP-002 |
| Thesis best epsilon | 0.002 | agent-progress.md |
| Thesis baseline epsilon | 0.156 | agent-progress.md |
| pONE states (expected) | ~36,800 | pone.md (~20% reduction) |

---

## Resource Estimates

### Memory

| Component | Estimate | Notes |
|-----------|----------|-------|
| MCCFR info state table | ~40-50 MB | ~184k entries, avg 8 actions, f32 regrets+strats |
| Exploitability memo (per call) | ~3.2 GB per player | 32M game states, Vec<u8> keys |
| Exploitability total (both players) | ~6.5 GB | Two passes, sequential |
| pONE precomputation | ~8.5 min, ~hundreds of MB | Full game tree + minimax memo |

**Constraint**: The machine must have at least 8 GB free RAM for
exploitability computation. The MCCFR solve itself is cheap (~50 MB).

### Runtime

| Phase | Estimate | Notes |
|-------|----------|-------|
| pONE precomputation | ~8-10 min | One-time, before any MCCFR runs |
| MCCFR throughput (4x3) | ~5,000-10,000 iter/s | Extrapolated from 3x3 @ 16k/s |
| 1M MCCFR iterations | ~2-4 min | |
| 10M MCCFR iterations | ~20-35 min | |
| Exploitability (4x3) | ~11 min per call | From exploitability.md estimate |
| Full experiment (both conditions, all checkpoints) | ~3-5 hours | Dominated by exploitability calls |

### Bottleneck Analysis

Exploitability is the bottleneck, not MCCFR. At ~11 min per call, with
16 checkpoint measurements across 2 conditions, that is ~350 min (~6 hours)
of exploitability alone. This drives the checkpoint schedule design below.

---

## Experiment Design

### Conditions

| Condition | pONE | Description |
|-----------|------|-------------|
| A: Vanilla | off | Baseline MCCFR without pruning |
| B: pONE | on | MCCFR with pONE pruning via set_pone_db() |

Both use: Outcome Sampling, epsilon=0.6, seed=42, 4x3 CDH.

### Iteration Schedule and Checkpoint Strategy

Given the exploitability bottleneck, we use a logarithmic checkpoint
schedule. Each checkpoint requires ~11 min for exploitability, so we
limit to 8 checkpoints per condition (16 total, ~3 hours exploitability).

**Checkpoint iterations** (logarithmic spacing):

```
10k, 50k, 100k, 500k, 1M, 2M, 5M, 10M
```

Rationale:
- 10k: early baseline (expect high exploitability, ~0.3-0.5)
- 50k-100k: should see clear improvement
- 500k-1M: approaching thesis range
- 2M-5M: refinement zone
- 10M: final measurement, target < 0.01

**Implementation**: Use a single solver instance per condition. Call
`solver.solve(delta)` incrementally between checkpoints. This avoids
restarting MCCFR and preserves the full regret history.

### Measurements at Each Checkpoint

| Metric | Method | Cost |
|--------|--------|------|
| Exploitability | `exploitability(4, 3, strategy)` | ~11 min |
| BR Black, BR White | `best_response_values(4, 3, strategy)` | same call |
| Info states discovered | `solver.num_info_states()` | O(1) |
| Wall clock time | `time.time()` delta | O(1) |
| MCCFR throughput | iterations / wall_time | O(1) |

---

## Script Structure

### File: `experiments/exp003_4x3_mccfr.py`

```
experiments/
  exp003_4x3_mccfr.py     # Main experiment script
  exp003_4x3_mccfr.md     # This plan (copy into experiments/ after run)
results/
  exp003_4x3_mccfr/
    convergence.csv         # Machine-readable: iters, expl, br_b, br_w, ...
    full_results.json       # Full data including strategies at final checkpoint
    convergence.png         # Convergence plot (log-log)
```

### Script Skeleton (pseudocode)

```python
"""EXP-003: MCCFR Convergence on 4x3 CDH Dark Hex.

Hypothesis: OS-MCCFR converges to epsilon < 0.01 on 4x3,
with pONE pruning accelerating convergence.
"""

import csv, json, time
from pathlib import Path
from darkhex._engine import (
    MCCFRSolver, Sampling, PoneDb,
    exploitability, best_response_values,
)

RESULTS_DIR = Path("results/exp003_4x3_mccfr")
ROWS, COLS = 4, 3
SEED = 42
EPSILON = 0.6

CHECKPOINTS = [10_000, 50_000, 100_000, 500_000,
               1_000_000, 2_000_000, 5_000_000, 10_000_000]

CONDITIONS = [
    {"name": "vanilla", "pone": False},
    {"name": "pone",    "pone": True},
]


def build_pone_db():
    """One-time pONE precomputation (~8-10 min)."""
    print("Building pONE database for 4x3...")
    t0 = time.time()
    db = PoneDb(ROWS, COLS)
    elapsed = time.time() - t0
    print(f"  pONE: {db.len()} states, {elapsed:.1f}s")
    return db


def run_condition(name, pone_db=None):
    """Run MCCFR with checkpointed exploitability measurements."""
    solver = MCCFRSolver(ROWS, COLS, Sampling.Outcome,
                         epsilon=EPSILON, seed=SEED)
    if pone_db is not None:
        solver.set_pone_db(pone_db)

    records = []
    prev_iters = 0
    total_solve_time = 0.0

    for target in CHECKPOINTS:
        delta = target - prev_iters

        # MCCFR solve phase
        t0 = time.time()
        solver.solve(delta)
        solve_time = time.time() - t0
        total_solve_time += solve_time
        prev_iters = target

        # Exploitability measurement
        strategy = solver.get_average_strategy()
        t_expl = time.time()
        br_b, br_w, expl = best_response_values(
            ROWS, COLS, strategy
        )
        expl_time = time.time() - t_expl

        record = {
            "condition": name,
            "iterations": target,
            "exploitability": expl,
            "br_black": br_b,
            "br_white": br_w,
            "info_states": solver.num_info_states(),
            "solve_time_s": round(total_solve_time, 1),
            "expl_time_s": round(expl_time, 1),
            "throughput_iter_s": round(target / total_solve_time, 0),
        }
        records.append(record)

        print(f"  {name} @ {target:>10,}: "
              f"expl={expl:.6f}  "
              f"info_states={solver.num_info_states():>7,}  "
              f"solve={total_solve_time:.0f}s  "
              f"expl_compute={expl_time:.0f}s")

    return records


def main():
    print("=== EXP-003: MCCFR on 4x3 CDH ===\n")

    # Phase 1: pONE precomputation
    pone_db = build_pone_db()

    all_records = []

    # Phase 2: Vanilla condition
    print("\n--- Condition A: Vanilla (no pONE) ---")
    all_records.extend(run_condition("vanilla"))

    # Phase 3: pONE condition
    print("\n--- Condition B: pONE pruning ---")
    all_records.extend(run_condition("pone", pone_db))

    # Save results
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)

    csv_path = RESULTS_DIR / "convergence.csv"
    # ... write CSV with fieldnames from record keys ...

    json_path = RESULTS_DIR / "full_results.json"
    # ... write JSON with all records + final strategies ...

    # Phase 4: Generate convergence plot
    # plot_convergence(all_records, RESULTS_DIR / "convergence.png")


if __name__ == "__main__":
    main()
```

### Key Implementation Details

1. **Incremental solve**: Call `solver.solve(delta)` not `solver.solve(target)`.
   The solver accumulates iterations internally. Verified by `solver.iterations()`.

2. **Strategy extraction**: `solver.get_average_strategy()` returns the full
   `{info_state_str: [(action, prob), ...]}` dict. This must be passed to
   `best_response_values()` each time (it is a snapshot, not a reference).

3. **pONE is opt-in**: Pass the `PoneDb` via `solver.set_pone_db(db)` before
   the first `solve()` call. Do NOT pass it to `exploitability()` -- the
   exploitability doc explicitly notes pONE is not used there (it would
   bias the measurement).

4. **No strategy serialization at intermediate checkpoints**: The strategy
   at 10M iterations for 184k info states is ~50 MB as JSON. Only save
   the final strategy, not intermediate ones.

5. **Progress logging**: Print after each checkpoint so the user can monitor
   the multi-hour run. Include wall-clock timing.

---

## Convergence Plot Spec

**Figure**: Log-log plot, exploitability (y) vs iterations (x).

- Two lines: vanilla (blue solid) and pONE (orange dashed)
- X-axis: log scale, 10k to 10M
- Y-axis: log scale, 0.001 to 1.0
- Horizontal reference line at epsilon = 0.002 (thesis result), labeled
- Horizontal reference line at epsilon = 0.156 (thesis baseline), labeled
- Grid, legend, axis labels with proper font sizes
- Save as PNG (300 dpi) and PDF (vector, for paper)

This is the **main result figure** in the paper.

### Plotting function outline

```python
import matplotlib.pyplot as plt

def plot_convergence(records, output_path):
    fig, ax = plt.subplots(figsize=(8, 5))

    for condition in ["vanilla", "pone"]:
        subset = [r for r in records if r["condition"] == condition]
        iters = [r["iterations"] for r in subset]
        expls = [r["exploitability"] for r in subset]
        label = "MCCFR" if condition == "vanilla" else "MCCFR + pONE"
        style = "-o" if condition == "vanilla" else "--s"
        ax.plot(iters, expls, style, label=label)

    ax.axhline(0.002, color="green", linestyle=":", alpha=0.7,
               label="Thesis best (0.002)")
    ax.axhline(0.156, color="red", linestyle=":", alpha=0.7,
               label="Thesis baseline (0.156)")

    ax.set_xscale("log")
    ax.set_yscale("log")
    ax.set_xlabel("MCCFR Iterations")
    ax.set_ylabel("Exploitability")
    ax.set_title("4x3 CDH Dark Hex: MCCFR Convergence")
    ax.legend()
    ax.grid(True, alpha=0.3)
    fig.tight_layout()
    fig.savefig(output_path, dpi=300)
    fig.savefig(output_path.with_suffix(".pdf"))
    plt.close(fig)
```

---

## Verification Plan

### V1: Sanity checks (before long run)

Run a quick 1,000-iteration warmup on 4x3 and verify:
- `solver.num_info_states() > 0` (solver is working)
- `solver.iterations() == 1000`
- `exploitability(4, 3, {})` returns a positive number (empty strategy check)
- `exploitability(4, 3, solver.get_average_strategy())` returns a value < 1.0

**If any fail**: Do not proceed to the full run. Debug first.

### V2: Monotonic convergence

Exploitability should be monotonically non-increasing across checkpoints
(with possible noise). If it ever INCREASES by more than 20% between
consecutive checkpoints, flag a warning (could indicate a bug or extreme
Outcome Sampling variance).

### V3: Info state count convergence

At the final checkpoint (10M iterations), the number of canonical info
states should be in the range [170,000, 190,000]. This confirms isomorphic
reduction is working correctly at this scale.

With pONE, the count should be lower (pruned subtrees are never visited),
but the reduction should match the ~20% figure from the thesis.

### V4: Cross-validation with 3x3

Before running 4x3, run a quick 3x3 exploitability check at 100k
iterations. This should match EXP-001 data (exploitability in the range
0.01-0.1). If it does not, the exploitability code has regressed.

### V5: Best response decomposition

At each checkpoint, verify `(br_black + br_white) / 2 == exploitability`
to within 1e-10 (floating point). The `best_response_values` function
returns all three.

### V6: Thesis comparison

Final exploitability should be in the same order of magnitude as the thesis
result (0.002). Acceptable range: 0.0005 to 0.01. If outside this range:
- Above 0.01: need more iterations or there is a bug
- Below 0.0005: our implementation may be more efficient (good news, investigate why)

---

## What @implementer Should Create

### New files

| File | Purpose | Est. lines |
|------|---------|------------|
| `experiments/exp003_4x3_mccfr.py` | Main experiment script | ~180 |
| `experiments/exp003_4x3_mccfr.md` | Copy of this plan, results filled in after | ~100 |

### No files to modify

The Rust engine, MCCFR solver, exploitability, and pONE are all implemented
and tested. This experiment is pure Python, calling existing Rust APIs.

### Dependencies (all already available)

```python
from darkhex._engine import (
    MCCFRSolver,        # MCCFR solver
    Sampling,           # Sampling.Outcome
    PoneDb,             # pONE database
    exploitability,     # exploitability(rows, cols, strategy, pone_db=None)
    best_response_values,  # (br_b, br_w, expl)
)
```

Plus standard library: `csv`, `json`, `time`, `pathlib`.
Plus matplotlib for the plot (already in the environment).

---

## Run Instructions

```bash
# Build Rust engine (release mode for performance)
make build

# Run the experiment (expect 3-5 hours)
cd /Users/bedirt/Documents/Github/darkhex
python experiments/exp003_4x3_mccfr.py

# Results will be in:
#   results/exp003_4x3_mccfr/convergence.csv
#   results/exp003_4x3_mccfr/full_results.json
#   results/exp003_4x3_mccfr/convergence.png
#   results/exp003_4x3_mccfr/convergence.pdf
```

---

## Risk Mitigation

### R1: Exploitability OOM on 4x3

The exploitability memoization table needs ~6.5 GB. If the machine has
less than 8 GB free, this will fail.

**Mitigation**: The script should print estimated memory before starting.
If memory is tight, reduce checkpoints (skip early ones where exploitability
is high and uninteresting). Minimum viable: measure at 1M, 5M, 10M only
(3 checkpoints per condition = 6 calls = ~66 min of exploitability).

### R2: Exploitability takes longer than estimated

The 11 min estimate is from the exploitability.md doc. If actual runtime
is >20 min per call, the full experiment exceeds 5 hours.

**Mitigation**: Time the first exploitability call. If it exceeds 20 min,
reduce to 5 checkpoints per condition:
`100k, 500k, 1M, 5M, 10M`

### R3: MCCFR throughput lower than estimated

If throughput is < 2,000 iter/s, the 10M solve takes > 80 min.

**Mitigation**: This is acceptable -- the solve is not the bottleneck.
But if throughput is < 500 iter/s, something is wrong (HashMap pathology,
canonicalization overhead). Print throughput after the first checkpoint
and abort if < 1,000 iter/s.

### R4: pONE precomputation fails or takes too long

Expected ~8-10 min. If it exceeds 30 min, abort and investigate.

**Mitigation**: The script runs pONE first. If it fails, the vanilla
condition can still run. Print timing and continue.

### R5: Intermediate results lost to crash

A 5-hour experiment crashing at hour 4 loses everything.

**Mitigation**: Write partial results after each checkpoint. The CSV
should be appended to (or overwritten with all records so far) after
every measurement. On restart, the script can skip already-measured
checkpoints by reading the existing CSV.

---

## Estimated Total Effort

| Task | Time |
|------|------|
| Implement exp003 script | 30-45 min (for @implementer) |
| Build and sanity check | 5 min |
| Full experiment run | 3-5 hours (background) |
| Review results + fill in exp003 md | 15 min |
| Generate/polish convergence figure | 15 min |
| **Total** | **~4-6 hours** (mostly unattended compute) |

---

## Paper Connection

This experiment produces:

1. **Table**: "4x3 CDH Convergence Results" -- exploitability at key
   iteration counts, with and without pONE. Compare against thesis.
2. **Figure**: Log-log convergence plot (the main result figure).
3. **Claim**: "Our Rust-based OS-MCCFR implementation matches the thesis
   result (epsilon = 0.002) on 4x3 CDH, with 2,300x speedup over the
   original Python implementation."
4. **Claim**: "pONE pruning reduces info state storage by ~20% and
   accelerates convergence by X% at matched iteration counts."

These support Section 5 (Experiments) and Section 6 (Results) of the paper.
