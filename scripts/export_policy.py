#!/usr/bin/env python3
"""Convert MCCFR solver checkpoint (.bin) to DSaGe policy JSON.

Usage:
    python scripts/export_policy.py checkpoint.bin --player 0 -o policy.json
    python scripts/export_policy.py checkpoint.bin --player 0 --sip 0.05 2
    python scripts/export_policy.py checkpoint.bin --player 0 --sip-plus 0.05 2 8 0.01
"""

import argparse
import json
import sys

from darkhex._engine import MCCFRSolver, simplify_policy, simplify_policy_plus


def rotate_info_state(info_state: str, rows: int, cols: int) -> str:
    """Compute the 180° rotated info state string."""
    lines = info_state.split("\n")
    player_line = lines[0]
    board = "".join(lines[1:])
    rotated = board[::-1]
    board_lines = [rotated[i : i + cols] for i in range(0, len(rotated), cols)]
    return player_line + "\n" + "\n".join(board_lines)


def rotate_action(action: int, total_cells: int) -> int:
    """Rotate a cell index by 180°."""
    return total_cells - 1 - action


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Convert MCCFR checkpoint to DSaGe policy JSON",
    )
    parser.add_argument("checkpoint", help="Path to .bin checkpoint file")
    parser.add_argument(
        "--player", "-p", type=int, required=True, choices=[0, 1],
        help="Player to extract (0=Black, 1=White)",
    )
    parser.add_argument(
        "--output", "-o", default=None,
        help="Output JSON path (default: stdout)",
    )
    parser.add_argument(
        "--sip", nargs=2, type=float, metavar=("EPSILON", "ACTION_CAP"),
        help="Apply SIP simplification (epsilon, action_cap)",
    )
    parser.add_argument(
        "--sip-plus", nargs=4, type=float,
        metavar=("EPSILON", "ACTION_CAP", "FRAC_LIMIT", "ETA"),
        help="Apply SIP+ simplification (epsilon, action_cap, frac_limit, eta)",
    )

    args = parser.parse_args()

    # Load checkpoint
    solver = MCCFRSolver.load(args.checkpoint)
    print(
        f"Loaded: {solver.rows()}x{solver.cols()}, "
        f"{solver.iterations()} iters, "
        f"{solver.num_info_states()} info states",
        file=sys.stderr,
    )

    # Extract average strategy
    strategy = solver.get_average_strategy()

    # Optional simplification
    if args.sip:
        epsilon, action_cap = args.sip
        strategy = simplify_policy(strategy, epsilon, int(action_cap))
        print(f"SIP: eps={epsilon}, cap={int(action_cap)}", file=sys.stderr)
    elif args.sip_plus:
        epsilon, action_cap, frac_limit, eta = args.sip_plus
        strategy = simplify_policy_plus(
            strategy, epsilon, int(action_cap), int(frac_limit), eta,
        )
        print(
            f"SIP+: eps={epsilon}, cap={int(action_cap)}, "
            f"frac={int(frac_limit)}, eta={eta}",
            file=sys.stderr,
        )

    # Filter by player
    prefix = f"P{args.player}\n"
    player_strategy = {
        k: v for k, v in strategy.items() if k.startswith(prefix)
    }
    print(
        f"Player {args.player}: {len(player_strategy)} info states "
        f"(of {len(strategy)} total)",
        file=sys.stderr,
    )

    # Convert to ExportedPolicy format: {info_state: {action_str: prob}}
    # The solver stores canonical keys (isomorphic reduction via 180° rotation).
    # The tree builder generates raw keys. Emit both canonical and rotated forms
    # so lookups succeed regardless of which form the tree builder produces.
    rows, cols = solver.rows(), solver.cols()
    total_cells = rows * cols
    policy_dict: dict[str, dict[str, float]] = {}
    for info_state, actions in player_strategy.items():
        action_map: dict[str, float] = {}
        for action, prob in actions:
            action_map[str(action)] = round(float(prob), 6)
        policy_dict[info_state] = action_map

        # Also emit the rotated (non-canonical) form
        rotated_key = rotate_info_state(info_state, rows, cols)
        if rotated_key != info_state and rotated_key not in policy_dict:
            rotated_actions: dict[str, float] = {}
            for action, prob in actions:
                rotated_actions[str(rotate_action(action, total_cells))] = round(
                    float(prob), 6,
                )
            policy_dict[rotated_key] = rotated_actions

    output = {
        "player": args.player,
        "rows": solver.rows(),
        "cols": solver.cols(),
        "perfectRecall": False,
        "policy": policy_dict,
    }

    json_str = json.dumps(output, ensure_ascii=False, indent=2)

    if args.output:
        with open(args.output, "w") as f:
            f.write(json_str)
            f.write("\n")
        print(f"Written to {args.output}", file=sys.stderr)
    else:
        print(json_str)


if __name__ == "__main__":
    main()
