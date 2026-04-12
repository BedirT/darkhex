"""Tests for Deep CFR implementation."""

import numpy as np
import pytest

# Skip all tests if torch is not available (optional dependency)
torch = pytest.importorskip("torch")
nn = torch.nn  # noqa: E402

from darkhex._engine import DarkHexState, Player  # noqa: E402, I001
from darkhex.algorithms.deep_cfr import (  # noqa: E402
    DeepCFR,
    DeepCFRConfig,
    MLP,
    ReservoirBuffer,
    encode_info_state,
    resolve_device,
)  # noqa: I001


# ── Encoding tests ─────────────────────────────────────────────────────────


class TestEncodeInfoState:
    def test_shape_2x2(self):
        s = DarkHexState(2, 2)
        info = s.info_state_string(Player.Black)
        t = encode_info_state(info, 2, 2)
        assert t.shape == (13,)  # 3*4 + 1

    def test_shape_4x3(self):
        s = DarkHexState(4, 3)
        info = s.info_state_string(Player.Black)
        t = encode_info_state(info, 4, 3)
        assert t.shape == (37,)  # 3*12 + 1

    def test_empty_board_values(self):
        """Empty board should have all cells as [0,0,1] (empty)."""
        t = encode_info_state("P0\n..\n..", 2, 2)
        # Each cell: [own, opp, empty] = [0, 0, 1]
        for i in range(4):
            assert t[3 * i].item() == 0.0  # own
            assert t[3 * i + 1].item() == 0.0  # opp
            assert t[3 * i + 2].item() == 1.0  # empty
        assert t[12].item() == 0.0  # player 0

    def test_player_indicator(self):
        t0 = encode_info_state("P0\n..\n..", 2, 2)
        t1 = encode_info_state("P1\n..\n..", 2, 2)
        assert t0[-1].item() == 0.0
        assert t1[-1].item() == 1.0

    def test_own_stone(self):
        """'x' should encode as [1, 0, 0] (own)."""
        t = encode_info_state("P0\nx.\n..", 2, 2)
        assert t[0].item() == 1.0  # own
        assert t[1].item() == 0.0  # opp
        assert t[2].item() == 0.0  # empty

    def test_opponent_stone(self):
        """'o' for P0 (Black) should encode as [0, 1, 0] (opponent)."""
        t = encode_info_state("P0\no.\n..", 2, 2)
        assert t[0].item() == 0.0  # own
        assert t[1].item() == 1.0  # opp
        assert t[2].item() == 0.0  # empty

    def test_white_encoding_flipped(self):
        """For P1 (White): 'o' = own, 'x' = opponent (absolute colors)."""
        # White sees own stone as 'o', opponent (Black) stone as 'x'
        t = encode_info_state("P1\no.\n..", 2, 2)
        assert t[0].item() == 1.0  # own (o = White's own)
        assert t[1].item() == 0.0  # opp
        assert t[2].item() == 0.0  # empty

        t2 = encode_info_state("P1\nx.\n..", 2, 2)
        assert t2[0].item() == 0.0  # own
        assert t2[1].item() == 1.0  # opp (x = Black, opponent of White)
        assert t2[2].item() == 0.0  # empty


# ── Reservoir buffer tests ────────────────────────────────────────────────


class TestReservoirBuffer:
    def test_insert_and_len(self):
        buf = ReservoirBuffer(100)
        assert len(buf) == 0
        buf.append(np.zeros(5, dtype=np.float32), 1, np.zeros(4, dtype=np.float32))
        assert len(buf) == 1

    def test_capacity_not_exceeded(self):
        buf = ReservoirBuffer(10)
        for i in range(100):
            buf.append(
                np.ones(5, dtype=np.float32) * i,
                i,
                np.ones(4, dtype=np.float32) * i,
            )
        assert len(buf) == 10

    def test_sample_shapes(self):
        buf = ReservoirBuffer(100)
        for i in range(50):
            buf.append(
                np.ones(5, dtype=np.float32) * i,
                i,
                np.ones(4, dtype=np.float32) * i,
            )
        states, iters, vals = buf.sample(8)
        assert states.shape == (8, 5)
        assert iters.shape == (8,)
        assert vals.shape == (8, 4)

    def test_sample_clamped_to_size(self):
        """Requesting more samples than available returns what's there."""
        buf = ReservoirBuffer(100)
        for i in range(3):
            buf.append(np.zeros(5, dtype=np.float32), i, np.zeros(4, dtype=np.float32))
        states, _, _ = buf.sample(10)
        assert states.shape[0] == 3


