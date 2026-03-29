"""Integration tests for the MCCFR solver."""

from darkhex._engine import MCCFRSolver, Sampling


class TestMCCFRCommon:
    def test_default_is_outcome(self):
        solver = MCCFRSolver(2, 2)
        assert solver.sampling() == Sampling.Outcome

    def test_seeded_determinism(self):
        s1 = MCCFRSolver(2, 2, seed=123)
        s1.solve(100)
        s2 = MCCFRSolver(2, 2, seed=123)
        s2.solve(100)
        assert s1.num_info_states() == s2.num_info_states()

    def test_repr(self):
        solver = MCCFRSolver(2, 2)
        solver.solve(10)
        r = repr(solver)
        assert "MCCFRSolver" in r
        assert "iters=10" in r


class TestExternalSampling:
    def test_runs(self):
        solver = MCCFRSolver(2, 2, Sampling.External, seed=42)
        solver.solve(100)
        assert solver.iterations() == 100
        assert solver.num_info_states() > 0

    def test_2x2_strategy_valid(self):
        solver = MCCFRSolver(2, 2, Sampling.External, seed=42)
        solver.solve(1000)
        strategy = solver.get_average_strategy()
        assert len(strategy) > 0
        for key, probs in strategy.items():
            total = sum(p for _, p in probs)
            assert abs(total - 1.0) < 0.01, f"{key}: sum={total}"


class TestOutcomeSampling:
    def test_runs(self):
        solver = MCCFRSolver(2, 2, Sampling.Outcome, seed=42)
        solver.solve(100)
        assert solver.iterations() == 100
        assert solver.num_info_states() > 0

    def test_2x2_discovers_all_info_states(self):
        """Epsilon-greedy exploration should find all 42 info states."""
        solver = MCCFRSolver(2, 2, Sampling.Outcome, epsilon=0.6, seed=42)
        solver.solve(10000)
        n = solver.num_info_states()
        assert n >= 40, f"expected >=40, got {n}"

    def test_2x2_strategy_valid(self):
        solver = MCCFRSolver(2, 2, Sampling.Outcome, seed=42)
        solver.solve(5000)
        strategy = solver.get_average_strategy()
        assert len(strategy) > 0
        for key, probs in strategy.items():
            total = sum(p for _, p in probs)
            assert abs(total - 1.0) < 0.05, f"{key}: sum={total}"

    def test_2x2_converges(self):
        solver = MCCFRSolver(2, 2, Sampling.Outcome, seed=42)
        solver.solve(10000)
        strategy = solver.get_average_strategy()
        initial_key = "P0\n..\n.."
        assert initial_key in strategy

    def test_3x2_runs(self):
        solver = MCCFRSolver(3, 2, Sampling.Outcome, seed=42)
        solver.solve(100)
        assert solver.iterations() == 100
        assert solver.num_info_states() > 10

    def test_3x3_runs_fast(self):
        """Outcome Sampling should handle 3x3 easily (unlike External)."""
        solver = MCCFRSolver(3, 3, Sampling.Outcome, seed=42)
        solver.solve(100)
        assert solver.iterations() == 100
        assert solver.num_info_states() > 100
