"""EXP-004: Ab-BR vs Clairvoyant BR Comparison on 4x3 CDH.

Trains OS-MCCFR on 4x3 CDH and computes both Abstract Best Response
(Ab-BR) and clairvoyant Best Response exploitability at each checkpoint,
enabling direct comparison with thesis results (Ab-BR = 0.002 at 1B).

Usage:
    # Fresh run
    uv run python experiments/exp004_ab_br_comparison.py

    # Resume from checkpoint
    uv run python experiments/exp004_ab_br_comparison.py \
        --resume results/exp004_ab_br/solver_100M.bin

    # Custom iteration target
    uv run python experiments/exp004_ab_br_comparison.py --max-iters 100000000
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
    ab_best_response_values,
    best_response_values,
)

RESULTS_DIR = Path("results/exp004_ab_br")
ROWS, COLS = 4, 3
SEED = 42
EPSILON = 0.6

DEFAULT_CHECKPOINTS = [
    1_000_000,
    10_000_000,
    100_000_000,
    500_000_000,
    1_000_000_000,
    2_000_000_000,
    5_000_000_000,
    10_000_000_000,
]

CSV_FIELDS = [
    "iterations",
    "ab_exploitability",
    "ab_br_black",
    "ab_br_white",
    "cl_exploitability",
    "cl_br_black",
    "cl_br_white",
    "info_states",
    "solve_time_s",
    "ab_time_s",
    "cl_time_s",
    "throughput_iter_s",
]

MIN_THROUGHPUT = 1000


def sanity_check_2x2():
    """Quick 2x2 check: both Ab-BR and clairvoyant should be computable."""
    print("Sanity check: 2x2 @ 1000 iters...")
    solver = MCCFRSolver(2, 2, Sampling.Outcome, epsilon=EPSILON, seed=SEED)
    solver.solve(1000)
    strategy = solver.get_average_strategy()
    ab_b, ab_w, ab_expl = ab_best_response_values(2, 2, strategy)
    cl_b, cl_w, cl_expl = best_response_values(2, 2, strategy)
    print(f"  Ab-BR: expl={ab_expl:.6f}  br_b={ab_b:.6f}  br_w={ab_w:.6f}")
    print(f"  Clair: expl={cl_expl:.6f}  br_b={cl_b:.6f}  br_w={cl_w:.6f}")
    if not (ab_expl == ab_expl and cl_expl == cl_expl):  # NaN check
        print("ABORT: NaN in exploitability.")
        sys.exit(1)
    print("  OK\n")


def write_csv(records, path):
    """Write all records to CSV (overwrite for crash resilience)."""
    with open(path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=CSV_FIELDS)
        writer.writeheader()
        for r in records:
            writer.writerow({k: r[k] for k in CSV_FIELDS})


def _fmt_iters(n):
    """Format iteration count: 1000000 -> '1M'."""
    if n >= 1_000_000_000 and n % 1_000_000_000 == 0:
        return f"{n // 1_000_000_000}B"
    if n >= 1_000_000 and n % 1_000_000 == 0:
        return f"{n // 1_000_000}M"
    if n >= 1_000 and n % 1_000 == 0:
        return f"{n // 1_000}k"
    return str(n)


def run_experiment(solver, checkpoints, all_records, csv_path):
    """Run MCCFR with dual-metric checkpointing."""
    prev_iters = solver.iterations()
    total_solve_time = 0.0

    for i, target in enumerate(checkpoints):
        if target <= prev_iters:
            continue

        delta = target - prev_iters

        # MCCFR solve
        t0 = time.time()
        solver.solve(delta)
        solve_time = time.time() - t0
        total_solve_time += solve_time
        prev_iters = target

        # Throughput guard
        throughput = delta / solve_time if solve_time > 0 else float("inf")
        if i == 0 and total_solve_time > 0 and throughput < MIN_THROUGHPUT:
            print(
                f"ABORT: throughput {throughput:.0f} iter/s < "
                f"{MIN_THROUGHPUT} minimum."
            )
            sys.exit(1)

        # Save solver checkpoint
        checkpoint_path = RESULTS_DIR / f"solver_{_fmt_iters(target)}.bin"
        solver.save(str(checkpoint_path))

        strategy = solver.get_average_strategy()

        # Ab-BR exploitability
        t_ab = time.time()
        ab_b, ab_w, ab_expl = ab_best_response_values(ROWS, COLS, strategy)
        ab_time = time.time() - t_ab

        # Clairvoyant BR exploitability
        t_cl = time.time()
        cl_b, cl_w, cl_expl = best_response_values(ROWS, COLS, strategy)
        cl_time = time.time() - t_cl

        overall_throughput = (
            target / total_solve_time if total_solve_time > 0 else 0
        )
        record = {
            "iterations": target,
            "ab_exploitability": ab_expl,
            "ab_br_black": ab_b,
            "ab_br_white": ab_w,
            "cl_exploitability": cl_expl,
            "cl_br_black": cl_b,
            "cl_br_white": cl_w,
            "info_states": solver.num_info_states(),
            "solve_time_s": round(total_solve_time, 1),
            "ab_time_s": round(ab_time, 1),
            "cl_time_s": round(cl_time, 1),
            "throughput_iter_s": round(overall_throughput, 0),
        }
        all_records.append(record)
        write_csv(all_records, csv_path)

        print(
            f"  @ {target:>13,}:  "
            f"Ab-BR={ab_expl:.6f}  Clair={cl_expl:.6f}  "
            f"info_states={solver.num_info_states():>7,}  "
            f"solve={total_solve_time:.0f}s  "
            f"ab={ab_time:.0f}s  cl={cl_time:.0f}s  "
            f"[saved {checkpoint_path.name}]"
        )

    return solver


def plot_convergence(records, output_path):
    """Dual-line log-log convergence plot."""
    import matplotlib.pyplot as plt

    fig, ax = plt.subplots(figsize=(8, 5))

    iters = [r["iterations"] for r in records]
    ab_expls = [r["ab_exploitability"] for r in records]
    cl_expls = [r["cl_exploitability"] for r in records]

    ax.plot(
        iters, ab_expls, "-o", color="#1f77b4", label="Ab-BR",
        markersize=5, linewidth=1.5,
    )
    ax.plot(
        iters, cl_expls, "-s", color="#d62728", label="Clairvoyant BR",
        markersize=5, linewidth=1.5,
    )

    # Thesis reference lines
    ax.axhline(
        0.002, color="green", linestyle=":", alpha=0.7,
        label="Thesis Ab-BR (0.002)",
    )
    ax.axhline(
        0.156, color="orange", linestyle=":", alpha=0.7,
        label="Thesis baseline (0.156)",
    )

    ax.set_xscale("log")
    ax.set_yscale("log")
    ax.set_xlabel("MCCFR Iterations", fontsize=12)
    ax.set_ylabel("Exploitability", fontsize=12)
    ax.set_title("4x3 CDH: Ab-BR vs Clairvoyant BR", fontsize=14)
    ax.legend(fontsize=10)
    ax.grid(True, which="both", alpha=0.3)
    ax.tick_params(labelsize=10)
    fig.tight_layout()
    fig.savefig(output_path, dpi=300, bbox_inches="tight")
    fig.savefig(output_path.with_suffix(".pdf"), bbox_inches="tight")
    plt.close(fig)
    print(f"Plot saved to {output_path} and {output_path.with_suffix('.pdf')}")


def main():
    parser = argparse.ArgumentParser(
        description="EXP-004: Ab-BR vs Clairvoyant BR on 4x3 CDH",
    )
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

    print("=== EXP-004: Ab-BR vs Clairvoyant BR on 4x3 CDH ===")
    print(f"Config: rows={ROWS} cols={COLS} seed={SEED} epsilon={EPSILON}")
    print(f"Checkpoints: {[_fmt_iters(c) for c in checkpoints]}")

    if args.resume:
        print(f"Resuming from: {args.resume}")
        solver = MCCFRSolver.load(args.resume)
        print(
            f"  Loaded: {solver.iterations():,} iters, "
            f"{solver.num_info_states():,} info states\n"
        )
    else:
        sanity_check_2x2()
        solver = MCCFRSolver(
            ROWS, COLS, Sampling.Outcome, epsilon=EPSILON, seed=SEED,
        )

    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    csv_path = RESULTS_DIR / "convergence.csv"
    all_records = []

    print(f"--- Training to {_fmt_iters(checkpoints[-1])} ---")
    t_start = time.time()
    solver = run_experiment(solver, checkpoints, all_records, csv_path)
    total_time = time.time() - t_start

    # Save full results
    full_results = {
        "experiment": "exp004_ab_br_comparison",
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

    # Plot
    if all_records:
        plot_convergence(all_records, RESULTS_DIR / "convergence.png")

    # Summary
    final = all_records[-1] if all_records else {}
    print("\n=== Summary ===")
    print(f"Iterations: {solver.iterations():,}")
    print(f"Info states: {solver.num_info_states():,}")
    print(f"Ab-BR exploitability: {final.get('ab_exploitability', 'N/A')}")
    print(f"Clairvoyant exploitability: {final.get('cl_exploitability', 'N/A')}")
    print(f"Total time: {total_time:.0f}s")
    print(f"\nResults: {RESULTS_DIR}/")


if __name__ == "__main__":
    main()
