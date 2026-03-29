use super::*;

#[test]
fn enumerate_2x2() {
    let stats = enumerate_game_tree(2, 2);
    assert_eq!(stats.total_info_states, 42, "2x2 CDH should have exactly 42 IR info states");
    assert!(stats.terminal_states > 0);
    assert!(stats.max_depth >= 4);
}

#[test]
fn enumerate_3x2() {
    let stats = enumerate_game_tree(3, 2);
    assert_eq!(stats.total_info_states, 410, "3x2 CDH should have exactly 410 IR info states");
}

#[test]
fn enumerate_2x2_player_split() {
    let stats = enumerate_game_tree(2, 2);
    // Both players should have info states
    assert!(stats.info_states_by_player[0] > 0);
    assert!(stats.info_states_by_player[1] > 0);
    assert_eq!(
        stats.info_states_by_player[0] + stats.info_states_by_player[1],
        stats.total_info_states
    );
}
