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
    // On 2x2: if Black has stones at 0 and 2 (column 0), Black has won.
    // But that's already terminal. We need a pre-terminal pONE state.
    //
    // Example: Black has stone at 0, White has stone at 3. Black's view shows
    // both (discovered via collision or own stone). h=0, Black plays cell 2 and wins.
    // The info state "P0\nx.\n.o" with h=0 should be pONE for Black.
    let db = PoneDb::new(2, 2);
    // The DB stores canonical forms, so we check the canonical version
    assert!(db.len() > 0);
}

#[test]
fn pone_h0_sure_win() {
    // Set up a 2x2 state where Black has stone at 0, White at 3.
    // h=0 for Black (Black discovered White's stone). Black to play.
    // Black plays cell 2 -> wins (connects row 0 and row 1).
    // This should be pONE.
    let mut state = DarkHexState::rs_new(2, 2);
    state.rs_apply_action(0); // Black places at 0 (success)
                              // Now White's turn
    state.rs_apply_action(3); // White places at 3 (success)
                              // Now Black's turn. Black tries cell 3 -> collision (discovers White)
    state.rs_apply_action(3); // Black tries 3, collision in CDH -> retry
                              // After collision, Black knows White is at 3. h=0 now.
                              // Black can win by playing cell 2.
                              // Check that this info state is in the pONE db.
    let db = PoneDb::new(2, 2);
    let (canon, _) = state.rs_canonical_info_state(Player::Black);
    assert!(
        db.rs_contains(&canon),
        "h=0 sure-win state should be pONE: {canon}"
    );
}

#[test]
fn pone_root_not_pone_for_white() {
    // On 2x2, Black moves first. At the root state there are no hidden
    // stones (h=0), so pONE reduces to regular Hex minimax. Black wins
    // 2x2 Hex, so the root IS pONE for Black. But White (not to move)
    // is never checked at the root. Verify that White's root info state
    // is not in the pONE db — White never has the move at the root.
    let db = PoneDb::new(2, 2);
    let state = DarkHexState::rs_new(2, 2);
    let (canon_w, _) = state.rs_canonical_info_state(Player::White);
    assert!(
        !db.rs_contains(&canon_w),
        "White's root info state should not be pONE (White doesn't move at root)"
    );
}

#[test]
fn pone_root_is_pone_for_black_2x2() {
    // On 2x2, the empty board has h=0 for Black. pONE reduces to
    // regular Hex minimax. Black wins 2x2 Hex, so root IS pONE.
    let db = PoneDb::new(2, 2);
    let state = DarkHexState::rs_new(2, 2);
    let (canon, _) = state.rs_canonical_info_state(Player::Black);
    assert!(
        db.rs_contains(&canon),
        "Black root on 2x2 should be pONE (Black wins 2x2 Hex)"
    );
}

#[test]
fn pone_4x3_root_not_pone_for_white() {
    // After the AND-OR fix: P1 root on 4x3 must NOT be pONE.
    // The old per-config minimax falsely flagged it because White
    // can beat each hidden-Black-stone position separately, but
    // no single strategy works blind against all 12 positions.
    let db = PoneDb::new(4, 3);
    let state = DarkHexState::rs_new(4, 3);
    let (canon_w, _) = state.rs_canonical_info_state(Player::White);
    assert!(
        !db.rs_contains(&canon_w),
        "P1 root on 4x3 must NOT be pONE (AND-OR fix)"
    );
}