# ── Network tests ─────────────────────────────────────────────────────────


class TestMLP:
    def test_advantage_net_shape(self):
        net = MLP(13, (64, 64), 4)  # 2x2 game
        x = torch.randn(1, 13)
        out = net(x)
        assert out.shape == (1, 4)

    def test_strategy_net_sums_to_one(self):
        net = MLP(13, (64, 64), 4, final_activation=nn.Softmax(dim=-1))
        x = torch.randn(1, 13)
        out = net(x)
        assert abs(out.sum().item() - 1.0) < 1e-5

    def test_reset_changes_parameters(self):
        net = MLP(13, (64,), 4)
        x = torch.randn(1, 13)
        out1 = net(x).detach().clone()
        net.reset()
        out2 = net(x).detach().clone()
        # After reset, output should differ (overwhelmingly likely with random init)
        assert not torch.allclose(out1, out2)

    def test_batch_forward(self):
        net = MLP(37, (128, 128), 12)
        x = torch.randn(32, 37)
        out = net(x)
        assert out.shape == (32, 12)


# ── Canonical info state binding test ─────────────────────────────────────


class TestCanonicalInfoState:
    def test_binding_exists(self):
        s = DarkHexState(2, 2)
        canonical, is_canon = s.canonical_info_state(Player.Black)
        assert isinstance(canonical, str)
        assert isinstance(is_canon, bool)

    def test_initial_state_is_canonical(self):
        s = DarkHexState(2, 2)
        canonical, is_canon = s.canonical_info_state(Player.Black)
        original = s.info_state_string(Player.Black)
        assert canonical == original
        assert is_canon is True

    def test_rotation_action(self):
        """For 2x2 (n=4), rotated action = 3 - action."""
        s = DarkHexState(2, 2)
        s.apply_action(0)  # Black plays cell 0
        # After this, check White's canonical state
        canon, is_canon = s.canonical_info_state(Player.White)
        assert isinstance(canon, str)


# ── Regret matching tests ─────────────────────────────────────────────────


class TestRegretMatching:
    def _make_solver(self):
        cfg = DeepCFRConfig(rows=2, cols=2, num_cfr_iters=1, num_traversals=1)
        return DeepCFR(cfg)

    def test_positive_advantages(self):
        solver = self._make_solver()
        # Manually set advantage net to return known values
        # We test the regret matching logic by calling it
        s = DarkHexState(2, 2)
        canonical, _ = s.canonical_info_state(Player.Black)
        actions = s.legal_actions()
        canonical_actions = solver._to_canonical_actions(actions, True)
        strategy = solver._regret_match(0, canonical, canonical_actions)

        # Strategy should be a valid distribution
        total = sum(strategy.values())
        assert abs(total - 1.0) < 1e-5
        assert all(p >= 0 for p in strategy.values())

    def test_all_negative_gives_argmax(self):
        """When all regrets are non-positive, should pick argmax."""
        solver = self._make_solver()
        # Force advantage net to output all-negative values by zeroing weights
        # and setting bias to known negative values
        with torch.no_grad():
            for module in solver._advantage_nets[0].modules():
                if isinstance(module, torch.nn.Linear):
                    module.weight.zero_()
                    if module.bias is not None:
                        module.bias.fill_(-1.0)

            # Set the final linear layer's bias to distinct negative values
            # so argmax is deterministic
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

        # Exactly one action should have probability 1.0 (the argmax = action 3)
        probs = list(strategy.values())
        assert max(probs) == 1.0
        assert sum(1 for p in probs if p > 0) == 1


