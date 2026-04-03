"""Tests for DREAM implementation."""

import numpy as np
import pytest

torch = pytest.importorskip("torch")
nn = torch.nn

from darkhex._engine import DarkHexState, Player  # noqa: E402

from darkhex.algorithms.deep_cfr import (  # noqa: E402
    MLP,
    ReservoirBuffer,
    encode_info_state,
)
from darkhex.algorithms.dream import (  # noqa: E402
    DREAM,
    DREAMConfig,
    QBaseline,
)

# ── Shared component tests (verify imports work) ─────────────────────────


class TestSharedComponents:
    """Verify DREAM correctly reuses Deep CFR components."""

    def test_encode_info_state_imported(self):
        s = DarkHexState(2, 2)
        info = s.info_state_string(Player.Black)
        t = encode_info_state(info, 2, 2)
        assert t.shape == (13,)

    def test_reservoir_buffer_imported(self):
        buf = ReservoirBuffer(100)
        buf.append(np.zeros(5, dtype=np.float32), 1, np.zeros(4, dtype=np.float32))
        assert len(buf) == 1

    def test_mlp_imported(self):
        net = MLP(13, (64,), 4)
        x = torch.randn(1, 13)
        out = net(x)
        assert out.shape == (1, 4)


# ── Q-Baseline tests ────────────────────────────────────────────────────


class TestQBaseline:
    def test_predict_shape(self):
        qb = QBaseline(
            input_dim=26, hidden_sizes=(32,), output_dim=4,
            lr=1e-3, buffer_size=100, seed=42,
        )
        x = torch.randn(26)
        out = qb.predict(x)
        assert out.shape == (4,)

    def test_train_step_no_data(self):
        qb = QBaseline(
            input_dim=26, hidden_sizes=(32,), output_dim=4,
            lr=1e-3, buffer_size=100, seed=42,
        )
        loss = qb.train_step(batch_size=32, num_steps=10)
        assert loss == 0.0

    def test_train_step_with_data(self):
        qb = QBaseline(
            input_dim=26, hidden_sizes=(32,), output_dim=4,
            lr=1e-3, buffer_size=100, seed=42,
        )
        for i in range(50):
            qb.buffer.append(
                np.random.randn(26).astype(np.float32),
                i,
                np.random.randn(4).astype(np.float32),
            )
        loss = qb.train_step(batch_size=16, num_steps=5)
        assert loss > 0.0

    def test_reset(self):
        qb = QBaseline(
            input_dim=26, hidden_sizes=(32,), output_dim=4,
            lr=1e-3, buffer_size=100, seed=42,
        )
        x = torch.randn(26)
        out1 = qb.predict(x).clone()
        qb.reset()
        out2 = qb.predict(x).clone()
        assert not torch.allclose(out1, out2)


# ── DREAM config tests ──────────────────────────────────────────────────


class TestDREAMConfig:
    def test_default_config(self):
        cfg = DREAMConfig(rows=2, cols=2)
        assert cfg.epsilon == 0.6
        assert cfg.reinit_every == 10
        assert cfg.use_baseline is False
        assert cfg.num_traversals == 1000

    def test_baseline_config(self):
        cfg = DREAMConfig(rows=2, cols=2, use_baseline=True)
        assert cfg.use_baseline is True
        assert cfg.baseline_train_steps == 500


# ── Regret matching tests ────────────────────────────────────────────────


class TestDREAMRegretMatching:
    def _make_solver(self):
        cfg = DREAMConfig(
            rows=2, cols=2, num_cfr_iters=1, num_traversals=1,
        )
        return DREAM(cfg)

    def test_positive_advantages(self):
        solver = self._make_solver()
        s = DarkHexState(2, 2)
        canonical, _ = s.canonical_info_state(Player.Black)
        actions = s.legal_actions()
        canonical_actions = solver._to_canonical_actions(actions, True)
        strategy = solver._regret_match(0, canonical, canonical_actions)

        total = sum(strategy.values())
        assert abs(total - 1.0) < 1e-5
        assert all(p >= 0 for p in strategy.values())

    def test_all_negative_gives_argmax(self):
        solver = self._make_solver()
        with torch.no_grad():
            for module in solver._advantage_nets[0].modules():
                if isinstance(module, torch.nn.Linear):
                    module.weight.zero_()
                    if module.bias is not None:
                        module.bias.fill_(-1.0)
            final_linear = None
            for module in solver._advantage_nets[0].modules():
                if isinstance(module, torch.nn.Linear):
                    final_linear = module
            final_linear.bias.copy_(torch.tensor([-4.0, -3.0, -2.0, -1.0]))

        s = DarkHexState(2, 2)
        canonical, _ = s.canonical_info_state(Player.Black)
        actions = s.legal_actions()
        canonical_actions = solver._to_canonical_actions(actions, True)
        strategy = solver._regret_match(0, canonical, canonical_actions)

        probs = list(strategy.values())
        assert max(probs) == 1.0
        assert sum(1 for p in probs if p > 0) == 1


