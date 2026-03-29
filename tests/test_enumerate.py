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


class TestMCCFRMatchesEnumeration:
    """Verify MCCFR discovers all info states that enumeration finds."""

    def test_outcome_2x2_matches(self):
        expected = enumerate_game_tree(2, 2).total_info_states
        solver = MCCFRSolver(2, 2, Sampling.Outcome, epsilon=0.6, seed=42)
        solver.solve(10000)
        assert solver.num_info_states() == expected

    def test_outcome_3x2_matches(self):
        expected = enumerate_game_tree(3, 2).total_info_states
        solver = MCCFRSolver(3, 2, Sampling.Outcome, epsilon=0.6, seed=42)
        solver.solve(50000)
        assert solver.num_info_states() == expected
