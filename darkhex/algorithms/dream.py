"""DREAM: Deep Regret minimization with Advantage baselines and Model-free learning.

Steinberger, E., Lerer, A. & Brown, N. (2020).
DREAM: Deep Regret minimization with Advantage baselines and Model-free learning.
arXiv:2006.10410.

Uses Outcome Sampling (O(depth) per traversal) instead of External Sampling,
making neural CFR feasible for large games like 4x3+ Dark Hex.

Reference: OpenSpiel outcome_sampling_mccfr.py + deep_cfr.py
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

from darkhex._engine import DarkHexState, Player, best_response_values
from darkhex.algorithms.deep_cfr import (
    MLP,
    ReservoirBuffer,
    encode_info_state,
    resolve_device,
    _player_index,
)


# ---------------------------------------------------------------------------
# Q-Baseline network (Phase 2: variance reduction)
# ---------------------------------------------------------------------------


class QBaseline:
    """Learned Q-baseline for variance reduction in Outcome Sampling.

    Takes BOTH players' info states as input (privileged information
    available during training but not at test time). Predicts per-action
    values used to reduce variance of OS value estimates.

    Based on ESCHER's value network approach (McAleer et al., 2022) and
    DREAM's Q-baseline (Steinberger et al., 2020).
    """

    def __init__(
        self,
        input_dim: int,
        hidden_sizes: tuple[int, ...],
        output_dim: int,
        lr: float,
        buffer_size: int,
        seed: int,
        device: torch.device | None = None,
    ) -> None:
        self._device = device or torch.device("cpu")
        # Input: concatenated info states of both players
        self.net = MLP(input_dim, hidden_sizes, output_dim).to(self._device)
        self.buffer = ReservoirBuffer(buffer_size, seed=seed)
        self.optimizer = torch.optim.Adam(self.net.parameters(), lr=lr)
        self._input_dim = input_dim

    def predict(self, joint_info_state: torch.Tensor) -> torch.Tensor:
        """Predict Q-values for all actions given joint info state."""
        with torch.no_grad():
            return self.net(joint_info_state.unsqueeze(0).to(self._device)).squeeze(0)

    def train_step(
        self,
        batch_size: int,
        num_steps: int,
    ) -> float:
        """Train Q-baseline on buffer data. Returns average loss."""
        if len(self.buffer) < batch_size:
            return 0.0

        self.net.train()
        total_loss = 0.0

        for _ in range(num_steps):
            states, _, targets = self.buffer.sample(batch_size)
            states = states.to(self._device)
            targets = targets.to(self._device)
            preds = self.net(states)
            loss = F.mse_loss(preds, targets)

            self.optimizer.zero_grad()
            loss.backward()
            self.optimizer.step()
            total_loss += loss.item()

        self.net.eval()
        return total_loss / num_steps

    def reset(self) -> None:
        """Reinitialize network parameters."""
        self.net.reset()
        self.optimizer = torch.optim.Adam(
            self.net.parameters(), lr=self.optimizer.defaults["lr"]
        )


# ---------------------------------------------------------------------------
# DREAM config
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class DREAMConfig:
    """Configuration for DREAM solver."""

    rows: int
    cols: int
    hidden_sizes: tuple[int, ...] = (128, 128)
    lr: float = 1e-3
    buffer_size: int = 1_000_000
    batch_size_advantage: int = 256
    batch_size_strategy: int = 256
    advantage_train_steps: int = 500
    strategy_train_steps: int = 2500
    num_cfr_iters: int = 200
    num_traversals: int = 1000  # OS is cheap, can do many more than ES
    epsilon: float = 0.6  # Exploration rate for outcome sampling
    seed: int = 42
    reinit_every: int = 10  # Reset adv nets every N iters (paper best)

    # Phase 2: Q-baseline
    use_baseline: bool = False
    baseline_hidden_sizes: tuple[int, ...] = (128, 128)
    baseline_buffer_size: int = 200_000
    baseline_train_steps: int = 500
    baseline_batch_size: int = 256
    device: str = "cpu"


# ---------------------------------------------------------------------------
# DREAM solver
# ---------------------------------------------------------------------------


class DREAM:
    """DREAM solver for Dark Hex.

    Combines Outcome Sampling traversal with neural function approximation:
    - Advantage networks predict per-action regrets (one per player)
    - Strategy network approximates the average policy
    - Optional Q-baseline network reduces OS variance (Phase 2)

    Key differences from Deep CFR:
    - Outcome Sampling: O(depth) per traversal, not O(branching^depth)
    - Importance-weighted regret updates
    - Advantage nets reset every N iterations, not every iteration
    - Optional learned baseline for variance reduction
    """

    def __init__(self, cfg: DREAMConfig) -> None:
        self.cfg = cfg
        self._n = cfg.rows * cfg.cols
        self._input_dim = 3 * self._n + 1
        self._iteration = 0
        self._device = resolve_device(cfg.device)

        torch.manual_seed(cfg.seed)
        self._rng = np.random.default_rng(cfg.seed)

        # One advantage net per player, one shared strategy net
        self._advantage_nets = [
            MLP(self._input_dim, cfg.hidden_sizes, self._n).to(self._device)
            for _ in range(2)
        ]
        self._strategy_net = MLP(
            self._input_dim,
            cfg.hidden_sizes,
            self._n,
            final_activation=nn.Softmax(dim=-1),
        ).to(self._device)

        # Reservoir buffers
        self._advantage_buffers = [
            ReservoirBuffer(cfg.buffer_size, seed=cfg.seed + 1),
            ReservoirBuffer(cfg.buffer_size, seed=cfg.seed + 2),
        ]
        # Strategy buffer stores [sigma_0, ..., sigma_{n-1}, strat_weight]
        # where sigma is the raw policy (sums to 1) and strat_weight is
        # the OS importance weight (pi_i / pi_sample). The weight is applied
        # as a per-sample loss multiplier during training, NOT baked into
        # the target, because the strategy net uses Softmax (unit-sum output).
        self._strategy_buffer = ReservoirBuffer(
            cfg.buffer_size, seed=cfg.seed + 3
        )

        # Strategy net optimizer persists across iterations
        self._strategy_optimizer = torch.optim.Adam(
            self._strategy_net.parameters(), lr=cfg.lr
        )

        # Phase 2: Q-baseline (one per player)
        self._baselines: list[QBaseline | None] = [None, None]
        if cfg.use_baseline:
            # Q-baseline input: both players' info states concatenated
            baseline_input_dim = 2 * self._input_dim
            for p in range(2):
                self._baselines[p] = QBaseline(
                    input_dim=baseline_input_dim,
                    hidden_sizes=cfg.baseline_hidden_sizes,
                    output_dim=self._n,
                    lr=cfg.lr,
                    buffer_size=cfg.baseline_buffer_size,
                    seed=cfg.seed + 10 + p,
                    device=self._device,
                )

    @property
    def iteration(self) -> int:
        return self._iteration

    # -- Main solve loop ---------------------------------------------------

    def solve(self, num_cfr_iters: int | None = None) -> None:
        """Run DREAM for the specified number of CFR iterations."""
        T = num_cfr_iters or self.cfg.num_cfr_iters
        for _ in range(T):
            self._iteration += 1

            for player in range(2):
                # K outcome sampling traversals per player
                for _ in range(self.cfg.num_traversals):
                    state = DarkHexState(self.cfg.rows, self.cfg.cols)
                    self._traverse_os(state, player, 1.0, 1.0, 1.0)

                # Train advantage net for this player
                self._train_advantage_net(player)

                # Train Q-baseline if enabled
                if self._baselines[player] is not None:
                    self._baselines[player].train_step(
                        self.cfg.baseline_batch_size,
                        self.cfg.baseline_train_steps,
                    )

            # Train strategy net after both players
            self._train_strategy_net()

    # -- Outcome Sampling traversal ----------------------------------------

    def _traverse_os(
        self,
        state: DarkHexState,
        traverser: int,
        pi_i: float,
        pi_opp: float,
        pi_sample: float,
    ) -> float:
        """Outcome Sampling traversal with optional baseline correction.

        Samples ONE action at ALL decision nodes (O(depth) per traversal).
        Uses importance weighting to correct for the sampling bias.

        Based on OpenSpiel's outcome_sampling_mccfr.py + DREAM paper Eq. 6-8.
        """
        if state.is_terminal():
            return state.returns()[traverser]

        player_enum = state.current_player()
        player = _player_index(player_enum)
        actions = state.legal_actions()
        num_actions = len(actions)

        # Canonicalize info state
        canonical_str, is_canonical = state.canonical_info_state(player_enum)
        canonical_actions = self._to_canonical_actions(actions, is_canonical)

        # Get strategy via regret matching on advantage net
        sigma = self._regret_match(player, canonical_str, canonical_actions)

        # Compute sampling policy (epsilon-greedy for traverser)
        is_update = player == traverser
        if is_update:
            eps = self.cfg.epsilon
            uniform_p = 1.0 / num_actions
            q = {
                ca: eps * uniform_p + (1.0 - eps) * sigma[ca]
                for ca in canonical_actions
            }
        else:
            q = dict(sigma)

        # Sample ONE action
        q_arr = np.array(
            [q[ca] for ca in canonical_actions], dtype=np.float64
        )
        q_arr /= q_arr.sum()  # renormalize for numerical safety
        idx = self._rng.choice(len(canonical_actions), p=q_arr)
        sampled_ca = canonical_actions[idx]
        sampled_action = actions[idx]
        q_a = q_arr[idx]

        # Update reach probabilities
        sigma_a = sigma[sampled_ca]
        if is_update:
            new_pi_i = pi_i * sigma_a
            new_pi_opp = pi_opp
        else:
            new_pi_i = pi_i
            new_pi_opp = pi_opp * sigma_a
        new_pi_sample = pi_sample * q_a

        # Recurse on sampled action
        child = state.copy()
        child.apply_action(sampled_action)
        child_value = self._traverse_os(
            child, traverser, new_pi_i, new_pi_opp, new_pi_sample
        )

        # Compute baseline-corrected child values for each action
        # (Eq. 9 from Schmid et al. '19, used by OpenSpiel OS-MCCFR)
        child_values = np.zeros(num_actions, dtype=np.float64)

        if self._baselines[traverser] is not None and is_update:
            # Phase 2: Use Q-baseline for variance reduction
            joint_info = self._encode_joint_info_state(state)
            baseline_vals = self._baselines[traverser].predict(joint_info)

            for i, ca in enumerate(canonical_actions):
                baseline_a = baseline_vals[ca].item()
                if i == idx:
                    # Sampled action: baseline + importance-corrected residual
                    child_values[i] = baseline_a + (
                        child_value - baseline_a
                    ) / q_a
                else:
                    # Non-sampled: use baseline directly
                    child_values[i] = baseline_a

            # Store baseline training data: joint_info -> observed values
            target_vec = np.zeros(self._n, dtype=np.float32)
            for i, ca in enumerate(canonical_actions):
                target_vec[ca] = child_values[i]
            self._baselines[traverser].buffer.append(
                joint_info.numpy(),
                self._iteration,
                target_vec,
            )
        else:
            # Phase 1: Vanilla OS (baseline = 0)
            for i, ca in enumerate(canonical_actions):
                if i == idx:
                    child_values[i] = child_value / q_a
                else:
                    child_values[i] = 0.0

        # Value estimate under current strategy
        value_estimate = sum(
            sigma[ca] * child_values[i]
            for i, ca in enumerate(canonical_actions)
        )

        if is_update:
            # Counterfactual weight (guard against numerical instability)
            cf_prefix = pi_opp / pi_sample
            if not np.isfinite(cf_prefix) or abs(cf_prefix) > 1e6:
                return value_estimate

            # Compute advantages and store in buffer
            # (cf_action_value - cf_value for each action)
            advantages = np.zeros(self._n, dtype=np.float32)
            cf_value = value_estimate * cf_prefix

            for i, ca in enumerate(canonical_actions):
                cf_action_value = child_values[i] * cf_prefix
                advantages[ca] = cf_action_value - cf_value

            info_tensor = encode_info_state(
                canonical_str, self.cfg.rows, self.cfg.cols
            ).numpy()
            self._advantage_buffers[player].append(
                info_tensor, self._iteration, advantages
            )
        else:
            # Opponent node: store raw strategy + importance weight.
            # In OS, the average strategy must be importance-weighted
            # by pi_i / pi_sample. We store raw sigma (unit-sum) as
            # the regression target and the weight separately, to be
            # applied as a per-sample loss multiplier during training.
            # This keeps targets compatible with the Softmax output.
            strat_weight = pi_i / pi_sample
            if not np.isfinite(strat_weight) or abs(strat_weight) > 1e6:
                return value_estimate

            # Layout: [sigma_0, ..., sigma_{n-1}, strat_weight]
            strategy_vec = np.zeros(self._n + 1, dtype=np.float32)
            for ca, prob in sigma.items():
                strategy_vec[ca] = prob
            strategy_vec[self._n] = strat_weight

            info_tensor = encode_info_state(
                canonical_str, self.cfg.rows, self.cfg.cols
            ).numpy()
            self._strategy_buffer.append(
                info_tensor, self._iteration, strategy_vec
            )

        return value_estimate

    # -- Regret matching (same as Deep CFR) --------------------------------

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
            ).unsqueeze(0).to(self._device)
            advantages = self._advantage_nets[player](x).squeeze(0)

        positive = {
            ca: max(0.0, advantages[ca].item()) for ca in canonical_actions
        }
        total = sum(positive.values())

        if total > 0:
            return {ca: positive[ca] / total for ca in canonical_actions}
        else:
            # Argmax tiebreaker (per Deep CFR paper ablation)
            best = max(
                canonical_actions, key=lambda ca: advantages[ca].item()
            )
            return {
                ca: (1.0 if ca == best else 0.0) for ca in canonical_actions
            }

    # -- Network training (same as Deep CFR with LCFR weighting) -----------

    def _train_advantage_net(self, player: int) -> float:
        """Train advantage net on buffer with LCFR weighting."""
        buf = self._advantage_buffers[player]
        if len(buf) < self.cfg.batch_size_advantage:
            return 0.0

        net = self._advantage_nets[player]

        # Reinitialize periodically (DREAM paper: best at every 10 iters)
        if self._iteration % self.cfg.reinit_every == 0:
            net.reset()

        optimizer = torch.optim.Adam(net.parameters(), lr=self.cfg.lr)
        net.train()
        total_loss = 0.0

        for _ in range(self.cfg.advantage_train_steps):
            states, iterations, targets = buf.sample(
                self.cfg.batch_size_advantage
            )
            states = states.to(self._device)
            iterations = iterations.to(self._device)
            targets = targets.to(self._device)
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
        """Train strategy net on buffer with LCFR + OS importance weighting.

        Buffer values layout: [sigma_0, ..., sigma_{n-1}, strat_weight].
        The raw sigma (unit-sum) is the regression target for the Softmax
        output. The importance weight (pi_i / pi_sample) is applied as a
        per-sample loss multiplier alongside the LCFR sqrt(t) weight.
        """
        buf = self._strategy_buffer
        if len(buf) < self.cfg.batch_size_strategy:
            return 0.0

        self._strategy_net.train()
        total_loss = 0.0

        for _ in range(self.cfg.strategy_train_steps):
            states, iterations, raw_vals = buf.sample(
                self.cfg.batch_size_strategy
            )
            states = states.to(self._device)
            iterations = iterations.to(self._device)
            raw_vals = raw_vals.to(self._device)
            # Unpack: targets = sigma (first n columns), weights = last column
            targets = raw_vals[:, : self._n]
            strat_weights = raw_vals[:, self._n].unsqueeze(1)

            # Combined weight: LCFR (sqrt(t)) * OS importance weight
            lcfr_weights = torch.sqrt(iterations.float()).unsqueeze(1)
            weights = lcfr_weights * torch.sqrt(strat_weights.abs().clamp(min=1e-8))

            preds = self._strategy_net(states)
            loss = F.mse_loss(weights * preds, weights * targets)

            self._strategy_optimizer.zero_grad()
            loss.backward()
            self._strategy_optimizer.step()
            total_loss += loss.item()

        self._strategy_net.eval()
        return total_loss / self.cfg.strategy_train_steps

    # -- Helpers -----------------------------------------------------------

    def _to_canonical_actions(
        self, actions: list[int], is_canonical: bool
    ) -> list[int]:
        if is_canonical:
            return list(actions)
        return [self._n - 1 - a for a in actions]

    def _encode_joint_info_state(self, state: DarkHexState) -> torch.Tensor:
        """Encode both players' canonical info states for Q-baseline input.

        Uses canonical (possibly 180°-rotated) info states so the encoding
        is consistent with canonical action indices used for baseline
        predictions and targets.
        """
        canon_b, _ = state.canonical_info_state(Player.Black)
        canon_w, _ = state.canonical_info_state(Player.White)
        enc_b = encode_info_state(canon_b, self.cfg.rows, self.cfg.cols)
        enc_w = encode_info_state(canon_w, self.cfg.rows, self.cfg.cols)
        return torch.cat([enc_b, enc_w])

    # -- Strategy extraction (same as Deep CFR) ----------------------------

    def extract_strategy(self) -> dict[str, list[tuple[int, float]]]:
        """Extract strategy from strategy net via game tree walk.

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
        visited: set[str],
    ) -> None:
        if state.is_terminal():
            return

        # Memoize by observable state to avoid exponential re-traversal
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
                ).unsqueeze(0).to(self._device)
                probs = self._strategy_net(x).squeeze(0)

            # Renormalize over legal actions only
            legal_probs = {ca: probs[ca].item() for ca in canonical_actions}
            total = sum(legal_probs.values())
            if total > 1e-8:
                result[canonical_str] = [
                    (ca, p / total)
                    for ca, p in legal_probs.items()
                    if p / total > 1e-6
                ]
            else:
                n_legal = len(canonical_actions)
                result[canonical_str] = [
                    (ca, 1.0 / n_legal) for ca in canonical_actions
                ]

        for action in state.legal_actions():
            child = state.copy()
            child.apply_action(action)
            self._extract_recursive(child, result, visited)

    def exploitability(self) -> float:
        """Compute exploitability of the current strategy."""
        strategy = self.extract_strategy()
        _, _, expl = best_response_values(
            self.cfg.rows, self.cfg.cols, strategy
        )
        return expl