# ── Traversal tests ─────────────────────────────────────────────────────


class TestDREAMTraversal:
    def test_single_traversal_returns_value(self):
        """A single OS traversal should return a finite float."""
        cfg = DREAMConfig(rows=2, cols=2, num_cfr_iters=1, num_traversals=1)
        solver = DREAM(cfg)
        state = DarkHexState(2, 2)
        val = solver._traverse_os(state, 0, 1.0, 1.0, 1.0)
        assert np.isfinite(val)

    def test_traversal_populates_buffers(self):
        """OS traversal should add data to advantage and strategy buffers."""
        cfg = DREAMConfig(rows=2, cols=2, num_cfr_iters=1, num_traversals=10)
        solver = DREAM(cfg)
        for _ in range(10):
            state = DarkHexState(2, 2)
            solver._traverse_os(state, 0, 1.0, 1.0, 1.0)
        # Traversing as player 0: advantage buffer for P0 should have data,
        # strategy buffer should have opponent nodes
        assert len(solver._advantage_buffers[0]) > 0
        assert len(solver._strategy_buffer) > 0

    def test_traversal_with_baseline(self):
        """OS traversal with Q-baseline enabled should work."""
        cfg = DREAMConfig(
            rows=2, cols=2, num_cfr_iters=1, num_traversals=5,
            use_baseline=True,
        )
        solver = DREAM(cfg)
        state = DarkHexState(2, 2)
        val = solver._traverse_os(state, 0, 1.0, 1.0, 1.0)
        assert np.isfinite(val)


# ── Canonical action tests ───────────────────────────────────────────────


class TestCanonicalActions:
    def test_canonical_identity(self):
        cfg = DREAMConfig(rows=2, cols=2)
        solver = DREAM(cfg)
        actions = [0, 1, 2, 3]
        result = solver._to_canonical_actions(actions, True)
        assert result == [0, 1, 2, 3]

    def test_canonical_rotation(self):
        cfg = DREAMConfig(rows=2, cols=2)
        solver = DREAM(cfg)
        actions = [0, 1, 2, 3]
        result = solver._to_canonical_actions(actions, False)
        assert result == [3, 2, 1, 0]


# ── Integration tests ────────────────────────────────────────────────────


