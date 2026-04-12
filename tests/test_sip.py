"""Integration tests for SIP/SIP+ policy simplification."""

import pytest
from darkhex._engine import (
    MCCFRSolver,
    Sampling,
    exploitability,
    simplify_policy,
    simplify_policy_plus,
)


class TestSimplifyPolicyBasic:
    """Test SIP on hand-crafted strategies."""

    def test_empty_strategy(self):
        result = simplify_policy({}, 0.1, 2)
        assert result == {}

    def test_single_action_info_state(self):
        strategy = {"P0\n..\n..": [(0, 1.0)]}
        result = simplify_policy(strategy, 0.1, 2)
        assert result["P0\n..\n.."] == [(0, 1.0)]

    def test_filter_and_cap(self):
        strategy = {"P0\n..\n..": [(0, 0.6), (1, 0.3), (2, 0.05), (3, 0.05)]}
        result = simplify_policy(strategy, 0.1, 2)
        actions = result["P0\n..\n.."]
        assert len(actions) == 2
        total = sum(p for _, p in actions)
        assert abs(total - 1.0) < 1e-5

    def test_probs_sum_to_one(self):
        strategy = {"P0\n..\n..": [(0, 0.4), (1, 0.3), (2, 0.2), (3, 0.1)]}
        result = simplify_policy(strategy, 0.05, 3)
        for actions in result.values():
            total = sum(p for _, p in actions)
            assert abs(total - 1.0) < 1e-5

    def test_action_cap_respected(self):
        strategy = {"P0\n..\n..": [(0, 0.4), (1, 0.3), (2, 0.2), (3, 0.1)]}
        result = simplify_policy(strategy, 0.0, 2)
        for actions in result.values():
            assert len(actions) <= 2

    def test_invalid_action_cap(self):
        with pytest.raises(ValueError):
            simplify_policy({}, 0.1, 0)


class TestSimplifyPolicyMCCFR:
    """Test SIP on MCCFR-generated strategies."""

    @pytest.fixture
    def trained_strategy(self):
        solver = MCCFRSolver(2, 2, Sampling.Outcome, epsilon=0.6, seed=42)
        solver.solve(10_000)
        return solver.get_average_strategy()

    def test_simplified_probs_valid(self, trained_strategy):
        result = simplify_policy(trained_strategy, 0.1, 2)
        for actions in result.values():
            total = sum(p for _, p in actions)
            assert abs(total - 1.0) < 1e-5
            assert len(actions) <= 2

    def test_composable_with_exploitability(self, trained_strategy):
        simplified = simplify_policy(trained_strategy, 0.1, 2)
        expl = exploitability(2, 2, simplified)
        assert expl >= 0.0
        assert expl < 1.0  # should be reasonable

    def test_exploitability_reasonable(self, trained_strategy):
        raw_expl = exploitability(2, 2, trained_strategy)
        simplified = simplify_policy(trained_strategy, 0.1, 2)
        sip_expl = exploitability(2, 2, simplified)
        # Both should be low for a well-trained 2x2 strategy
        assert raw_expl < 0.05
        # SIP can improve exploitability (removing MCCFR noise) or slightly
        # worsen it — either is fine. Thesis confirms SIP improved epsilon
        # from 0.009 to 0.002 on 4x3. Just verify it stays reasonable.
        assert sip_expl < 0.05

    def test_fewer_total_actions(self, trained_strategy):
        simplified = simplify_policy(trained_strategy, 0.1, 2)
        raw_total = sum(len(a) for a in trained_strategy.values())
        sip_total = sum(len(a) for a in simplified.values())
        assert sip_total <= raw_total


class TestSimplifyPolicyPlus:
    """Test SIP+ fractionization."""

    def test_fractionizes_cleanly(self):
        # Strategy that should fractionize to 2/3, 1/3
        strategy = {"P0\n..\n..": [(0, 0.6), (1, 0.3), (2, 0.05), (3, 0.05)]}
        result = simplify_policy_plus(strategy, 0.1, 2, 3, 0.01)
        actions = result["P0\n..\n.."]
        assert len(actions) == 2
        probs = sorted([p for _, p in actions], reverse=True)
        # Should be 2/3 and 1/3
        assert abs(probs[0] - 2 / 3) < 1e-5
        assert abs(probs[1] - 1 / 3) < 1e-5

    def test_invalid_frac_limit(self):
        with pytest.raises(ValueError):
            simplify_policy_plus({}, 0.1, 2, 0, 0.005)

    def test_eta_zero_equals_sip(self):
        strategy = {"P0\n..\n..": [(0, 0.6), (1, 0.3), (2, 0.05), (3, 0.05)]}
        sip = simplify_policy(strategy.copy(), 0.1, 2)
        sip_plus = simplify_policy_plus(strategy, 0.1, 2, 20, 0.0)
        # With eta=0, no fractionization possible, should match SIP
        assert sip.keys() == sip_plus.keys()
        for key in sip:
            for (a1, p1), (a2, p2) in zip(sip[key], sip_plus[key], strict=True):
                assert a1 == a2
                assert abs(p1 - p2) < 1e-6

    @pytest.fixture
    def trained_strategy(self):
        solver = MCCFRSolver(2, 2, Sampling.Outcome, epsilon=0.6, seed=42)
        solver.solve(10_000)
        return solver.get_average_strategy()

    def test_fractionization_fallback(self):
        # Strategy that won't fractionize with frac_limit=2 (only 1/2 available)
        strategy = {"P0\n..\n..": [(0, 0.7), (1, 0.3)]}
        sip = simplify_policy(strategy.copy(), 0.0, 2)
        sip_plus = simplify_policy_plus(strategy, 0.0, 2, 2, 0.01)
        # Should fall back to SIP output since 0.7 and 0.3 don't match 1/2
        for key in sip:
            for (a1, p1), (a2, p2) in zip(sip[key], sip_plus[key], strict=True):
                assert a1 == a2
                assert abs(p1 - p2) < 1e-6

    def test_pipeline_with_exploitability(self, trained_strategy):
        simplified = simplify_policy_plus(trained_strategy, 0.1, 2, 20, 0.005)
        sip_plus_expl = exploitability(2, 2, simplified)
        assert sip_plus_expl >= 0.0
        assert sip_plus_expl < 0.05
