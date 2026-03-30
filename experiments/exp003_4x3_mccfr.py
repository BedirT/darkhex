"""EXP-003: MCCFR Convergence on 4x3 CDH Dark Hex.

Hypothesis: OS-MCCFR converges to exploitability < 0.01 on 4x3 CDH
within 10M iterations, matching the thesis result (epsilon 0.002).
pONE pruning accelerates convergence and reduces info state count.

Two conditions: vanilla (no pONE) and pONE-enabled.
Both use Outcome Sampling, epsilon=0.6, seed=42.
"""

import csv
import json
import sys
import time
from pathlib import Path

from darkhex._engine import (
    MCCFRSolver,
    PoneDb,
    Sampling,
    best_response_values,
)

RESULTS_DIR = Path("results/exp003_4x3_mccfr")
ROWS, COLS = 4, 3
SEED = 42
EPSILON = 0.6

CHECKPOINTS = [
    100_000,
    1_000_000,
    10_000_000,
    50_000_000,
    100_000_000,
]

CSV_FIELDS = [
    "condition",
    "iterations",
    "exploitability",
    "br_black",
    "br_white",
    "info_states",
    "solve_time_s",
    "expl_time_s",
    "throughput_iter_s",
]

MIN_THROUGHPUT = 1000  # iter/s — abort below this


# -- Sanity checks ----------------------------------------------------------


def sanity_check_3x3():
    """Quick 3x3 exploitability check to verify the pipeline works."""
    print("Sanity check: 3x3 @ 1000 iters...")
    solver = MCCFRSolver(3, 3, Sampling.Outcome, epsilon=EPSILON, seed=SEED)
    solver.solve(1000)
    strategy = solver.get_average_strategy()
    br_b, br_w, expl = best_response_values(3, 3, strategy)
    print(f"  expl={expl:.6f}  br_b={br_b:.6f}  br_w={br_w:.6f}")
    if expl <= 0 or expl > 2.0:
        print("ABORT: exploitability out of range, something is broken.")
        sys.exit(1)
    print("  OK\n")


# -- pONE precomputation -----------------------------------------------------


def build_pone_db():
    """One-time pONE precomputation."""
    print(f"Building pONE database for {ROWS}x{COLS}...")
    t0 = time.time()
    db = PoneDb(ROWS, COLS)
    elapsed = time.time() - t0
    print(f"  pONE: {db.len()} states, {elapsed:.1f}s\n")
    return db, elapsed


# -- CSV I/O -----------------------------------------------------------------


def write_csv(records, path):
    """Write all records so far to CSV (overwrite for crash resilience)."""
    with open(path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=CSV_FIELDS)
        writer.writeheader()
        for r in records:
            writer.writerow({k: r[k] for k in CSV_FIELDS})


# -- Core experiment ---------------------------------------------------------


def run_condition(name, all_records, csv_path, pone_db=None):
    """Run MCCFR with checkpointed exploitability measurements."""
    solver = MCCFRSolver(
        ROWS, COLS, Sampling.Outcome, epsilon=EPSILON, seed=SEED
    )
    if pone_db is not None:
        solver.set_pone_db(pone_db)

    prev_iters = 0
    total_solve_time = 0.0
    prev_expl = None

    for i, target in enumerate(CHECKPOINTS):
        delta = target - prev_iters

        # MCCFR solve phase
        t0 = time.time()
        solver.solve(delta)
        solve_time = time.time() - t0
        total_solve_time += solve_time
        prev_iters = target

        # Throughput guard after first checkpoint
        throughput = target / total_solve_time
        if i == 0 and throughput < MIN_THROUGHPUT:
            print(
                f"ABORT: throughput {throughput:.0f} iter/s < "
                f"{MIN_THROUGHPUT} minimum after first checkpoint."
            )
            sys.exit(1)

        # Exploitability measurement (NO pone_db — would bias result)
        strategy = solver.get_average_strategy()
        t_expl = time.time()
        br_b, br_w, expl = best_response_values(ROWS, COLS, strategy)
        expl_time = time.time() - t_expl

        # Convergence warning
        if prev_expl is not None and expl > prev_expl * 1.2:
            print(
                f"  WARNING: exploitability increased by "
                f"{(expl / prev_expl - 1) * 100:.1f}% "
                f"({prev_expl:.6f} -> {expl:.6f})"
            )
        prev_expl = expl

        record = {
            "condition": name,
            "iterations": target,
            "exploitability": expl,
            "br_black": br_b,
            "br_white": br_w,
            "info_states": solver.num_info_states(),
            "solve_time_s": round(total_solve_time, 1),
            "expl_time_s": round(expl_time, 1),
            "throughput_iter_s": round(throughput, 0),
        }
        all_records.append(record)

        # Crash-resilient: write CSV after every checkpoint
        write_csv(all_records, csv_path)

        print(
            f"  {name} @ {target:>10,}: "
            f"expl={expl:.6f}  "
            f"info_states={solver.num_info_states():>7,}  "
            f"solve={total_solve_time:.0f}s  "
            f"expl_compute={expl_time:.0f}s  "
            f"throughput={throughput:.0f} it/s"
        )

    return solver