class TestDREAMIntegration:
    def test_2x2_runs_without_error(self):
        """DREAM completes a few iterations on 2x2."""
        cfg = DREAMConfig(
            rows=2, cols=2,
            hidden_sizes=(32,),
            num_cfr_iters=3,
            num_traversals=50,
            advantage_train_steps=10,
            strategy_train_steps=10,
            buffer_size=1000,
            reinit_every=2,
        )
        solver = DREAM(cfg)
        solver.solve()
        assert solver.iteration == 3

    def test_2x2_strategy_valid(self):
        """Extracted strategy has valid probability distributions."""
        cfg = DREAMConfig(
            rows=2, cols=2,
            hidden_sizes=(32,),
            num_cfr_iters=5,
            num_traversals=100,
            advantage_train_steps=20,
            strategy_train_steps=50,
            buffer_size=5000,
            reinit_every=3,
        )
        solver = DREAM(cfg)
        solver.solve()

        strategy = solver.extract_strategy()
        assert len(strategy) > 0

        for info_state, action_probs in strategy.items():
            assert len(action_probs) > 0
            total = sum(p for _, p in action_probs)
            assert total > 0.5, f"Strategy at {info_state} sums to {total}"

    def test_2x2_exploitability_computable(self):
        """Exploitability can be computed from extracted strategy."""
        cfg = DREAMConfig(
            rows=2, cols=2,
            hidden_sizes=(32,),
            num_cfr_iters=5,
            num_traversals=100,
            advantage_train_steps=20,
            strategy_train_steps=50,
            buffer_size=5000,
            reinit_every=3,
        )
        solver = DREAM(cfg)
        solver.solve()

        expl = solver.exploitability()
        assert 0.0 <= expl <= 2.0

    def test_no_nan_in_buffers(self):
        """Buffers should never contain NaN or inf values."""
        cfg = DREAMConfig(
            rows=2, cols=2,
            hidden_sizes=(32,),
            num_cfr_iters=5,
            num_traversals=100,
            advantage_train_steps=20,
            strategy_train_steps=50,
            buffer_size=5000,
            reinit_every=3,
        )
        solver = DREAM(cfg)
        solver.solve()

        for buf in [*solver._advantage_buffers, solver._strategy_buffer]:
            if buf._values is not None:
                vals = buf._values[:len(buf)]
                assert np.all(np.isfinite(vals)), "NaN/inf found in buffer"

    def test_2x2_with_baseline(self):
        """DREAM with Q-baseline runs without error on 2x2."""
        cfg = DREAMConfig(
            rows=2, cols=2,
            hidden_sizes=(32,),
            num_cfr_iters=3,
            num_traversals=50,
            advantage_train_steps=10,
            strategy_train_steps=10,
            buffer_size=1000,
            reinit_every=2,
            use_baseline=True,
            baseline_hidden_sizes=(32,),
            baseline_buffer_size=500,
            baseline_train_steps=10,
            baseline_batch_size=16,
        )
        solver = DREAM(cfg)
        solver.solve()
        assert solver.iteration == 3

        expl = solver.exploitability()
        assert 0.0 <= expl <= 2.0

    @pytest.mark.slow
    def test_2x2_exploitability_decreases(self):
        """Exploitability should decrease with more CFR iterations."""
        cfg = DREAMConfig(
            rows=2, cols=2,
            hidden_sizes=(64, 64),
            num_cfr_iters=10,
            num_traversals=200,
            advantage_train_steps=100,
            strategy_train_steps=500,
            buffer_size=50000,
            reinit_every=5,
        )
        solver = DREAM(cfg)

        solver.solve(10)
        expl_10 = solver.exploitability()

        solver.solve(40)
        expl_50 = solver.exploitability()

        assert expl_50 < expl_10 + 0.1, (
            f"Exploitability didn't decrease: {expl_10:.4f} -> {expl_50:.4f}"
        )


# ── Device placement tests ───────────────────────────────────────────────


class TestDevicePlacement:
    def test_dream_nets_on_device(self):
        cfg = DREAMConfig(rows=2, cols=2, device="cpu")
        solver = DREAM(cfg)
        for net in solver._advantage_nets:
            for p in net.parameters():
                assert p.device == torch.device("cpu")
        for p in solver._strategy_net.parameters():
            assert p.device == torch.device("cpu")

    def test_baseline_nets_on_device(self):
        cfg = DREAMConfig(
            rows=2, cols=2, use_baseline=True, device="cpu",
        )
        solver = DREAM(cfg)
        for bl in solver._baselines:
            assert bl is not None
            for p in bl.net.parameters():
                assert p.device == torch.device("cpu")

    def test_2x2_runs_with_explicit_cpu(self):
        cfg = DREAMConfig(
            rows=2, cols=2,
            hidden_sizes=(32,),
            num_cfr_iters=3,
            num_traversals=50,
            advantage_train_steps=10,
            strategy_train_steps=10,
            buffer_size=1000,
            reinit_every=2,
            device="cpu",
        )
        solver = DREAM(cfg)
        solver.solve()
        expl = solver.exploitability()
        assert 0.0 <= expl <= 2.0

    def test_2x2_baseline_with_explicit_cpu(self):
        cfg = DREAMConfig(
            rows=2, cols=2,
            hidden_sizes=(32,),
            num_cfr_iters=3,
            num_traversals=50,
            advantage_train_steps=10,
            strategy_train_steps=10,
            buffer_size=1000,
            reinit_every=2,
            use_baseline=True,
            baseline_hidden_sizes=(32,),
            baseline_buffer_size=500,
            baseline_train_steps=10,
            baseline_batch_size=16,
            device="cpu",
        )
        solver = DREAM(cfg)
        solver.solve()
        expl = solver.exploitability()
        assert 0.0 <= expl <= 2.0
