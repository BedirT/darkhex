use crate::game::state::DarkHexState;
use crate::game::types::Player;

use super::*;

#[test]
fn pone_2x2_builds() {
    let db = PoneDb::new(2, 2);
    assert!(db.len() > 0, "should find some pONE states on 2x2");
}

#[test]
fn pone_3x2_builds() {
    let db = PoneDb::new(3, 2);
    assert!(db.len() > 0, "should find some pONE states on 3x2");
}

#[test]
fn pone_contains_known_state() {
    let db = PoneDb::new(2, 2);
    assert!(db.len() > 0);
}

#[test]
fn pone_h0_sure_win() {
    let mut state = DarkHexState::new_cdh(2, 2);
    state.apply_action_unchecked(0); // Black places at 0
    state.apply_action_unchecked(3); // White places at 3
    state.apply_action_unchecked(3); // Black tries 3, collision in CDH -> retry
    let db = PoneDb::new(2, 2);
    let (canon, _) = state.canonical_info_state(Player::Black);
    assert!(
        db.contains_canonical(&canon),
        "h=0 sure-win state should be pONE: {canon}"
    );
}

#[test]
fn pone_root_not_pone_for_white() {
    let db = PoneDb::new(2, 2);
    let state = DarkHexState::new_cdh(2, 2);
    let (canon_w, _) = state.canonical_info_state(Player::White);
    assert!(
        !db.contains_canonical(&canon_w),
        "White's root info state should not be pONE (White doesn't move at root)"
    );
}

#[test]
fn pone_root_is_pone_for_black_2x2() {
    let db = PoneDb::new(2, 2);
    let state = DarkHexState::new_cdh(2, 2);
    let (canon, _) = state.canonical_info_state(Player::Black);
    assert!(
        db.contains_canonical(&canon),
        "Black root on 2x2 should be pONE (Black wins 2x2 Hex)"
    );
}

#[test]
fn pone_4x3_root_not_pone_for_white() {
    let db = PoneDb::new(4, 3);
    let state = DarkHexState::new_cdh(4, 3);
    let (canon_w, _) = state.canonical_info_state(Player::White);
    assert!(
        !db.contains_canonical(&canon_w),
        "P1 root on 4x3 must NOT be pONE (AND-OR fix)"
    );
}
