"""EXP-004: Deep CFR Prototype on Dark Hex.

Trains Deep CFR (Brown et al., ICML 2019) on 2x2 and 4x3 CDH Dark Hex.
Measures exploitability at each CFR iteration and compares against tabular
OS-MCCFR baseline.

Usage:
    # 2x2 verification (fast)
    uv run python experiments/exp004_deep_cfr_prototype.py --board 2x2

    # 4x3 experiment
    uv run python experiments/exp004_deep_cfr_prototype.py --board 4x3

    # Custom settings
    uv run python experiments/exp004_deep_cfr_prototype.py \
        --board 4x3 --iters 200 --traversals 500
"""

import argparse
import csv
import json
import sys
import time
from pathlib import Path

from darkhex.algorithms.deep_cfr import DeepCFR, DeepCFRConfig

RESULTS_DIR = Path("results/exp004_deep_cfr")

# Board-specific defaults
CONFIGS = {
    "2x2": dict(
        rows=2,
        cols=2,
        hidden_sizes=(64, 64),
        num_traversals=100,
        advantage_train_steps=100,
        strategy_train_steps=500,
        buffer_size=50_000,
    ),
    "3x2": dict(
        rows=3,
        cols=2,
        hidden_sizes=(64, 64),
        num_traversals=100,
        advantage_train_steps=100,
        strategy_train_steps=500,
        buffer_size=100_000,
    ),
    "3x3": dict(
        rows=3,
        cols=3,
        hidden_sizes=(128, 128),
        num_traversals=5,
        advantage_train_steps=200,
        strategy_train_steps=1000,
        buffer_size=500_000,
    ),
    "4x3": dict(
        rows=4,
        cols=3,
        hidden_sizes=(128, 128),
        num_traversals=500,
        advantage_train_steps=500,
        strategy_train_steps=3000,
        buffer_size=1_000_000,
    ),
}

CSV_FIELDS = [
    "cfr_iter",
    "iteration",
    "exploitability",
    "br_black",
    "br_white",
    "info_states_in_strategy",
    "adv_buf_0",
    "adv_buf_1",
    "strat_buf",
    "iter_time_s",
    "expl_time_s",
    "cumulative_time_s",
]


def write_csv(records: list[dict], path: Path) -> None:
    with open(path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=CSV_FIELDS)
        writer.writeheader()
        for r in records:
            writer.writerow({k: r[k] for k in CSV_FIELDS})


def sanity_check_2x2() -> None:
    """Quick 2x2 check to verify Deep CFR pipeline works."""
    print("Sanity check: 2x2 @ 5 CFR iters...")
    cfg = DeepCFRConfig(
        rows=2, cols=2,
        hidden_sizes=(32,),
        num_cfr_iters=5,
        num_traversals=20,
        advantage_train_steps=20,
        strategy_train_steps=50,
        buffer_size=5000,
    )
    solver = DeepCFR(cfg)
    solver.solve()
    expl = solver.exploitability()
    print(f"  expl={expl:.4f}")
    if expl > 1.5:
        print("ABORT: exploitability too high, something is broken.")
        sys.exit(1)
    print("  OK\n")