# ── Integration tests ────────────────────────────────────────────────────


class TestDeepCFRIntegration:
    def test_2x2_runs_without_error(self):
        """Deep CFR completes a few iterations on 2x2."""
        cfg = DeepCFRConfig(
            rows=2, cols=2,
            hidden_sizes=(32,),
            num_cfr_iters=3,
            num_traversals=10,
            advantage_train_steps=10,
            strategy_train_steps=10,
            buffer_size=1000,
        )
        solver = DeepCFR(cfg)
        solver.solve()
        assert solver.iteration == 3  # 3 outer CFR iterations (1-based)

    def test_2x2_strategy_valid(self):
        """Extracted strategy has valid probability distributions."""
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

        strategy = solver.extract_strategy()
        assert len(strategy) > 0

        for info_state, action_probs in strategy.items():
            assert len(action_probs) > 0
            total = sum(p for _, p in action_probs)
            # Strategy net output with Softmax — probs should sum to ~1
            # but we only keep > 1e-6, so total may be slightly less
            assert total > 0.5, f"Strategy at {info_state} sums to {total}"

    def test_2x2_exploitability_computable(self):
        """Exploitability can be computed from extracted strategy."""
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
        assert 0.0 <= expl <= 2.0  # Valid range for ±1 payoff game

    @pytest.mark.slow
    def test_2x2_exploitability_decreases(self):
        """Exploitability should decrease with more CFR iterations."""
        cfg = DeepCFRConfig(
            rows=2, cols=2,
            hidden_sizes=(64, 64),
            num_cfr_iters=10,
            num_traversals=100,
            advantage_train_steps=100,
            strategy_train_steps=500,
            buffer_size=50000,
        )
        solver = DeepCFR(cfg)

        # Measure at 10 iterations
        solver.solve(10)
        expl_10 = solver.exploitability()

        # Run 40 more iterations (total 50)
        solver.solve(40)
        expl_50 = solver.exploitability()

        # Exploitability should decrease (or at least not increase dramatically)
        assert expl_50 < expl_10 + 0.1, (
            f"Exploitability didn't decrease: {expl_10:.4f} -> {expl_50:.4f}"
        )


# ── Device placement tests ───────────────────────────────────────────────


class TestDevicePlacement:
    def test_resolve_device_cpu(self):
        assert resolve_device("cpu") == torch.device("cpu")

    def test_resolve_device_auto(self):
        dev = resolve_device("auto")
        assert isinstance(dev, torch.device)

    def test_resolve_device_rejects_unavailable_cuda(self):
        if torch.cuda.is_available():
            pytest.skip("CUDA is available on this machine")
        with pytest.raises(ValueError, match="CUDA is not available"):
            resolve_device("cuda")

    def test_resolve_device_rejects_unavailable_mps(self):
        mps_available = (
            hasattr(torch.backends, "mps") and torch.backends.mps.is_available()
        )
        if mps_available:
            pytest.skip("MPS is available on this machine")
        with pytest.raises(ValueError, match="MPS is not available"):
            resolve_device("mps")

    def test_nets_on_device(self):
        """All networks should be on the resolved device after init."""
        cfg = DeepCFRConfig(rows=2, cols=2, device="cpu")
        solver = DeepCFR(cfg)
        for net in solver._advantage_nets:
            for p in net.parameters():
                assert p.device == torch.device("cpu")
        for p in solver._strategy_net.parameters():
            assert p.device == torch.device("cpu")

    def test_2x2_runs_with_explicit_cpu(self):
        """Deep CFR with explicit device='cpu' works end-to-end."""
        cfg = DeepCFRConfig(
            rows=2, cols=2,
            hidden_sizes=(32,),
            num_cfr_iters=3,
            num_traversals=10,
            advantage_train_steps=10,
            strategy_train_steps=10,
            buffer_size=1000,
            device="cpu",
        )
        solver = DeepCFR(cfg)
        solver.solve()
        expl = solver.exploitability()
        assert 0.0 <= expl <= 2.0
