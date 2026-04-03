"""ESCHER: Eschewing Importance Sampling in Games by Computing a
History value function to Estimate Regret.

McAleer, S., Farina, G., Lanctot, M. & Brown, N. (2023).
ESCHER. ICLR. arXiv:2206.04122.

Eliminates importance sampling entirely by using a learned history-value
network to compute regrets deterministically. Variance is orders of
magnitude lower than DREAM on Dark Hex.

Reference implementation: open_spiel/python/pytorch/escher.py
"""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

from darkhex._engine import DarkHexState, Player, best_response_values
from darkhex.algorithms.deep_cfr import (
    MLP,
    ReservoirBuffer,
    encode_info_state,
    _player_index,
)


# ---------------------------------------------------------------------------
# ESCHER config
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class ESCHERConfig:
    """Configuration for ESCHER solver."""

    rows: int
    cols: int

    # Network architecture
    value_hidden: tuple[int, ...] = (128, 128)
    regret_hidden: tuple[int, ...] = (128, 128)
    policy_hidden: tuple[int, ...] = (128, 128)
    lr: float = 1e-3

    # Buffer sizes
    value_buffer_size: int = 1_000_000
    regret_buffer_size: int = 1_000_000
    policy_buffer_size: int = 1_000_000

    # Training
    value_batch_size: int = 256
    value_train_steps: int = 512
    regret_batch_size: int = 256
    regret_train_steps: int = 375
    policy_batch_size: int = 256
    policy_train_steps: int = 2500

    # Traversals
    value_traversals: int = 512
    regret_traversals: int = 1024
    value_exploration: float = 0.1
    num_iters: int = 100

    seed: int = 42


# ---------------------------------------------------------------------------
# ESCHER solver
# ---------------------------------------------------------------------------


