"""Tests for ESCHER implementation."""

import numpy as np
import pytest

torch = pytest.importorskip("torch")

from darkhex._engine import DarkHexState  # noqa: E402

from darkhex.algorithms.escher import ESCHER, ESCHERConfig  # noqa: E402

# ── Config tests ──────────────────────────────────────────────────────────


class TestESCHERConfig:
    def test_default_config(self):
        cfg = ESCHERConfig(rows=2, cols=2)
        assert cfg.num_iters == 100
        assert cfg.value_exploration == 0.1
        assert cfg.device == "cpu"

    def test_explicit_device(self):
        cfg = ESCHERConfig(rows=2, cols=2, device="cpu")
        assert cfg.device == "cpu"


# ── Device placement tests ───────────────────────────────────────────────


class TestDevicePlacement:
    def test_all_nets_on_device(self):
        cfg = ESCHERConfig(rows=2, cols=2, device="cpu")
        solver = ESCHER(cfg)
        for net in solver._value_nets:
            for p in net.parameters():
                assert p.device == torch.device("cpu")
        for net in solver._regret_nets:
            for p in net.parameters():
                assert p.device == torch.device("cpu")
        for p in solver._policy_net.parameters():
            assert p.device == torch.device("cpu")


# ── Integration tests ────────────────────────────────────────────────────


class TestESCHERIntegration:
    def _make_small_solver(self, **overrides):
        defaults = dict(
            rows=2, cols=2,
            value_hidden=(32,),
            regret_hidden=(32,),
            policy_hidden=(32,),
            value_buffer_size=1000,
            regret_buffer_size=1000,
            policy_buffer_size=1000,
            value_batch_size=16,
            value_train_steps=10,
            regret_batch_size=16,
            regret_train_steps=10,
            policy_batch_size=16,
            policy_train_steps=10,
            value_traversals=20,
            regret_traversals=20,
            num_iters=3,
            device="cpu",
        )
        defaults.update(overrides)
        cfg = ESCHERConfig(**defaults)
        return ESCHER(cfg)

    def test_2x2_runs_without_error(self):
        solver = self._make_small_solver()
        solver.solve()
        assert solver.iteration == 3

    def test_2x2_strategy_valid(self):
        solver = self._make_small_solver(num_iters=5)
        solver.solve()
        strategy = solver.extract_strategy()
        assert len(strategy) > 0
        for info_state, action_probs in strategy.items():
            assert len(action_probs) > 0
            total = sum(p for _, p in action_probs)
            assert total > 0.5, f"Strategy at {info_state} sums to {total}"

    def test_2x2_exploitability_computable(self):
        solver = self._make_small_solver(num_iters=5)
        solver.solve()
        expl = solver.exploitability()
        assert 0.0 <= expl <= 2.0

    def test_compute_regret_returns_finite(self):
        solver = self._make_small_solver()
        state = DarkHexState(2, 2)
        actions = state.legal_actions()
        policy = np.ones(len(actions), dtype=np.float64) / len(actions)
        regret = solver._compute_regret(state, policy, 0)
        assert np.all(np.isfinite(regret))
        assert len(regret) == len(actions)
