"""EXP-003: MCCFR Convergence on 4x3 CDH Dark Hex.

Trains OS-MCCFR on 4x3 CDH with solver checkpointing. Supports
resume from a saved checkpoint to continue training incrementally.

Usage:
    # Fresh run
    uv run python experiments/exp003_4x3_mccfr.py

    # Resume from checkpoint
    uv run python experiments/exp003_4x3_mccfr.py --resume results/exp003_4x3_mccfr/solver_100M.bin

    # Custom iteration target
    uv run python experiments/exp003_4x3_mccfr.py --max-iters 10000000000
"""

import argparse
import csv
import json
import sys
import time
from pathlib import Path

from darkhex._engine import (
    MCCFRSolver,
    Sampling,
    best_response_values,
)

RESULTS_DIR = Path("results/exp003_4x3_mccfr")
ROWS, COLS = 4, 3
SEED = 42
EPSILON = 0.6

DEFAULT_CHECKPOINTS = [
    1_000_000,
    10_000_000,
    100_000_000,
    500_000_000,
    1_000_000_000,
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


# -- CSV I/O -----------------------------------------------------------------


def write_csv(records, path):
    """Write all records so far to CSV (overwrite for crash resilience)."""
    with open(path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=CSV_FIELDS)
        writer.writeheader()
        for r in records:
            writer.writerow({k: r[k] for k in CSV_FIELDS})


# -- Core experiment ---------------------------------------------------------


def run_condition(name, solver, checkpoints, all_records, csv_path):
    """Run MCCFR with checkpointed exploitability and solver saves."""
    prev_iters = solver.iterations()
    total_solve_time = 0.0
    prev_expl = None

    for i, target in enumerate(checkpoints):
        if target <= prev_iters:
            continue  # Skip checkpoints already completed (resume case)

        delta = target - prev_iters

        # MCCFR solve phase
        t0 = time.time()
        solver.solve(delta)
        solve_time = time.time() - t0
        total_solve_time += solve_time
        prev_iters = target

        # Throughput guard after first actual solve
        throughput = delta / solve_time if solve_time > 0 else float("inf")
        if i == 0 and total_solve_time > 0 and throughput < MIN_THROUGHPUT:
            print(
                f"ABORT: throughput {throughput:.0f} iter/s < "
                f"{MIN_THROUGHPUT} minimum after first checkpoint."
            )
            sys.exit(1)

        # Save solver checkpoint
        checkpoint_path = RESULTS_DIR / f"solver_{_fmt_iters(target)}.bin"
        solver.save(str(checkpoint_path))

        # Exploitability measurement
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

        overall_throughput = target / total_solve_time if total_solve_time > 0 else 0
        record = {
            "condition": name,
            "iterations": target,
            "exploitability": expl,
            "br_black": br_b,
            "br_white": br_w,
            "info_states": solver.num_info_states(),
            "solve_time_s": round(total_solve_time, 1),
            "expl_time_s": round(expl_time, 1),
            "throughput_iter_s": round(overall_throughput, 0),
        }
        all_records.append(record)

        # Crash-resilient: write CSV after every checkpoint
        write_csv(all_records, csv_path)

        print(
            f"  {name} @ {target:>13,}: "
            f"expl={expl:.6f}  "
            f"info_states={solver.num_info_states():>7,}  "
            f"solve={total_solve_time:.0f}s  "
            f"expl_compute={expl_time:.0f}s  "
            f"throughput={overall_throughput:.0f} it/s  "
            f"[saved {checkpoint_path.name}]"
        )

    return solver


def _fmt_iters(n):
    """Format iteration count for filenames: 1000000 -> '1M'."""
    if n >= 1_000_000_000 and n % 1_000_000_000 == 0:
        return f"{n // 1_000_000_000}B"
    if n >= 1_000_000 and n % 1_000_000 == 0:
        return f"{n // 1_000_000}M"
    if n >= 1_000 and n % 1_000 == 0:
        return f"{n // 1_000}k"
    return str(n)


# -- Plotting ----------------------------------------------------------------


def plot_convergence(records, output_path):
    """Generate log-log convergence plot (publication quality)."""
    import matplotlib.pyplot as plt

    fig, ax = plt.subplots(figsize=(8, 5))

    styles = {
        "vanilla": {"fmt": "-o", "color": "#1f77b4", "label": "OS-MCCFR"},
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
        label="Thesis best (Ab-BR, 0.002)",
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
    ax.set_ylabel("Exploitability (Clairvoyant BR)", fontsize=12)
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
    parser = argparse.ArgumentParser(description="EXP-003: MCCFR on 4x3 CDH")
    parser.add_argument(
        "--resume", type=str, default=None,
        help="Path to solver checkpoint to resume from",
    )
    parser.add_argument(
        "--max-iters", type=int, default=None,
        help="Override max iteration target",
    )
    args = parser.parse_args()

    checkpoints = list(DEFAULT_CHECKPOINTS)
    if args.max_iters:
        checkpoints = [c for c in checkpoints if c <= args.max_iters]
        if not checkpoints or checkpoints[-1] != args.max_iters:
            checkpoints.append(args.max_iters)

    print("=== EXP-003: MCCFR on 4x3 CDH ===")
    print(f"Config: rows={ROWS} cols={COLS} seed={SEED} epsilon={EPSILON}")
    print(f"Checkpoints: {[_fmt_iters(c) for c in checkpoints]}")

    if args.resume:
        print(f"Resuming from: {args.resume}")
        solver = MCCFRSolver.load(args.resume)
        print(f"  Loaded: {solver.iterations():,} iters, "
              f"{solver.num_info_states():,} info states\n")
    else:
        # Phase 0: Sanity check (only on fresh runs)
        sanity_check_3x3()
        solver = MCCFRSolver(
            ROWS, COLS, Sampling.Outcome, epsilon=EPSILON, seed=SEED
        )

    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    csv_path = RESULTS_DIR / "convergence.csv"
    all_records = []

    # Run
    print(f"--- OS-MCCFR (target: {_fmt_iters(checkpoints[-1])}) ---")
    t_start = time.time()
    solver = run_condition("vanilla", solver, checkpoints, all_records, csv_path)
    total_time = time.time() - t_start

    # Save full results JSON
    full_results = {
        "experiment": "exp003_4x3_mccfr",
        "config": {
            "rows": ROWS,
            "cols": COLS,
            "seed": SEED,
            "epsilon": EPSILON,
            "sampling": "Outcome",
            "checkpoints": checkpoints,
        },
        "total_time_s": round(total_time, 1),
        "final_info_states": solver.num_info_states(),
        "final_iterations": solver.iterations(),
        "records": all_records,
    }

    json_path = RESULTS_DIR / "full_results.json"
    with open(json_path, "w") as f:
        json.dump(full_results, f, indent=2)

    # Generate convergence plot
    plot_convergence(all_records, RESULTS_DIR / "convergence.png")

    # Summary
    final = all_records[-1] if all_records else {}
    print(f"\n=== Summary ===")
    print(f"Iterations: {solver.iterations():,}")
    print(f"Info states: {solver.num_info_states():,}")
    print(f"Exploitability: {final.get('exploitability', 'N/A')}")
    print(f"Total time: {total_time:.0f}s")
    print(f"Latest checkpoint: {RESULTS_DIR}/solver_{_fmt_iters(solver.iterations())}.bin")
    print(f"\nResults: {RESULTS_DIR}/")


if __name__ == "__main__":
    main()
