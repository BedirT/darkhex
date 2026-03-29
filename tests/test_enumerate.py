"""Tests for exhaustive game tree enumeration — ground truth info state counts."""

from darkhex._engine import MCCFRSolver, Sampling, enumerate_game_tree


class TestEnumerateGameTree:
    def test_2x2_exact_count(self):
        stats = enumerate_game_tree(2, 2)
        assert stats.total_info_states == 42

    def test_3x2_exact_count(self):
        stats = enumerate_game_tree(3, 2)
        assert stats.total_info_states == 410

    def test_player_split_sums(self):
        stats = enumerate_game_tree(2, 2)
        p0, p1 = stats.info_states_by_player
        assert p0 + p1 == stats.total_info_states
        assert p0 > 0
        assert p1 > 0

    def test_has_terminals(self):
        stats = enumerate_game_tree(2, 2)
        assert stats.terminal_states > 0

    def test_repr(self):
        stats = enumerate_game_tree(2, 2)
        r = repr(stats)
        assert "info_states=42" in r


class TestMCCFRCoverage:
    """Verify MCCFR discovers info states.

    OS-MCCFR applies epsilon exploration only at update player nodes
    (per OpenSpiel), so opponent zero-probability branches are not
    explored. This is correct — those states don't affect the Nash
    equilibrium. Coverage is high but not 100%.
    """

    def test_outcome_2x2_full_coverage(self):
        """2x2 is small enough for full coverage."""
        expected = enumerate_game_tree(2, 2).total_info_states
        solver = MCCFRSolver(2, 2, Sampling.Outcome, epsilon=0.6, seed=42)
        solver.solve(10000)
        assert solver.num_info_states() == expected

    def test_outcome_3x2_high_coverage(self):
        """3x2: expect >85% coverage (some opponent zero-prob paths missed)."""
        expected = enumerate_game_tree(3, 2).total_info_states
        solver = MCCFRSolver(3, 2, Sampling.Outcome, epsilon=0.6, seed=42)
        solver.solve(20000)
        coverage = solver.num_info_states() / expected
        assert coverage > 0.85, f"coverage {coverage:.1%} below 85%"

    def test_external_2x2_partial_coverage(self):
        """External misses more states (zero-prob at opponent nodes)."""
        expected = enumerate_game_tree(2, 2).total_info_states
        solver = MCCFRSolver(2, 2, Sampling.External, seed=42)
        solver.solve(10000)
        assert solver.num_info_states() < expected  # known gap