# -- Plotting ----------------------------------------------------------------


def plot_convergence(records, output_path):
    """Generate log-log convergence plot (publication quality)."""
    import matplotlib.pyplot as plt

    fig, ax = plt.subplots(figsize=(8, 5))

    styles = {
        "vanilla": {"fmt": "-o", "color": "#1f77b4", "label": "MCCFR"},
        "pone": {
            "fmt": "--s",
            "color": "#ff7f0e",
            "label": "MCCFR + pONE",
        },
    }

    for condition, style in styles.items():
        subset = [r for r in records if r["condition"] == condition]
        if not subset:
            continue
        iters = [r["iterations"] for r in subset]
        expls = [r["exploitability"] for r in subset]
        ax.plot(
            iters,
            expls,
            style["fmt"],
            color=style["color"],
            label=style["label"],
            markersize=5,
            linewidth=1.5,
        )

    # Thesis reference lines
    ax.axhline(
        0.002,
        color="green",
        linestyle=":",
        alpha=0.7,
        label="Thesis best (0.002)",
    )
    ax.axhline(
        0.156,
        color="red",
        linestyle=":",
        alpha=0.7,
        label="Thesis baseline (0.156)",
    )

    ax.set_xscale("log")
    ax.set_yscale("log")
    ax.set_xlabel("MCCFR Iterations", fontsize=12)
    ax.set_ylabel("Exploitability", fontsize=12)
    ax.set_title("4x3 CDH Dark Hex: MCCFR Convergence", fontsize=14)
    ax.legend(fontsize=10)
    ax.grid(True, which="both", alpha=0.3)
    ax.tick_params(labelsize=10)
    fig.tight_layout()
    fig.savefig(output_path, dpi=300, bbox_inches="tight")
    fig.savefig(
        output_path.with_suffix(".pdf"), bbox_inches="tight"
    )
    plt.close(fig)
    print(f"Plot saved to {output_path} and {output_path.with_suffix('.pdf')}")


# -- Main --------------------------------------------------------------------


def main():
    print("=== EXP-003: MCCFR on 4x3 CDH ===")
    print(f"Config: rows={ROWS} cols={COLS} seed={SEED} epsilon={EPSILON}")
    print(f"Checkpoints: {CHECKPOINTS}\n")

    # Phase 0: Sanity check
    sanity_check_3x3()

    # Phase 1: pONE precomputation
    pone_db, pone_time = build_pone_db()

    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    csv_path = RESULTS_DIR / "convergence.csv"
    all_records = []

    # Phase 2: Vanilla condition
    print("--- Condition A: Vanilla (no pONE) ---")
    t_vanilla = time.time()
    vanilla_solver = run_condition("vanilla", all_records, csv_path)
    vanilla_time = time.time() - t_vanilla

    # Phase 3: pONE condition
    print("\n--- Condition B: pONE pruning ---")
    t_pone = time.time()
    pone_solver = run_condition("pone", all_records, csv_path, pone_db)
    pone_time_total = time.time() - t_pone

    # Phase 4: Save full results
    full_results = {
        "experiment": "exp003_4x3_mccfr",
        "config": {
            "rows": ROWS,
            "cols": COLS,
            "seed": SEED,
            "epsilon": EPSILON,
            "sampling": "Outcome",
            "checkpoints": CHECKPOINTS,
        },
        "pone_precompute_time_s": round(pone_time, 1),
        "pone_states": pone_db.len(),
        "vanilla_total_time_s": round(vanilla_time, 1),
        "pone_total_time_s": round(pone_time_total, 1),
        "vanilla_final_info_states": vanilla_solver.num_info_states(),
        "pone_final_info_states": pone_solver.num_info_states(),
        "records": all_records,
    }

    json_path = RESULTS_DIR / "full_results.json"
    with open(json_path, "w") as f:
        json.dump(full_results, f, indent=2)

    # Phase 5: Generate convergence plot
    plot_convergence(all_records, RESULTS_DIR / "convergence.png")

    # Summary
    print("\n=== Summary ===")
    print(f"pONE precomputation: {pone_time:.1f}s, {pone_db.len()} states")
    print(
        f"Vanilla: {vanilla_solver.num_info_states():,} info states, "
        f"{vanilla_time:.0f}s total"
    )
    print(
        f"pONE:    {pone_solver.num_info_states():,} info states, "
        f"{pone_time_total:.0f}s total"
    )
    van_final = [
        r for r in all_records if r["condition"] == "vanilla"
    ][-1]
    pone_final = [
        r for r in all_records if r["condition"] == "pone"
    ][-1]
    print(
        f"Final exploitability: "
        f"vanilla={van_final['exploitability']:.6f}, "
        f"pone={pone_final['exploitability']:.6f}"
    )
    print(f"\nResults: {RESULTS_DIR}/")


if __name__ == "__main__":
    main()
