"""Deep Counterfactual Regret Minimization for Dark Hex.

Brown, N., Lerer, A., Gross, S. & Sandholm, T. (2019).
Deep Counterfactual Regret Minimization. ICML.

Reference implementation: open_spiel/python/pytorch/deep_cfr.py
"""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

from darkhex._engine import DarkHexState, Player, best_response_values

# ---------------------------------------------------------------------------
# Info state encoding
# ---------------------------------------------------------------------------

def _player_index(p: Player) -> int:
    return 0 if p == Player.Black else 1


def encode_info_state(info_state_str: str, rows: int, cols: int) -> torch.Tensor:
    """Convert canonical info state string to a flat tensor.

    Format: "P{p}\\n{grid}" where grid uses x/o/. for own/opponent/empty.

    Encoding: per-cell one-hot [is_own, is_opponent, is_empty] + player indicator.
    Output shape: (3 * rows * cols + 1,).
    """
    lines = info_state_str.split("\n")
    player = int(lines[0][1])
    grid = "".join(lines[1:])

    # Info state uses absolute colors: x = Black stone, o = White stone.
    # We encode player-relative: own/opponent/empty.
    # For P0 (Black): x = own, o = opponent
    # For P1 (White): x = opponent, o = own
    own_char = "x" if player == 0 else "o"
    opp_char = "o" if player == 0 else "x"

    n = rows * cols
    features = np.zeros(3 * n + 1, dtype=np.float32)
    for i, ch in enumerate(grid):
        if ch == own_char:
            features[3 * i] = 1.0  # own
        elif ch == opp_char:
            features[3 * i + 1] = 1.0  # opponent
        else:  # '.'
            features[3 * i + 2] = 1.0  # empty
    features[3 * n] = float(player)

    return torch.from_numpy(features)


# ---------------------------------------------------------------------------
# Reservoir buffer
# ---------------------------------------------------------------------------


class ReservoirBuffer:
    """Fixed-capacity reservoir sampling buffer (Vitter 1985).

    Maintains a uniform random sample over all entries ever appended.
    Pre-allocates numpy arrays on first append (lazy init).
    """

    def __init__(self, capacity: int, seed: int | None = None) -> None:
        self.capacity = capacity
        self._count = 0  # total appends seen
        self._size = 0  # entries currently stored
        self._info_states: np.ndarray | None = None
        self._iterations: np.ndarray | None = None
        self._values: np.ndarray | None = None
        self._rng = np.random.default_rng(seed)

    def _init_arrays(self, info_dim: int, val_dim: int) -> None:
        self._info_states = np.zeros((self.capacity, info_dim), dtype=np.float32)
        self._iterations = np.zeros(self.capacity, dtype=np.int64)
        self._values = np.zeros((self.capacity, val_dim), dtype=np.float32)

    def append(
        self,
        info_state: np.ndarray,
        iteration: int,
        values: np.ndarray,
    ) -> None:
        if self._info_states is None:
            self._init_arrays(len(info_state), len(values))

        if self._count < self.capacity:
            idx = self._count
        else:
            idx = self._rng.integers(0, self._count + 1)
            if idx >= self.capacity:
                self._count += 1
                return

        self._info_states[idx] = info_state
        self._iterations[idx] = iteration
        self._values[idx] = values
        self._size = min(self._count + 1, self.capacity)
        self._count += 1

    def sample(self, n: int) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
        """Return a random batch of (info_states, iterations, values) as tensors."""
        indices = self._rng.choice(self._size, size=min(n, self._size), replace=False)
        return (
            torch.from_numpy(self._info_states[indices].copy()),
            torch.from_numpy(self._iterations[indices].copy()),
            torch.from_numpy(self._values[indices].copy()),
        )

    def __len__(self) -> int:
        return self._size


# ---------------------------------------------------------------------------
# Neural networks
# ---------------------------------------------------------------------------


