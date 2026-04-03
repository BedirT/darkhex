"""EXP-005: DREAM on Dark Hex (Outcome Sampling Neural CFR).

Tests whether DREAM can converge on boards where External Sampling
(Deep CFR) is infeasible, particularly 4x3 CDH.

Usage:
    uv run python experiments/exp005_dream.py --board 2x2
    uv run python experiments/exp005_dream.py --board 4x3
    uv run python experiments/exp005_dream.py --board 3x3 --baseline
"""

from __future__ import annotations

import argparse
import csv
import json
import time
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

from darkhex.algorithms.dream import DREAM, DREAMConfig


# ---------------------------------------------------------------------------
# Per-board configurations
# ---------------------------------------------------------------------------

CONFIGS: dict[str, dict] = {
    "2x2": dict(
        rows=2, cols=2,
        hidden_sizes=(64, 64),
        num_cfr_iters=100,
        num_traversals=200,
        advantage_train_steps=100,
        strategy_train_steps=500,
        buffer_size=50_000,
        reinit_every=10,
    ),
    "3x2": dict(
        rows=3, cols=2,
        hidden_sizes=(64, 64),
        num_cfr_iters=100,
        num_traversals=200,
        advantage_train_steps=100,
        strategy_train_steps=500,
        buffer_size=100_000,
        reinit_every=10,
    ),
    "3x3": dict(
        rows=3, cols=3,
        hidden_sizes=(128, 128),
        num_cfr_iters=100,
        num_traversals=500,
        advantage_train_steps=200,
        strategy_train_steps=1000,
        buffer_size=500_000,
        reinit_every=10,
    ),
    "4x3": dict(
        rows=4, cols=3,
        hidden_sizes=(128, 128),
        num_cfr_iters=200,
        num_traversals=1000,
        advantage_train_steps=500,
        strategy_train_steps=2500,
        buffer_size=1_000_000,
        reinit_every=10,
    ),
}

BASELINE_OVERRIDES = dict(
    use_baseline=True,
    baseline_hidden_sizes=(128, 128),
    baseline_buffer_size=200_000,
    baseline_train_steps=500,
    baseline_batch_size=256,
)


def run_experiment(
    board: str,
    use_baseline: bool = False,
    seed: int = 42,
) -> list[dict]:
    """Run DREAM on the given board size and collect convergence data."""

    if board not in CONFIGS:
        raise ValueError(f"Unknown board: {board}. Choose from {list(CONFIGS)}")

    params = {**CONFIGS[board], "seed": seed}
    if use_baseline:
        params.update(BASELINE_OVERRIDES)

    cfg = DREAMConfig(**params)
    solver = DREAM(cfg)

    condition = "dream_bl" if use_baseline else "dream"
    records: list[dict] = []

    print(f"\n{'='*60}")
    print(f"EXP-005: DREAM on {board} CDH (condition={condition})")
    print(f"  traversals/iter: {cfg.num_traversals}")
    print(f"  epsilon: {cfg.epsilon}")
    print(f"  reinit_every: {cfg.reinit_every}")
    print(f"  baseline: {cfg.use_baseline}")
    print(f"{'='*60}\n")

    total_time = 0.0

    for cfr_iter in range(1, cfg.num_cfr_iters + 1):
        t0 = time.time()
        solver.solve(1)
        iter_time = time.time() - t0
        total_time += iter_time

        # Measure exploitability at logarithmic intervals
        # (more frequent early, sparser later)
        should_measure = (
            cfr_iter <= 10
            or cfr_iter % 5 == 0 and cfr_iter <= 50
            or cfr_iter % 10 == 0 and cfr_iter <= 200
            or cfr_iter % 50 == 0
        )

        if should_measure or cfr_iter == cfg.num_cfr_iters:
            t_expl = time.time()
            expl = solver.exploitability()
            expl_time = time.time() - t_expl

            strategy = solver.extract_strategy()

            record = {
                "condition": condition,
                "cfr_iter": cfr_iter,
                "exploitability": expl,
                "info_states_in_strategy": len(strategy),
                "adv_buf_p0": len(solver._advantage_buffers[0]),
                "adv_buf_p1": len(solver._advantage_buffers[1]),
                "strat_buf": len(solver._strategy_buffer),
                "iter_time_s": round(iter_time, 2),
                "total_time_s": round(total_time, 1),
                "expl_time_s": round(expl_time, 2),
            }
            records.append(record)

            print(
                f"  iter {cfr_iter:>4d}: "
                f"expl={expl:.6f}  "
                f"info_states={len(strategy):>6d}  "
                f"time={total_time:.0f}s  "
                f"expl_compute={expl_time:.1f}s"
            )

    return records


def save_results(
    records: list[dict],
    output_dir: Path,
    board: str,
) -> None:
    """Save results to CSV, JSON, and generate convergence plot."""
    output_dir.mkdir(parents=True, exist_ok=True)

    # CSV
    csv_path = output_dir / "convergence.csv"
    if records:
        fieldnames = list(records[0].keys())
        with open(csv_path, "w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames)
            writer.writeheader()
            writer.writerows(records)
        print(f"\nCSV saved: {csv_path}")

    # JSON
    json_path = output_dir / "full_results.json"
    with open(json_path, "w") as f:
        json.dump(
            {"board": board, "records": records},
            f, indent=2,
        )
    print(f"JSON saved: {json_path}")

    # Convergence plot
    plot_convergence(records, output_dir / "convergence.png", board)


def plot_convergence(
    records: list[dict],
    output_path: Path,
    board: str,
) -> None:
    """Generate a convergence plot."""
    fig, ax = plt.subplots(figsize=(8, 5))

    conditions = sorted(set(r["condition"] for r in records))
    styles = {"dream": "-o", "dream_bl": "--s"}
    labels = {"dream": "DREAM (no baseline)", "dream_bl": "DREAM + Q-baseline"}

    for cond in conditions:
        subset = [r for r in records if r["condition"] == cond]
        iters = [r["cfr_iter"] for r in subset]
        expls = [r["exploitability"] for r in subset]
        ax.plot(
            iters, expls,
            styles.get(cond, "-o"),
            label=labels.get(cond, cond),
            markersize=4,
        )

    ax.set_xlabel("CFR Iteration")
    ax.set_ylabel("Exploitability")
    ax.set_title(f"{board} CDH Dark Hex: DREAM Convergence")
    ax.legend()
    ax.grid(True, alpha=0.3)

    if max(r["exploitability"] for r in records) > 0.1:
        ax.set_yscale("log")

    fig.tight_layout()
    fig.savefig(output_path, dpi=300)
    fig.savefig(output_path.with_suffix(".pdf"))
    plt.close(fig)
    print(f"Plot saved: {output_path}")


def main():
    parser = argparse.ArgumentParser(description="EXP-005: DREAM on Dark Hex")
    parser.add_argument(
        "--board", choices=list(CONFIGS.keys()), default="2x2",
        help="Board size to run",
    )
    parser.add_argument(
        "--baseline", action="store_true",
        help="Enable Q-baseline for variance reduction",
    )
    parser.add_argument("--seed", type=int, default=42)
    args = parser.parse_args()

    condition = "dream_bl" if args.baseline else "dream"
    output_dir = Path(f"results/exp005_dream/{args.board}_{condition}")

    records = run_experiment(args.board, args.baseline, args.seed)
    save_results(records, output_dir, args.board)


if __name__ == "__main__":
    main()
