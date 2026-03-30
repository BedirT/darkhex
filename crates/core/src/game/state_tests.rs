use super::*;

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
    s.apply_action(0).unwrap(); // Black at (0,0) -> success, White's turn
    s.apply_action(1).unwrap(); // White at (0,1) -> success, Black's turn
    s.apply_action(2).unwrap(); // Black at (1,0) -> N-S win
    assert!(s.is_terminal());
    assert_eq!(s.winner(), Some(Player::Black));
    assert_eq!(s.returns(), [1.0, -1.0]);
}

#[test]
fn cdh_collision_retries() {
    // CDH: after collision, SAME player tries again
    let mut s = cdh(2, 2);
    assert!(s.apply_action(0).unwrap()); // Black places at 0
                                         // White's turn: tries cell 0 -> collision
    assert!(!s.apply_action(0).unwrap()); // collision, returns false
                                          // White should STILL be the current player (CDH retry)
    assert_eq!(s.current_player(), Player::White);
    assert_eq!(s.num_stones(), [1, 0]);
    // White retries on cell 1 -> success
    assert!(s.apply_action(1).unwrap());
    // Now Black's turn
    assert_eq!(s.current_player(), Player::Black);
    assert_eq!(s.num_stones(), [1, 1]);
}

#[test]
fn cdh_collision_reveals_in_view() {
    let mut s = cdh(2, 2);
    s.apply_action(0).unwrap(); // Black at (0,0)
    s.apply_action(0).unwrap(); // White collides at (0,0) -> discovers Black
                                // White sees Black at cell 0
    let info_w = s.info_state_string(Player::White);
    assert_eq!(info_w, "P1\nx.\n..");
    // White still has 3 empty-looking cells (cell 0 now revealed)
    assert_eq!(s.num_legal_actions(), 3);
}

#[test]
fn cdh_info_state_hides_opponent() {
    let mut s = cdh(2, 2);
    s.apply_action(0).unwrap(); // Black at (0,0)
    s.apply_action(3).unwrap(); // White at (1,1)
    assert_eq!(s.info_state_string(Player::Black), "P0\nx.\n..");
    assert_eq!(s.info_state_string(Player::White), "P1\n..\n.o");
}

#[test]
fn cdh_multiple_collisions_then_success() {
    // Black places at 0 and 1, White collides on both then succeeds on 2
    let mut s = cdh(2, 2);
    s.apply_action(0).unwrap(); // Black places 0
    s.apply_action(0).unwrap(); // White collides 0 -> stays White
    assert_eq!(s.current_player(), Player::White);
    // White needs to place somewhere. Black hasn't placed at 1 yet so...
    // Actually Black only placed 0. White collided 0. White tries 1.
    s.apply_action(1).unwrap(); // White places 1 -> success
    assert_eq!(s.current_player(), Player::Black);
    s.apply_action(2).unwrap(); // Black places 2 -> N-S win (0 and 2)
    assert!(s.is_terminal());
    assert_eq!(s.winner(), Some(Player::Black));
}

// --- ADH (Abrupt) tests ---

#[test]
fn adh_collision_wastes_turn() {
    let mut s = adh(2, 2);
    s.apply_action(0).unwrap(); // Black places at 0
                                // White collides on 0 -> turn wasted
    assert!(!s.apply_action(0).unwrap());
    // Turn passed to Black (ADH)
    assert_eq!(s.current_player(), Player::Black);
    assert_eq!(s.num_stones(), [1, 0]);
}

// --- Common tests ---

#[test]
fn terminal_rejects_action() {
    let mut s = cdh(2, 2);
    s.apply_action(0).unwrap();
    s.apply_action(1).unwrap();
    s.apply_action(2).unwrap(); // Black wins
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
    // Black plays column 0: cells 0, 3, 6
    for a in [0, 1, 3, 4, 6] {
        if s.is_terminal() {
            break;
        }
        s.apply_action(a).unwrap();
    }
    assert!(s.is_terminal());
    assert_eq!(s.winner(), Some(Player::Black));
}

// --- Canonical info state (180-degree rotation symmetry) tests ---

#[test]
fn canonical_info_state_symmetry_2x2() {
    // 2x2 board: 180-degree rotation maps cell 0<->3, 1<->2
    // Black at cell 0 -> "P0\nx.\n.." and Black at cell 3 -> "P0\n..\n.x"
    // These should have the same canonical form.
    let mut s1 = DarkHexState::rs_new(2, 2);
    s1.rs_apply_action(0); // Black at 0
    let (c1, _) = s1.rs_canonical_info_state(Player::Black);

    let mut s2 = DarkHexState::rs_new(2, 2);
    s2.rs_apply_action(3); // Black at 3
    let (c2, _) = s2.rs_canonical_info_state(Player::Black);

    assert_eq!(c1, c2, "symmetric states should have same canonical form");
}

#[test]
fn canonical_empty_board_is_self() {
    // Empty board is self-canonical (all dots, palindromic)
    let s = DarkHexState::rs_new(2, 2);
    let orig = s.rs_info_state_string(Player::Black);
    let (canon, is_orig) = s.rs_canonical_info_state(Player::Black);
    assert_eq!(orig, canon);
    assert!(is_orig);
}

#[test]
fn canonical_preserves_player_prefix() {
    let s = DarkHexState::rs_new(2, 2);
    let (c0, _) = s.rs_canonical_info_state(Player::Black);
    let (c1, _) = s.rs_canonical_info_state(Player::White);
    assert!(c0.starts_with("P0\n"));
    assert!(c1.starts_with("P1\n"));
}

#[test]
fn canonical_3x2_symmetry() {
    // 3x2 board (6 cells): 180-degree maps 0<->5, 1<->4, 2<->3
    // Black at cell 0 and Black at cell 5 should share canonical form.
    let mut s1 = DarkHexState::rs_new(3, 2);
    s1.rs_apply_action(0);
    let (c1, _) = s1.rs_canonical_info_state(Player::Black);

    let mut s2 = DarkHexState::rs_new(3, 2);
    s2.rs_apply_action(5);
    let (c2, _) = s2.rs_canonical_info_state(Player::Black);

    assert_eq!(c1, c2);
}

#[test]
fn canonical_non_symmetric_state_chooses_smaller() {
    // Create a state that is NOT self-symmetric, verify canonical is the smaller.
    let mut s = DarkHexState::rs_new(2, 2);
    s.rs_apply_action(0); // Black at 0
    let orig = s.rs_info_state_string(Player::Black);
    let (canon, is_orig) = s.rs_canonical_info_state(Player::Black);
    // "P0\nx.\n.." vs rotated "P0\n..\n.x" -- ".." < "x." so rotated is smaller
    assert!(!is_orig, "rotated should be chosen as canonical");
    assert!(canon < orig, "canonical should be lex-smaller");
}

#[test]
fn rotated_info_state_reverses_grid() {
    // Verify the rotation produces the expected string.
    let mut s = DarkHexState::rs_new(2, 2);
    s.rs_apply_action(0); // Black at cell 0
                          // Original: "P0\nx.\n.."  grid cells: [x, ., ., .]
                          // Rotated:  "P0\n..\n.x"  grid cells reversed: [., ., ., x]
    let orig = s.rs_info_state_string(Player::Black);
    assert_eq!(orig, "P0\nx.\n..");
    let rotated = s.rs_rotated_info_state_string(Player::Black);
    assert_eq!(rotated, "P0\n..\n.x");
}