def run_experiment(
    board: str,
    num_cfr_iters: int,
    num_traversals: int | None,
    seed: int,
) -> None:
    board_cfg = dict(CONFIGS[board])  # copy to avoid mutating module-level default
    if num_traversals is not None:
        board_cfg["num_traversals"] = num_traversals

    cfg = DeepCFRConfig(
        num_cfr_iters=1,  # we iterate manually
        seed=seed,
        **board_cfg,
    )

    print(f"=== EXP-004: Deep CFR on {board} CDH ===")
    print(f"Config: rows={cfg.rows} cols={cfg.cols} seed={cfg.seed}")
    print(f"  hidden_sizes={cfg.hidden_sizes}")
    print(f"  traversals/iter={cfg.num_traversals}")
    print(f"  advantage_steps={cfg.advantage_train_steps}")
    print(f"  strategy_steps={cfg.strategy_train_steps}")
    print(f"  buffer_size={cfg.buffer_size}")
    print(f"  CFR iterations={num_cfr_iters}\n")

    solver = DeepCFR(cfg)

    out_dir = RESULTS_DIR / board
    out_dir.mkdir(parents=True, exist_ok=True)
    csv_path = out_dir / "convergence.csv"
    records: list[dict] = []
    cumulative_time = 0.0

    # Measure exploitability every iteration for small boards,
    # every 10 iterations for larger ones
    eval_interval = 1 if board == "2x2" else 5

    from darkhex._engine import best_response_values

    for cfr_iter in range(1, num_cfr_iters + 1):
        t0 = time.time()
        solver.solve(1)
        iter_time = time.time() - t0
        cumulative_time += iter_time

        if cfr_iter % eval_interval == 0 or cfr_iter == num_cfr_iters:
            t_expl = time.time()
            strategy = solver.extract_strategy()
            br_b, br_w, expl = best_response_values(
                cfg.rows, cfg.cols, strategy
            )
            expl_time = time.time() - t_expl

            record = {
                "cfr_iter": cfr_iter,
                "iteration": solver.iteration,
                "exploitability": round(expl, 6),
                "br_black": round(br_b, 6),
                "br_white": round(br_w, 6),
                "info_states_in_strategy": len(strategy),
                "adv_buf_0": len(solver._advantage_buffers[0]),
                "adv_buf_1": len(solver._advantage_buffers[1]),
                "strat_buf": len(solver._strategy_buffer),
                "iter_time_s": round(iter_time, 2),
                "expl_time_s": round(expl_time, 2),
                "cumulative_time_s": round(cumulative_time, 1),
            }
            records.append(record)
            write_csv(records, csv_path)

            print(
                f"  CFR {cfr_iter:>4d} (iter={solver.iteration:>4d}): "
                f"expl={expl:.4f}  "
                f"br_b={br_b:.4f} br_w={br_w:.4f}  "
                f"strat_states={len(strategy):>6d}  "
                f"time={iter_time:.1f}s  "
                f"cumul={cumulative_time:.0f}s"
            )

    # Save full results
    full_results = {
        "experiment": "exp004_deep_cfr",
        "board": board,
        "config": {
            "rows": cfg.rows,
            "cols": cfg.cols,
            "hidden_sizes": list(cfg.hidden_sizes),
            "lr": cfg.lr,
            "num_traversals": cfg.num_traversals,
            "advantage_train_steps": cfg.advantage_train_steps,
            "strategy_train_steps": cfg.strategy_train_steps,
            "buffer_size": cfg.buffer_size,
            "num_cfr_iters": num_cfr_iters,
            "seed": cfg.seed,
        },
        "total_time_s": round(cumulative_time, 1),
        "final_exploitability": records[-1]["exploitability"] if records else None,
        "records": records,
    }
    json_path = out_dir / "full_results.json"
    with open(json_path, "w") as f:
        json.dump(full_results, f, indent=2)

    # Generate convergence plot
    try:
        plot_convergence(records, out_dir / "convergence.png", board)
    except ImportError:
        print("matplotlib not installed, skipping plot generation")

    # Summary
    final = records[-1] if records else {}
    print("\n=== Summary ===")
    print(f"Board: {board}")
    print(f"CFR iterations: {num_cfr_iters}")
    print(f"Final exploitability: {final.get('exploitability', 'N/A')}")
    print(f"Total time: {cumulative_time:.0f}s")
    print(f"Results: {out_dir}/")


def plot_convergence(records: list[dict], output_path: Path, board: str) -> None:
    import matplotlib.pyplot as plt

    fig, ax = plt.subplots(figsize=(8, 5))

    iters = [r["cfr_iter"] for r in records]
    expls = [r["exploitability"] for r in records]
    ax.plot(iters, expls, "-o", color="#1f77b4", label="Deep CFR",
            markersize=4, linewidth=1.5)

    if board == "4x3":
        ax.axhline(0.989, color="orange", linestyle=":", alpha=0.7,
                    label="OS-MCCFR 1B (0.989)")
        ax.axhline(0.002, color="green", linestyle=":", alpha=0.7,
                    label="Thesis best (Ab-BR, 0.002)")

    ax.set_xlabel("Deep CFR Iteration", fontsize=12)
    ax.set_ylabel("Exploitability (Clairvoyant BR)", fontsize=12)
    ax.set_title(f"{board} CDH Dark Hex: Deep CFR Convergence", fontsize=14)
    ax.legend(fontsize=10)
    ax.grid(True, alpha=0.3)
    ax.tick_params(labelsize=10)
    fig.tight_layout()
    fig.savefig(output_path, dpi=300, bbox_inches="tight")
    fig.savefig(output_path.with_suffix(".pdf"), bbox_inches="tight")
    plt.close(fig)
    print(f"Plot saved to {output_path}")


def main() -> None:
    parser = argparse.ArgumentParser(description="EXP-004: Deep CFR on Dark Hex")
    parser.add_argument(
        "--board", type=str, default="2x2", choices=list(CONFIGS.keys()),
        help="Board size (default: 2x2)",
    )
    parser.add_argument(
        "--iters", type=int, default=None,
        help="Number of CFR iterations (default: 50 for 2x2, 200 for 4x3)",
    )
    parser.add_argument(
        "--traversals", type=int, default=None,
        help="Traversals per player per CFR iteration",
    )
    parser.add_argument("--seed", type=int, default=42)
    args = parser.parse_args()

    default_iters = {"2x2": 100, "3x2": 50, "3x3": 30, "4x3": 200}
    num_iters = args.iters or default_iters[args.board]

    # Sanity check on fresh runs
    if args.board != "2x2":
        sanity_check_2x2()

    run_experiment(args.board, num_iters, args.traversals, args.seed)


if __name__ == "__main__":
    main()
