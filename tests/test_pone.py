"""Integration tests for pONE (probability-one win states)."""

from darkhex._engine import (
    MCCFRSolver,
    PoneDb,
    Sampling,
    exploitability,
)


class TestPoneDb:
    def test_2x2_builds(self):
        db = PoneDb(2, 2)
        assert db.len() > 0

    def test_3x2_builds(self):
        db = PoneDb(3, 2)
        assert db.len() > 0

    def test_repr(self):
        db = PoneDb(2, 2)
        r = repr(db)
        assert "PoneDb" in r
        assert "2x2" in r
        assert "pONE" in r


class TestPoneIntegration:
    def test_mccfr_with_pone(self):
        """MCCFR with pONE should produce valid strategies on 3x2."""
        db = PoneDb(3, 2)
        solver = MCCFRSolver(3, 2, Sampling.Outcome, seed=42)
        solver.set_pone_db(db)
        solver.solve(5000)
        strategy = solver.get_average_strategy()
        assert len(strategy) > 0
        for key, probs in strategy.items():
            total = sum(p for _, p in probs)
            assert abs(total - 1.0) < 0.05, f"{key}: sum={total}"

    def test_mccfr_pone_reduces_info_states(self):
        """pONE should reduce info state count (pruned subtrees)."""
        solver_without = MCCFRSolver(2, 2, Sampling.Outcome, seed=42)
        solver_without.solve(5000)

        db = PoneDb(2, 2)
        solver_with = MCCFRSolver(2, 2, Sampling.Outcome, seed=42)
        solver_with.set_pone_db(db)
        solver_with.solve(5000)

        assert solver_with.num_info_states() <= solver_without.num_info_states(), (
            f"pONE should not increase info states: "
            f"{solver_with.num_info_states()} vs {solver_without.num_info_states()}"
        )

    def test_exploitability_with_pone(self):
        """exploitability() accepts optional pone_db."""
        solver = MCCFRSolver(2, 2, Sampling.Outcome, seed=42)
        solver.solve(5000)
        strategy = solver.get_average_strategy()

        db = PoneDb(2, 2)
        expl = exploitability(2, 2, strategy, db)
        assert expl >= 0.0
        assert expl <= 1.0 + 1e-6

    def test_exploitability_without_pone_backward_compat(self):
        """exploitability() still works without pone_db."""
        solver = MCCFRSolver(2, 2, Sampling.Outcome, seed=42)
        solver.solve(5000)
        strategy = solver.get_average_strategy()
        expl = exploitability(2, 2, strategy)
        assert expl >= 0.0