class ESCHER:
    """ESCHER solver for Dark Hex.

    Three-network architecture:
    - Value networks (one per player): predict Q(h,a) from full history
      (both players' canonical info states). Used to compute regrets
      without importance sampling.
    - Regret networks (one per player): predict per-action regrets from
      single-player info state. Regret-matched to produce current policy.
    - Average policy network (shared): approximates the average strategy
      for evaluation.

    Key advantage over DREAM: no importance sampling means no IS weight
    explosions. Regret variance is bounded by value network accuracy,
    not by game depth or reach probabilities.
    """

    def __init__(self, cfg: ESCHERConfig) -> None:
        self.cfg = cfg
        self._n = cfg.rows * cfg.cols
        self._obs_dim = 3 * self._n + 1  # single-player info state
        self._history_dim = 2 * self._obs_dim  # both players' info states
        self._iteration = 0

        torch.manual_seed(cfg.seed)
        self._rng = np.random.default_rng(cfg.seed)

        # Value networks: predict V(h) from history (per player)
        # Single scalar output — state value, not per-action
        self._value_nets = [
            MLP(self._history_dim, cfg.value_hidden, 1)
            for _ in range(2)
        ]

        # Regret networks: predict regret from info state (per player)
        self._regret_nets = [
            MLP(self._obs_dim, cfg.regret_hidden, self._n)
            for _ in range(2)
        ]

        # Average policy network: shared
        self._policy_net = MLP(
            self._obs_dim, cfg.policy_hidden, self._n,
        )

        # Buffers
        self._value_buffers = [
            ReservoirBuffer(cfg.value_buffer_size, seed=cfg.seed + i)
            for i in range(2)
        ]
        self._regret_buffers = [
            ReservoirBuffer(cfg.regret_buffer_size, seed=cfg.seed + 10 + i)
            for i in range(2)
        ]
        self._policy_buffer = ReservoirBuffer(
            cfg.policy_buffer_size, seed=cfg.seed + 20
        )

    @property
    def iteration(self) -> int:
        return self._iteration

    # -- Main solve loop ---------------------------------------------------

    def solve(self, num_iters: int | None = None) -> None:
        """Run ESCHER for the specified number of iterations."""
        T = num_iters or self.cfg.num_iters

        for _ in range(T):
            self._iteration += 1

            for player in range(2):
                # Phase 1: Train value network for this player
                self._gather_value_data(player)
                self._train_value_net(player)

                # Phase 2: Gather regret data using value net
                self._gather_regret_data(player)
                self._train_regret_net(player)

            # Phase 3: Train average policy
            self._train_policy_net()

    # -- Value data collection ---------------------------------------------

    def _gather_value_data(self, player: int) -> None:
        """Collect value training data via epsilon-greedy trajectories.

        Rolls out trajectories, then computes importance-corrected
        discounted returns backwards. The importance ratio is bounded
        by 1/epsilon (unlike DREAM where it's unbounded).
        """
        self._value_buffers[player].clear() if hasattr(
            self._value_buffers[player], 'clear'
        ) else None

        for _ in range(self.cfg.value_traversals):
            state = DarkHexState(self.cfg.rows, self.cfg.cols)
            transitions: list[tuple[np.ndarray, int, float, np.ndarray]] = []

            # Forward pass: collect trajectory
            while not state.is_terminal():
                player_enum = state.current_player()
                p = _player_index(player_enum)
                actions = state.legal_actions()
                num_actions = len(actions)

                # Get current policy from regret net
                canon, is_canon = state.canonical_info_state(player_enum)
                ca = self._to_canonical_actions(actions, is_canon)
                policy = self._regret_match(p, canon, ca)
                policy_arr = np.zeros(num_actions, dtype=np.float64)
                for i, c in enumerate(ca):
                    policy_arr[i] = policy[c]

                # Epsilon-greedy sampling
                eps = self.cfg.value_exploration
                uniform = np.ones(num_actions, dtype=np.float64) / num_actions
                sample_policy = eps * uniform + (1.0 - eps) * policy_arr
                sample_policy /= sample_policy.sum()

                action_idx = self._rng.choice(num_actions, p=sample_policy)
                importance = policy_arr[action_idx] / sample_policy[action_idx]

                # Store transition
                history = self._encode_history(state)
                transitions.append((
                    history,
                    actions[action_idx],
                    importance,
                    np.array(state.returns(), dtype=np.float64),
                ))

                state.apply_action(actions[action_idx])

            # Terminal transition
            transitions.append((
                self._encode_history(state),
                -1,
                1.0,
                np.array(state.returns(), dtype=np.float64),
            ))

            # Backward pass: compute importance-corrected values
            value = np.zeros(2, dtype=np.float64)
            for i in range(len(transitions) - 1, -1, -1):
                history, action, importance, returns = transitions[i]
                value = importance * (returns + value)

                # Store value for the target player
                self._value_buffers[player].append(
                    history,
                    self._iteration,
                    np.array([value[player]], dtype=np.float32),
                )

    # -- Regret data collection --------------------------------------------

    def _gather_regret_data(self, player: int) -> None:
        """Gather regret training data using value network predictions.

        At each traverser node, query value net for ALL children to get
        Q(h,a) for each action. Compute regret = Q(h,a) - V(h).
        No importance sampling needed.
        """
        for _ in range(self.cfg.regret_traversals):
            state = DarkHexState(self.cfg.rows, self.cfg.cols)

            while not state.is_terminal():
                player_enum = state.current_player()
                p = _player_index(player_enum)
                actions = state.legal_actions()
                num_actions = len(actions)

                # Get current policy from regret net
                canon, is_canon = state.canonical_info_state(player_enum)
                ca = self._to_canonical_actions(actions, is_canon)
                policy = self._regret_match(p, canon, ca)
                policy_arr = np.zeros(num_actions, dtype=np.float64)
                for i, c in enumerate(ca):
                    policy_arr[i] = policy[c]

                if p == player:
                    # Compute regret using value network (THE KEY INSIGHT)
                    regret = self._compute_regret(state, policy_arr, player)

                    # Store regret + legal action mask
                    info_tensor = encode_info_state(
                        canon, self.cfg.rows, self.cfg.cols
                    ).numpy()
                    mask = np.zeros(self._n, dtype=np.float32)
                    regret_vec = np.zeros(self._n, dtype=np.float32)
                    for i, c in enumerate(ca):
                        mask[c] = 1.0
                        regret_vec[c] = regret[i]

                    # Store [regret_0..n-1, mask_0..n-1]
                    combined = np.concatenate([regret_vec, mask])
                    self._regret_buffers[player].append(
                        info_tensor, self._iteration, combined
                    )

                    # Store policy for average strategy
                    policy_vec = np.zeros(self._n, dtype=np.float32)
                    for i, c in enumerate(ca):
                        policy_vec[c] = policy_arr[i]
                    self._policy_buffer.append(
                        info_tensor, self._iteration, policy_vec
                    )

                    # Traverser uses UNIFORM sampling (no IS needed)
                    sample_policy = np.ones(num_actions) / num_actions
                else:
                    # Opponent uses current policy
                    sample_policy = policy_arr

                action_idx = self._rng.choice(num_actions, p=sample_policy)
                state.apply_action(actions[action_idx])

    def _compute_regret(
        self,
        state: DarkHexState,
        policy: np.ndarray,
        player: int,
    ) -> np.ndarray:
        """Compute regret for each action using value network.

        Q(h,a) = value_net(history(child(h,a)))
        V(h) = sum_a policy(a) * Q(h,a)
        regret(a) = Q(h,a) - V(h)
        """
        actions = state.legal_actions()
        num_actions = len(actions)
        q_values = np.zeros(num_actions, dtype=np.float64)

        with torch.no_grad():
            for i, a in enumerate(actions):
                child = state.copy()
                child.apply_action(a)
                history = self._encode_history(child)
                x = torch.from_numpy(history).float().unsqueeze(0)
                q_values[i] = self._value_nets[player](x).item()

        value = np.sum(policy * q_values)
        return q_values - value

    # -- Network training --------------------------------------------------

    def _train_value_net(self, player: int) -> float:
        """Train value network on collected trajectory data."""
        buf = self._value_buffers[player]
        if len(buf) < self.cfg.value_batch_size:
            return 0.0

        net = self._value_nets[player]
        net.reset()
        optimizer = torch.optim.Adam(net.parameters(), lr=self.cfg.lr)
        net.train()
        total_loss = 0.0

        for _ in range(self.cfg.value_train_steps):
            states, _, targets = buf.sample(self.cfg.value_batch_size)
            preds = net(states)
            loss = F.mse_loss(preds, targets)

            optimizer.zero_grad()
            loss.backward()
            optimizer.step()
            total_loss += loss.item()

        net.eval()
        return total_loss / self.cfg.value_train_steps

    def _train_regret_net(self, player: int) -> float:
        """Train regret network with LCFR weighting and legal action mask."""
        buf = self._regret_buffers[player]
        if len(buf) < self.cfg.regret_batch_size:
            return 0.0

        net = self._regret_nets[player]
        net.reset()
        optimizer = torch.optim.Adam(net.parameters(), lr=self.cfg.lr)
        net.train()
        total_loss = 0.0

        for _ in range(self.cfg.regret_train_steps):
            states, iterations, combined = buf.sample(
                self.cfg.regret_batch_size
            )
            # Unpack: [regret_0..n-1, mask_0..n-1]
            targets = combined[:, :self._n]
            masks = combined[:, self._n:]

            # LCFR weighting: t / T (linear, per OpenSpiel ESCHER)
            weights = (iterations.float() / self._iteration).unsqueeze(1)
            weights = weights.expand_as(targets)

            preds = net(states)
            # Masked MSE loss (only over legal actions)
            diff = (preds - targets) ** 2
            loss = (diff * weights * masks).sum() / masks.sum().clamp(min=1)

            optimizer.zero_grad()
            loss.backward()
            optimizer.step()
            total_loss += loss.item()

        net.eval()
        return total_loss / self.cfg.regret_train_steps

    def _train_policy_net(self) -> float:
        """Train average policy network with LCFR weighting."""
        buf = self._policy_buffer
        if len(buf) < self.cfg.policy_batch_size:
            return 0.0

        optimizer = torch.optim.Adam(
            self._policy_net.parameters(), lr=self.cfg.lr
        )
        self._policy_net.train()
        total_loss = 0.0

        for _ in range(self.cfg.policy_train_steps):
            states, iterations, targets = buf.sample(
                self.cfg.policy_batch_size
            )
            # LCFR: t / T
            weights = (iterations.float() / self._iteration).unsqueeze(1)

            preds = self._policy_net(states)

            # Cross-entropy loss (per OpenSpiel ESCHER)
            # targets are policy vectors (sum ~1 over legal actions)
            log_preds = F.log_softmax(preds, dim=-1)
            loss = -(targets * log_preds * weights).sum() / targets.sum().clamp(min=1)

            optimizer.zero_grad()
            loss.backward()
            optimizer.step()
            total_loss += loss.item()

        self._policy_net.eval()
        return total_loss / self.cfg.policy_train_steps

    # -- Regret matching ---------------------------------------------------

    def _regret_match(
        self,
        player: int,
        canonical_str: str,
        canonical_actions: list[int],
    ) -> dict[int, float]:
        """Compute strategy from regret net via regret matching."""
        with torch.no_grad():
            x = encode_info_state(
                canonical_str, self.cfg.rows, self.cfg.cols
            ).unsqueeze(0)
            regrets = self._regret_nets[player](x).squeeze(0)

        positive = {
            ca: max(0.0, regrets[ca].item()) for ca in canonical_actions
        }
        total = sum(positive.values())

        if total > 1e-6:
            return {ca: positive[ca] / total for ca in canonical_actions}
        else:
            best = max(
                canonical_actions, key=lambda ca: regrets[ca].item()
            )
            return {
                ca: (1.0 if ca == best else 0.0) for ca in canonical_actions
            }

    # -- Helpers -----------------------------------------------------------

    def _to_canonical_actions(
        self, actions: list[int], is_canonical: bool
    ) -> list[int]:
        if is_canonical:
            return list(actions)
        return [self._n - 1 - a for a in actions]

    def _encode_history(self, state: DarkHexState) -> np.ndarray:
        """Encode full history (both players' canonical info states)."""
        canon_b, _ = state.canonical_info_state(Player.Black)
        canon_w, _ = state.canonical_info_state(Player.White)
        enc_b = encode_info_state(
            canon_b, self.cfg.rows, self.cfg.cols
        ).numpy()
        enc_w = encode_info_state(
            canon_w, self.cfg.rows, self.cfg.cols
        ).numpy()
        return np.concatenate([enc_b, enc_w])

    # -- Strategy extraction -----------------------------------------------

    def extract_strategy(self) -> dict[str, list[tuple[int, float]]]:
        """Extract strategy from average policy network."""
        result: dict[str, list[tuple[int, float]]] = {}
        state = DarkHexState(self.cfg.rows, self.cfg.cols)
        self._extract_recursive(state, result, set())
        return result

    def _extract_recursive(
        self,
        state: DarkHexState,
        result: dict[str, list[tuple[int, float]]],
        visited: set[str],
    ) -> None:
        if state.is_terminal():
            return

        is_b = state.info_state_string(Player.Black)
        is_w = state.info_state_string(Player.White)
        game_key = is_b + "|" + is_w
        if game_key in visited:
            return
        visited.add(game_key)

        player_enum = state.current_player()
        canonical_str, is_canonical = state.canonical_info_state(player_enum)

        if canonical_str not in result:
            actions = state.legal_actions()
            canonical_actions = self._to_canonical_actions(
                actions, is_canonical
            )

            with torch.no_grad():
                x = encode_info_state(
                    canonical_str, self.cfg.rows, self.cfg.cols
                ).unsqueeze(0)
                logits = self._policy_net(x).squeeze(0)

                # Softmax over legal actions only
                legal_logits = torch.full((self._n,), float('-inf'))
                for ca in canonical_actions:
                    legal_logits[ca] = logits[ca]
                probs = F.softmax(legal_logits, dim=0)

            result[canonical_str] = [
                (ca, probs[ca].item())
                for ca in canonical_actions
                if probs[ca].item() > 1e-6
            ]

            if not result[canonical_str]:
                n_legal = len(canonical_actions)
                result[canonical_str] = [
                    (ca, 1.0 / n_legal) for ca in canonical_actions
                ]

        for action in state.legal_actions():
            child = state.copy()
            child.apply_action(action)
            self._extract_recursive(child, result, visited)

    def exploitability(self) -> float:
        """Compute exploitability of the current average policy."""
        strategy = self.extract_strategy()
        _, _, expl = best_response_values(
            self.cfg.rows, self.cfg.cols, strategy
        )
        return expl
