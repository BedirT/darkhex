"""EXP-001: MCCFR Convergence Verification on 2x2, 3x2, 3x3 CDH.

Hypothesis: Outcome Sampling MCCFR converges to Nash equilibrium and
discovers all info states on small Dark Hex boards.

Evaluation: Compare discovered info states against exhaustive enumeration.
Record convergence of initial state strategies across iteration counts.
"""

import csv
import json
import time
from pathlib import Path

from darkhex._engine import MCCFRSolver, Sampling, enumerate_game_tree

RESULTS_DIR = Path("results/exp001_mccfr_verification")
SEED = 42


def run_board(rows, cols, iteration_counts):
    """Run MCCFR at multiple iteration counts, record results."""
    board_key = f"{rows}x{cols}"
    ground_truth = enumerate_game_tree(rows, cols)
    records = []

    for n_iters in iteration_counts:
        solver = MCCFRSolver(rows, cols, Sampling.Outcome, epsilon=0.6, seed=SEED)
        t0 = time.time()
        solver.solve(n_iters)
        wall_time = time.time() - t0

        strategy = solver.get_average_strategy()
        initial_p0 = "P0\n" + "\n".join(
            "." * cols for _ in range(rows)
        )
        initial_p1 = "P1\n" + "\n".join(
            "." * cols for _ in range(rows)
        )
        p0_strat = dict(strategy.get(initial_p0, []))
        p1_strat = dict(strategy.get(initial_p1, []))

        records.append({
            "board": board_key,
            "iterations": n_iters,
            "info_states_found": solver.num_info_states(),
            "info_states_ground_truth": ground_truth.total_info_states,
            "coverage_pct": solver.num_info_states()
            / ground_truth.total_info_states
            * 100,
            "wall_time_s": round(wall_time, 3),
            "iters_per_sec": round(n_iters / wall_time, 0),
            "p0_initial_strategy": p0_strat,
            "p1_initial_strategy": p1_strat,
        })
        print(
            f"  {board_key} @ {n_iters:>8,}: "
            f"{solver.num_info_states():>6}/{ground_truth.total_info_states} "
            f"({records[-1]['coverage_pct']:.1f}%)  "
            f"{wall_time:.2f}s"
        )

    return records


def main():
    all_records = []

    print("=== EXP-001: MCCFR Convergence Verification ===\n")

    print("2x2 CDH:")
    all_records.extend(
        run_board(2, 2, [100, 1_000, 10_000, 100_000])
    )

    print("\n3x2 CDH:")
    all_records.extend(
        run_board(3, 2, [1_000, 10_000, 50_000, 100_000])
    )

    print("\n3x3 CDH:")
    all_records.extend(
        run_board(3, 3, [1_000, 10_000, 50_000, 100_000, 500_000])
    )

    # Save CSV (machine-readable, for plots)
    csv_path = RESULTS_DIR / "convergence.csv"
    with open(csv_path, "w", newline="") as f:
        writer = csv.DictWriter(
            f,
            fieldnames=[
                "board",
                "iterations",
                "info_states_found",
                "info_states_ground_truth",
                "coverage_pct",
                "wall_time_s",
                "iters_per_sec",
            ],
        )
        writer.writeheader()
        for r in all_records:
            row = {k: v for k, v in r.items() if k in writer.fieldnames}
            writer.writerow(row)

    # Save full JSON (includes strategies)
    json_path = RESULTS_DIR / "full_results.json"
    with open(json_path, "w") as f:
        json.dump(all_records, f, indent=2, default=str)

    print(f"\nResults saved to {csv_path} and {json_path}")


if __name__ == "__main__":
    main()