class MLP(nn.Module):
    """MLP with LayerNorm on last hidden layer (per paper and OpenSpiel).

    Architecture: input -> [Linear -> ReLU]* -> LayerNorm -> Linear -> [activation]
    """

    def __init__(
        self,
        input_dim: int,
        hidden_sizes: tuple[int, ...],
        output_dim: int,
        final_activation: nn.Module | None = None,
    ) -> None:
        super().__init__()
        layers: list[nn.Module] = []
        prev = input_dim
        for i, h in enumerate(hidden_sizes):
            layers.append(nn.Linear(prev, h))
            layers.append(nn.ReLU())
            if i == len(hidden_sizes) - 1:
                layers.append(nn.LayerNorm(h))
            prev = h
        layers.append(nn.Linear(prev, output_dim))
        if final_activation is not None:
            layers.append(final_activation)
        self.net = nn.Sequential(*layers)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return self.net(x)

    def reset(self) -> None:
        """Reinitialize all parameters from scratch."""
        for module in self.modules():
            if hasattr(module, "reset_parameters"):
                module.reset_parameters()


# ---------------------------------------------------------------------------
# Deep CFR config and solver
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class DeepCFRConfig:
    """Configuration for Deep CFR."""

    rows: int
    cols: int
    hidden_sizes: tuple[int, ...] = (64, 64)
    lr: float = 1e-3
    buffer_size: int = 1_000_000
    batch_size_advantage: int = 256
    batch_size_strategy: int = 256
    advantage_train_steps: int = 375
    strategy_train_steps: int = 2500
    num_cfr_iters: int = 100
    num_traversals: int = 375
    seed: int = 42
    reinitialize_advantage_networks: bool = True


