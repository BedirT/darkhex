"""Integration tests for the Rust game engine."""

import pytest
from darkhex._engine import CollisionInfo, CollisionRule, DarkHexState, Player


class TestPlayer:
    def test_values(self):
        assert int(Player.Black) == 0
        assert int(Player.White) == 1

    def test_opponent(self):
        assert Player.Black.opponent() == Player.White
        assert Player.White.opponent() == Player.Black

    def test_equality(self):
        assert Player.Black == Player.Black
        assert Player.Black != Player.White


class TestCollisionEnums:
    def test_collision_rule_values(self):
        assert int(CollisionRule.Classic) == 0
        assert int(CollisionRule.Abrupt) == 1

    def test_collision_info_values(self):
        assert int(CollisionInfo.Silent) == 0
        assert int(CollisionInfo.Noisy) == 1
        assert int(CollisionInfo.Flash) == 2


class TestCDH:
    """Classic Dark Hex: player retries after collision, opponent unaware."""

    def test_new_state_defaults_to_cdh(self):
        s = DarkHexState(2, 2)
        assert s.collision_rule() == CollisionRule.Classic
        assert s.collision_info() == CollisionInfo.Silent

    def test_new_state(self):
        s = DarkHexState(2, 2)
        assert s.rows == 2
        assert s.cols == 2
        assert s.current_player() == Player.Black
        assert not s.is_terminal()
        assert s.legal_actions() == [0, 1, 2, 3]

    def test_invalid_dimensions(self):
        with pytest.raises(ValueError):
            DarkHexState(0, 2)

    def test_black_wins_2x2(self):
        s = DarkHexState(2, 2)
        s.apply_action(0)  # Black (0,0)
        s.apply_action(1)  # White (0,1)
        s.apply_action(2)  # Black (1,0) -> N-S win
        assert s.is_terminal()
        assert s.winner() == Player.Black
        assert s.returns() == [1.0, -1.0]

    def test_white_wins_2x2(self):
        s = DarkHexState(2, 2)
        s.apply_action(0)  # Black (0,0)
        s.apply_action(2)  # White (1,0)
        s.apply_action(1)  # Black (0,1)
        s.apply_action(3)  # White (1,1) -> W-E win
        assert s.is_terminal()
        assert s.winner() == Player.White

    def test_collision_returns_false(self):
        """apply_action returns True on success, False on collision."""
        s = DarkHexState(2, 2)
        assert s.apply_action(0) is True  # Black places
        assert s.apply_action(0) is False  # White collides

    def test_collision_player_retries(self):
        """CDH: after collision, same player remains current."""
        s = DarkHexState(2, 2)
        s.apply_action(0)  # Black places at 0
        s.apply_action(0)  # White collides -> stays White's turn
        assert s.current_player() == Player.White
        assert s.num_stones() == [1, 0]
        s.apply_action(1)  # White retries on 1 -> success
        assert s.current_player() == Player.Black
        assert s.num_stones() == [1, 1]

    def test_collision_reveals_in_info_state(self):
        s = DarkHexState(2, 2)
        s.apply_action(0)  # Black at (0,0)
        s.apply_action(0)  # White collides -> discovers Black
        assert s.info_state_string(Player.White) == "P1\nx.\n.."
        assert s.num_legal_actions() == 3  # White sees 3 empty

    def test_info_state_hides_opponent(self):
        s = DarkHexState(2, 2)
        s.apply_action(0)  # Black at (0,0)
        s.apply_action(3)  # White at (1,1)
        assert s.info_state_string(Player.Black) == "P0\nx.\n.."
        assert s.info_state_string(Player.White) == "P1\n..\n.o"

    def test_multiple_collisions_then_success(self):
        """White collides twice then places successfully."""
        s = DarkHexState(3, 3)
        s.apply_action(0)  # Black places 0
        s.apply_action(4)  # White places 4
        s.apply_action(1)  # Black places 1
        # White's turn: collide on 0, collide on 1, then place on 2
        assert not s.apply_action(0)  # collision
        assert s.current_player() == Player.White
        assert not s.apply_action(1)  # collision
        assert s.current_player() == Player.White
        assert s.apply_action(2)  # success
        assert s.current_player() == Player.Black

    def test_copy_is_independent(self):
        s = DarkHexState(2, 2)
        s.apply_action(0)
        s2 = s.copy()
        s2.apply_action(1)
        assert s.stones_placed() == 1
        assert s2.stones_placed() == 2

    def test_terminal_rejects_action(self):
        s = DarkHexState(2, 2)
        s.apply_action(0)
        s.apply_action(1)
        s.apply_action(2)  # Black wins
        with pytest.raises(ValueError):
            s.apply_action(3)

    def test_out_of_bounds_action(self):
        s = DarkHexState(2, 2)
        with pytest.raises(ValueError):
            s.apply_action(4)

    def test_perfect_recall_info_state(self):
        s = DarkHexState(2, 2)
        s.apply_action(0)
        s.apply_action(3)
        info = s.info_state_string_perfect_recall(Player.Black)
        assert info.startswith("P0\nx.\n..\n")
        assert "0,0" in info

    def test_3x3_game(self):
        s = DarkHexState(3, 3)
        assert len(s.legal_actions()) == 9
        for a in [0, 1, 3, 4, 6]:
            if s.is_terminal():
                break
            s.apply_action(a)
        assert s.is_terminal()
        assert s.winner() == Player.Black


class TestADH:
    """Abrupt Dark Hex: collision wastes turn, play passes to opponent."""

    def test_adh_collision_wastes_turn(self):
        s = DarkHexState(2, 2, CollisionRule.Abrupt)
        s.apply_action(0)  # Black places
        s.apply_action(0)  # White collides -> turn wasted
        assert s.current_player() == Player.Black  # turn passed
        assert s.num_stones() == [1, 0]

    def test_adh_no_retry(self):
        s = DarkHexState(2, 2, CollisionRule.Abrupt)
        s.apply_action(0)  # Black
        s.apply_action(0)  # White collides -> Black's turn
        assert s.current_player() == Player.Black
        s.apply_action(2)  # Black places 2 -> N-S win
        assert s.is_terminal()
        assert s.winner() == Player.Black


class TestVariantConfig:
    """Test creating states with different variant configurations."""

    def test_cdh_explicit(self):
        s = DarkHexState(2, 2, CollisionRule.Classic, CollisionInfo.Silent)
        assert s.collision_rule() == CollisionRule.Classic
        assert s.collision_info() == CollisionInfo.Silent

    def test_ndh(self):
        s = DarkHexState(2, 2, CollisionRule.Classic, CollisionInfo.Noisy)
        assert s.collision_info() == CollisionInfo.Noisy

    def test_fdh(self):
        s = DarkHexState(2, 2, CollisionRule.Abrupt, CollisionInfo.Flash)
        assert s.collision_rule() == CollisionRule.Abrupt
        assert s.collision_info() == CollisionInfo.Flash
