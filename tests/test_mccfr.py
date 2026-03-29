"""Integration tests for the MCCFR solver."""

from darkhex._engine import MCCFRSolver


class TestMCCFRSolver:
    def test_create(self):
        solver = MCCFRSolver(2, 2)
        assert solver.iterations() == 0
        assert solver.num_info_states() == 0

    def test_seeded_determinism(self):
        """Same seed produces same results."""
        s1 = MCCFRSolver(2, 2, seed=123)
        s1.solve(100)
        s2 = MCCFRSolver(2, 2, seed=123)
        s2.solve(100)
        assert s1.num_info_states() == s2.num_info_states()
        strat1 = s1.get_average_strategy()
        strat2 = s2.get_average_strategy()
        assert set(strat1.keys()) == set(strat2.keys())

    def test_solve_100_iterations(self):
        solver = MCCFRSolver(2, 2, seed=42)
        solver.solve(100)
        assert solver.iterations() == 100
        assert solver.num_info_states() > 0

    def test_average_strategy_valid(self):
        solver = MCCFRSolver(2, 2, seed=42)
        solver.solve(1000)
        strategy = solver.get_average_strategy()
        assert len(strategy) > 0
        for key, probs in strategy.items():
            total = sum(p for _, p in probs)
            assert abs(total - 1.0) < 0.01, f"{key}: sum={total}"
            for _action, prob in probs:
                assert prob >= 0.0, f"{key}: negative prob {prob}"

    def test_2x2_converges(self):
        """After many iterations, strategy should stabilize."""
        solver = MCCFRSolver(2, 2, seed=42)
        solver.solve(10000)
        strategy = solver.get_average_strategy()
        # Black's initial state should have a strategy over all 4 cells
        initial_key = "P0\n..\n.."
        keys = list(strategy.keys())[:5]
        assert initial_key in strategy, f"Missing initial state. Keys: {keys}"
        probs = dict(strategy[initial_key])
        # All 4 actions should have some probability
        assert len(probs) >= 2, f"Expected >=2 actions, got {probs}"

    def test_3x2_runs(self):
        """3x2 board should complete without errors."""
        solver = MCCFRSolver(3, 2, seed=42)
        solver.solve(100)
        assert solver.iterations() == 100
        assert solver.num_info_states() > 10

    def test_repr(self):
        solver = MCCFRSolver(2, 2)
        solver.solve(10)
        r = repr(solver)
        assert "MCCFRSolver" in r
        assert "iters=10" in r
