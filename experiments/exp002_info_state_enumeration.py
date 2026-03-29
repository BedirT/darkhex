"""EXP-002: Exhaustive Info State Enumeration for CDH Dark Hex.

Hypothesis: The exact number of imperfect-recall info states can be
computed by exhaustive game tree traversal for boards up to 3x3.

Evaluation: Run enumerate_game_tree on all feasible board sizes.
Record info state counts, terminal counts, depth, and timing.
Compare against thesis values where available.
"""

import csv
import json
import time
from pathlib import Path

from darkhex._engine import enumerate_game_tree

RESULTS_DIR = Path("results/exp002_info_state_enumeration")

# Thesis reference values (Tapkan 2022, Table X)
THESIS_REFERENCE = {
    "2x2": {"ir_info_states": 42, "pr_info_states": 441},
    "3x2": {"ir_info_states": 410, "pr_info_states": 196_357},
    "3x3": {"ir_info_states": 12_556, "pr_info_states": 19_119_486_978},
}


def main():
    print("=== EXP-002: Exhaustive Info State Enumeration ===\n")

    # Only enumerate boards that complete in reasonable time
    # 3x3 takes ~6h, so we include it but note the time
    boards = [(2, 2), (2, 3), (3, 2), (3, 3)]
    records = []

    for rows, cols in boards:
        board_key = f"{rows}x{cols}"
        print(f"{board_key}: ", end="", flush=True)

        t0 = time.time()
        stats = enumerate_game_tree(rows, cols)
        wall_time = time.time() - t0

        thesis = THESIS_REFERENCE.get(board_key, {})
        thesis_ir = thesis.get("ir_info_states", "N/A")
        match = (
            "MATCH"
            if thesis_ir != "N/A" and stats.total_info_states == thesis_ir
            else "NEW" if thesis_ir == "N/A" else "MISMATCH"
        )

        record = {
            "board": board_key,
            "rows": rows,
            "cols": cols,
            "total_info_states": stats.total_info_states,
            "p0_info_states": stats.info_states_by_player[0],
            "p1_info_states": stats.info_states_by_player[1],
            "terminal_states": stats.terminal_states,
            "max_depth": stats.max_depth,
            "wall_time_s": round(wall_time, 3),
            "thesis_ir": thesis_ir,
            "thesis_match": match,
        }
        records.append(record)

        print(
            f"{stats.total_info_states:>6} info states  "
            f"(P0={stats.info_states_by_player[0]}, "
            f"P1={stats.info_states_by_player[1]})  "
            f"terminals={stats.terminal_states:>12,}  "
            f"depth={stats.max_depth:>2}  "
            f"{wall_time:.2f}s  "
            f"[{match} vs thesis: {thesis_ir}]"
        )

        # Skip 3x3 if it took too long (>1h) and we haven't done larger
        if wall_time > 3600 and (rows, cols) != boards[-1]:
            print("  (skipping larger boards)")
            break

    # Save CSV
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    csv_path = RESULTS_DIR / "enumeration.csv"
    with open(csv_path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=list(records[0].keys()))
        writer.writeheader()
        writer.writerows(records)

    # Save JSON
    json_path = RESULTS_DIR / "full_results.json"
    with open(json_path, "w") as f:
        json.dump(records, f, indent=2)

    print(f"\nResults saved to {csv_path} and {json_path}")


if __name__ == "__main__":
    main()
