use super::*;
use crate::game::types::{CollisionRule, Player};

fn cdh(rows: usize, cols: usize) -> DarkHexState {
    DarkHexState::new(rows, cols, None, None).unwrap()
}

fn adh(rows: usize, cols: usize) -> DarkHexState {
    DarkHexState::new(rows, cols, Some(CollisionRule::Abrupt), None).unwrap()
}

// --- CDH (Classic) tests ---

#[test]
fn cdh_initial_state() {
    let s = cdh(2, 2);
    assert_eq!(s.current_player(), Player::Black);
    assert!(!s.is_terminal());
    assert_eq!(s.legal_actions().len(), 4);
}

#[test]
fn cdh_black_wins_2x2() {
    let mut s = cdh(2, 2);
    s.apply_action(0).unwrap(); // Black at (0,0) → success, White's turn
    s.apply_action(1).unwrap(); // White at (0,1) → success, Black's turn
    s.apply_action(2).unwrap(); // Black at (1,0) → N-S win
    assert!(s.is_terminal());
    assert_eq!(s.winner(), Some(Player::Black));
    assert_eq!(s.returns(), [1.0, -1.0]);
}

#[test]
fn cdh_collision_retries() {
    let mut s = cdh(2, 2);
    assert!(s.apply_action(0).unwrap()); // Black places at 0
    assert!(!s.apply_action(0).unwrap()); // collision, returns false
    assert_eq!(s.current_player(), Player::White);
    assert_eq!(s.num_stones(), [1, 0]);
    assert!(s.apply_action(1).unwrap());
    assert_eq!(s.current_player(), Player::Black);
    assert_eq!(s.num_stones(), [1, 1]);
}

#[test]
fn cdh_collision_reveals_in_view() {
    let mut s = cdh(2, 2);
    s.apply_action(0).unwrap();
    s.apply_action(0).unwrap();
    let info_w = s.info_state_string(Player::White);
    assert_eq!(info_w, "P1\nx.\n..");
    assert_eq!(s.num_legal_actions(), 3);
}

#[test]
fn cdh_info_state_hides_opponent() {
    let mut s = cdh(2, 2);
    s.apply_action(0).unwrap();
    s.apply_action(3).unwrap();
    assert_eq!(s.info_state_string(Player::Black), "P0\nx.\n..");
    assert_eq!(s.info_state_string(Player::White), "P1\n..\n.o");
}

#[test]
fn cdh_multiple_collisions_then_success() {
    let mut s = cdh(2, 2);
    s.apply_action(0).unwrap();
    s.apply_action(0).unwrap();
    assert_eq!(s.current_player(), Player::White);
    s.apply_action(1).unwrap();
    assert_eq!(s.current_player(), Player::Black);
    s.apply_action(2).unwrap();
    assert!(s.is_terminal());
    assert_eq!(s.winner(), Some(Player::Black));
}

// --- ADH (Abrupt) tests ---

#[test]
fn adh_collision_wastes_turn() {
    let mut s = adh(2, 2);
    s.apply_action(0).unwrap();
    assert!(!s.apply_action(0).unwrap());
    assert_eq!(s.current_player(), Player::Black);
    assert_eq!(s.num_stones(), [1, 0]);
}

// --- Common tests ---

#[test]
fn terminal_rejects_action() {
    let mut s = cdh(2, 2);
    s.apply_action(0).unwrap();
    s.apply_action(1).unwrap();
    s.apply_action(2).unwrap();
    assert!(s.apply_action(3).is_err());
}

#[test]
fn copy_is_independent() {
    let mut s = cdh(2, 2);
    s.apply_action(0).unwrap();
    let mut s2 = s.copy();
    s2.apply_action(1).unwrap();
    assert_eq!(s.stones_placed(), 1);
    assert_eq!(s2.stones_placed(), 2);
}

#[test]
fn perfect_recall_info_state() {
    let mut s = cdh(2, 2);
    s.apply_action(0).unwrap();
    s.apply_action(3).unwrap();
    let info = s.info_state_string_perfect_recall(Player::Black);
    assert!(info.starts_with("P0\nx.\n..\n"));
    assert!(info.contains("0,0"));
}

#[test]
fn three_by_three_black_wins() {
    let mut s = cdh(3, 3);
    assert_eq!(s.legal_actions().len(), 9);
    for a in [0, 1, 3, 4, 6] {
        if s.is_terminal() {
            break;
        }
        s.apply_action(a).unwrap();
    }
    assert!(s.is_terminal());
    assert_eq!(s.winner(), Some(Player::Black));
}

// --- Canonical info state (180° rotation symmetry) tests ---

#[test]
fn canonical_info_state_symmetry_2x2() {
    let mut s1 = DarkHexState::new_cdh(2, 2);
    s1.apply_action_unchecked(0);
    let (c1, _) = s1.canonical_info_state(Player::Black);

    let mut s2 = DarkHexState::new_cdh(2, 2);
    s2.apply_action_unchecked(3);
    let (c2, _) = s2.canonical_info_state(Player::Black);

    assert_eq!(c1, c2, "symmetric states should have same canonical form");
}

#[test]
fn canonical_empty_board_is_self() {
    let s = DarkHexState::new_cdh(2, 2);
    let orig = s.info_state_string(Player::Black);
    let (canon, is_orig) = s.canonical_info_state(Player::Black);
    assert_eq!(orig, canon);
    assert!(is_orig);
}

#[test]
fn canonical_preserves_player_prefix() {
    let s = DarkHexState::new_cdh(2, 2);
    let (c0, _) = s.canonical_info_state(Player::Black);
    let (c1, _) = s.canonical_info_state(Player::White);
    assert!(c0.starts_with("P0\n"));
    assert!(c1.starts_with("P1\n"));
}

#[test]
fn canonical_3x2_symmetry() {
    let mut s1 = DarkHexState::new_cdh(3, 2);
    s1.apply_action_unchecked(0);
    let (c1, _) = s1.canonical_info_state(Player::Black);

    let mut s2 = DarkHexState::new_cdh(3, 2);
    s2.apply_action_unchecked(5);
    let (c2, _) = s2.canonical_info_state(Player::Black);

    assert_eq!(c1, c2);
}

#[test]
fn canonical_non_symmetric_state_chooses_smaller() {
    let mut s = DarkHexState::new_cdh(2, 2);
    s.apply_action_unchecked(0);
    let orig = s.info_state_string(Player::Black);
    let (canon, is_orig) = s.canonical_info_state(Player::Black);
    assert!(!is_orig, "rotated should be chosen as canonical");
    assert!(canon < orig, "canonical should be lex-smaller");
}

#[test]
fn rotated_info_state_reverses_grid() {
    let mut s = DarkHexState::new_cdh(2, 2);
    s.apply_action_unchecked(0);
    let orig = s.info_state_string(Player::Black);
    assert_eq!(orig, "P0\nx.\n..");
    let rotated = s.rotated_info_state_string(Player::Black);
    assert_eq!(rotated, "P0\n..\n.x");
}
