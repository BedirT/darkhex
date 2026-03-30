use super::*;

#[test]
fn enumerate_2x2() {
    let stats = enumerate_game_tree(2, 2);
    assert_eq!(
        stats.total_info_states, 42,
        "2x2 CDH should have exactly 42 IR info states"
    );
    assert!(stats.game_states_visited > 0);
    assert!(stats.max_depth >= 4);
}

#[test]
fn enumerate_3x2() {
    let stats = enumerate_game_tree(3, 2);
    assert_eq!(
        stats.total_info_states, 410,
        "3x2 CDH should have exactly 410 IR info states"
    );
}

#[test]
fn enumerate_3x3() {
    // With memoization this should complete in seconds, not hours
    let stats = enumerate_game_tree(3, 3);
    assert_eq!(
        stats.total_info_states, 12556,
        "3x3 CDH should have exactly 12556 IR info states"
    );
}

#[test]
fn enumerate_2x2_player_split() {
    let stats = enumerate_game_tree(2, 2);
    assert!(stats.info_states_by_player[0] > 0);
    assert!(stats.info_states_by_player[1] > 0);
    assert_eq!(
        stats.info_states_by_player[0] + stats.info_states_by_player[1],
        stats.total_info_states
    );
}

#[test]
fn canonical_2x2_roughly_half() {
    let stats = enumerate_game_tree(2, 2);
    assert!(
        stats.canonical_info_states < stats.total_info_states,
        "canonical {} should be less than total {}",
        stats.canonical_info_states,
        stats.total_info_states
    );
    assert!(
        stats.canonical_info_states >= 20,
        "canonical {} too low (expected ~21-22)",
        stats.canonical_info_states
    );
    assert!(
        stats.canonical_info_states <= 24,
        "canonical {} too high (expected ~21-22)",
        stats.canonical_info_states
    );
}

#[test]
fn canonical_3x2_roughly_half() {
    let stats = enumerate_game_tree(3, 2);
    assert!(
        stats.canonical_info_states < stats.total_info_states,
        "canonical {} should be less than total {}",
        stats.canonical_info_states,
        stats.total_info_states
    );
    assert!(
        stats.canonical_info_states >= 195,
        "canonical {} too low (expected ~205)",
        stats.canonical_info_states
    );
    assert!(
        stats.canonical_info_states <= 215,
        "canonical {} too high (expected ~205)",
        stats.canonical_info_states
    );
}

#[test]
fn canonical_player_split_sums() {
    let stats = enumerate_game_tree(2, 2);
    assert_eq!(
        stats.canonical_by_player[0] + stats.canonical_by_player[1],
        stats.canonical_info_states
    );
}