class DeepCFR:
    """Deep CFR solver for Dark Hex.

    Implements Algorithm 1 & 2 from Brown et al. (2019) with:
    - External Sampling traversal
    - LCFR weighting (linear iteration weighting via sqrt(t) trick)
    - Advantage network reinitialization from scratch each CFR iteration
    - Argmax tiebreaker when all regrets <= 0
    - Isomorphic reduction via canonical info states
    """

    def __init__(self, cfg: DeepCFRConfig) -> None:
        self.cfg = cfg
        self._n = cfg.rows * cfg.cols
        self._input_dim = 3 * self._n + 1
        self._iteration = 0

        torch.manual_seed(cfg.seed)
        self._rng = np.random.default_rng(cfg.seed)

        # One advantage net per player, one shared strategy net
        self._advantage_nets = [
            MLP(self._input_dim, cfg.hidden_sizes, self._n)
            for _ in range(2)
        ]
        self._strategy_net = MLP(
            self._input_dim,
            cfg.hidden_sizes,
            self._n,
            final_activation=nn.Softmax(dim=-1),
        )

        # Reservoir buffers: one advantage buffer per player, one strategy buffer
        # Use derived seeds for deterministic replay buffer sampling.
        self._advantage_buffers = [
            ReservoirBuffer(cfg.buffer_size, seed=cfg.seed + 1),
            ReservoirBuffer(cfg.buffer_size, seed=cfg.seed + 2),
        ]
        self._strategy_buffer = ReservoirBuffer(
            cfg.buffer_size, seed=cfg.seed + 3
        )

        # Strategy net optimizer persists across iterations
        self._strategy_optimizer = torch.optim.Adam(
            self._strategy_net.parameters(), lr=cfg.lr
        )

    @property
    def iteration(self) -> int:
        return self._iteration

    # -- Main solve loop (Algorithm 1) -------------------------------------

    def solve(self, num_cfr_iters: int | None = None) -> None:
        """Run Deep CFR for the specified number of CFR iterations."""
        T = num_cfr_iters or self.cfg.num_cfr_iters
        for _ in range(T):
            # 1-based iteration counter, same for both players in this
            # outer CFR iteration. This ensures LCFR weighting (sqrt(t))
            # treats both players' data from the same iteration equally.
            self._iteration += 1

            for player in range(2):
                # K traversals per player
                for _ in range(self.cfg.num_traversals):
                    state = DarkHexState(self.cfg.rows, self.cfg.cols)
                    self._traverse(state, player)

                # Train advantage net (reinitialize first)
                self._train_advantage_net(player)

            # Train strategy net after both players
            self._train_strategy_net()

    # -- External Sampling traversal (Algorithm 2) -------------------------

    def _traverse(self, state: DarkHexState, traverser: int) -> float:
        if state.is_terminal():
            return state.returns()[traverser]

        player_enum = (
            Player.Black
            if state.current_player() == Player.Black
            else Player.White
        )
        player = _player_index(player_enum)
        actions = state.legal_actions()

        # Canonicalize info state
        canonical_str, is_canonical = state.canonical_info_state(player_enum)
        canonical_actions = self._to_canonical_actions(actions, is_canonical)

        # Get strategy via regret matching
        strategy = self._regret_match(player, canonical_str, canonical_actions)

        if player == traverser:
            # External Sampling: explore ALL actions
            values: dict[int, float] = {}
            for action in actions:
                child = state.copy()
                child.apply_action(action)
                values[action] = self._traverse(child, traverser)

            # Counterfactual value
            cfv = sum(
                strategy[ca] * values[a]
                for a, ca in zip(actions, canonical_actions)
            )

            # Compute advantages and store in buffer (full-width array)
            advantages = np.zeros(self._n, dtype=np.float32)
            for a, ca in zip(actions, canonical_actions):
                advantages[ca] = values[a] - cfv

            info_tensor = encode_info_state(
                canonical_str, self.cfg.rows, self.cfg.cols
            ).numpy()
            self._advantage_buffers[player].append(
                info_tensor, self._iteration, advantages
            )
            return cfv
        else:
            # Opponent node: store strategy, sample one action
            strategy_vec = np.zeros(self._n, dtype=np.float32)
            for ca, prob in strategy.items():
                strategy_vec[ca] = prob

            info_tensor = encode_info_state(
                canonical_str, self.cfg.rows, self.cfg.cols
            ).numpy()
            self._strategy_buffer.append(
                info_tensor, self._iteration, strategy_vec
            )

            # Sample action from strategy
            probs = np.array(
                [strategy[ca] for ca in canonical_actions], dtype=np.float64
            )
            probs /= probs.sum()  # renormalize for numerical safety
            idx = self._rng.choice(len(canonical_actions), p=probs)
            sampled_action = actions[idx]

            child = state.copy()
            child.apply_action(sampled_action)
            return self._traverse(child, traverser)

    # -- Regret matching with argmax tiebreaker ----------------------------

    def _regret_match(
        self,
        player: int,
        canonical_str: str,
        canonical_actions: list[int],
    ) -> dict[int, float]:
        """Compute strategy from advantage net via regret matching."""
        with torch.no_grad():
            x = encode_info_state(
                canonical_str, self.cfg.rows, self.cfg.cols
            ).unsqueeze(0)
            advantages = self._advantage_nets[player](x).squeeze(0)

        positive = {
            ca: max(0.0, advantages[ca].item()) for ca in canonical_actions
        }
        total = sum(positive.values())

        if total > 0:
            return {ca: positive[ca] / total for ca in canonical_actions}
        else:
            # Argmax tiebreaker (per paper ablation — NOT uniform)
            best = max(
                canonical_actions, key=lambda ca: advantages[ca].item()
            )
            return {
                ca: (1.0 if ca == best else 0.0) for ca in canonical_actions
            }

    # -- Network training --------------------------------------------------

    def _train_advantage_net(self, player: int) -> float:
        """Train advantage net on buffer with LCFR weighting. Returns avg loss."""
        buf = self._advantage_buffers[player]
        if len(buf) < self.cfg.batch_size_advantage:
            return 0.0

        net = self._advantage_nets[player]
        if self.cfg.reinitialize_advantage_networks:
            net.reset()

        optimizer = torch.optim.Adam(net.parameters(), lr=self.cfg.lr)
        net.train()
        total_loss = 0.0

        for _ in range(self.cfg.advantage_train_steps):
            states, iterations, targets = buf.sample(
                self.cfg.batch_size_advantage
            )
            # LCFR weighting: sqrt(t) trick -> MSE gives t*(pred-target)^2
            weights = torch.sqrt(iterations.float()).unsqueeze(1)
            preds = net(states)
            loss = F.mse_loss(weights * preds, weights * targets)

            optimizer.zero_grad()
            loss.backward()
            optimizer.step()
            total_loss += loss.item()

        net.eval()
        return total_loss / self.cfg.advantage_train_steps

    def _train_strategy_net(self) -> float:
        """Train strategy net on buffer with LCFR weighting. Returns avg loss."""
        buf = self._strategy_buffer
        if len(buf) < self.cfg.batch_size_strategy:
            return 0.0

        self._strategy_net.train()
        total_loss = 0.0

        for _ in range(self.cfg.strategy_train_steps):
            states, iterations, targets = buf.sample(
                self.cfg.batch_size_strategy
            )
            weights = torch.sqrt(iterations.float()).unsqueeze(1)
            preds = self._strategy_net(states)
            loss = F.mse_loss(weights * preds, weights * targets)

            self._strategy_optimizer.zero_grad()
            loss.backward()
            self._strategy_optimizer.step()
            total_loss += loss.item()

        self._strategy_net.eval()
        return total_loss / self.cfg.strategy_train_steps

    # -- Canonical action helpers ------------------------------------------

    def _to_canonical_actions(
        self, actions: list[int], is_canonical: bool
    ) -> list[int]:
        if is_canonical:
            return list(actions)
        return [self._n - 1 - a for a in actions]

    # -- Strategy extraction -----------------------------------------------

    def extract_strategy(
        self,
    ) -> dict[str, list[tuple[int, float]]]:
        """Extract strategy from strategy net for exploitability computation.

        Walks the game tree, queries strategy net at each canonical info state.
        Returns strategy in the same format as MCCFRSolver.get_average_strategy():
        Dict[canonical_info_state -> List[(canonical_action, probability)]].
        """
        result: dict[str, list[tuple[int, float]]] = {}
        state = DarkHexState(self.cfg.rows, self.cfg.cols)
        self._extract_recursive(state, result, set())
        return result

    def _extract_recursive(
        self,
        state: DarkHexState,
        result: dict[str, list[tuple[int, float]]],
        visited_game_states: set[str],
    ) -> None:
        if state.is_terminal():
            return

        # Memoize by observable game state (both player info states +
        # current player) to avoid exponential re-traversal.
        # 3x3 has 31.9M game tree nodes but far fewer distinct
        # observable states — this is the critical optimization.
        is_b = state.info_state_string(Player.Black)
        is_w = state.info_state_string(Player.White)
        game_key = is_b + "|" + is_w
        if game_key in visited_game_states:
            return
        visited_game_states.add(game_key)

        player_enum = (
            Player.Black
            if state.current_player() == Player.Black
            else Player.White
        )
        canonical_str, is_canonical = state.canonical_info_state(player_enum)

        if canonical_str not in result:
            actions = state.legal_actions()
            canonical_actions = self._to_canonical_actions(actions, is_canonical)

            with torch.no_grad():
                x = encode_info_state(
                    canonical_str, self.cfg.rows, self.cfg.cols
                ).unsqueeze(0)
                probs = self._strategy_net(x).squeeze(0)

            # Renormalize over legal actions only — the softmax output
            # spreads mass across all cells including illegal ones.
            legal_probs = {ca: probs[ca].item() for ca in canonical_actions}
            total = sum(legal_probs.values())
            if total > 1e-8:
                result[canonical_str] = [
                    (ca, p / total)
                    for ca, p in legal_probs.items()
                    if p / total > 1e-6
                ]
            else:
                # Fallback to uniform over legal actions
                n_legal = len(canonical_actions)
                result[canonical_str] = [
                    (ca, 1.0 / n_legal) for ca in canonical_actions
                ]

        for action in state.legal_actions():
            child = state.copy()
            child.apply_action(action)
            self._extract_recursive(child, result, visited_game_states)

    def exploitability(self) -> float:
        """Compute exploitability of the current strategy net."""
        strategy = self.extract_strategy()
        _, _, expl = best_response_values(
            self.cfg.rows, self.cfg.cols, strategy
        )
        return expl
