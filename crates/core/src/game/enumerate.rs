use std::collections::HashSet;

use crate::game::state::DarkHexState;

/// Exhaustive game tree traversal to collect all reachable info states.
///
/// This is the ground truth for info state counts — no sampling, no
/// randomness, no approximation. Every reachable decision point in
/// the game tree is visited and its info state string recorded.
///
/// Optimization: memoizes on (true_board, current_player, player_views)
/// to avoid re-traversing identical subtrees reached via different
/// move orderings. This cuts 3x3 from 6.2h to seconds.
pub struct GameTreeStats {
    /// All unique info states found (both players).
    pub total_info_states: usize,
    /// Info states per player.
    pub info_states_by_player: [usize; 2],
    /// Unique canonical info states (under 180° rotation symmetry).
    pub canonical_info_states: usize,
    /// Canonical info states per player.
    pub canonical_by_player: [usize; 2],
    /// Total unique game states visited (not terminal histories).
    pub game_states_visited: usize,
    /// Maximum depth reached in the tree.
    pub max_depth: usize,
}

/// Compact key for memoization: true board cells + both player views.
/// Two states with identical keys produce identical subtrees.
fn state_key(state: &DarkHexState) -> Vec<u8> {
    let n = state.board_cells().len();
    let mut key = Vec::with_capacity(3 * n + 1);
    for &c in state.board_cells() {
        key.push(c as u8);
    }
    for view in state.player_views() {
        for v in view {
            key.push(match v {
                None => 0,
                Some(c) => *c as u8 + 1,
            });
        }
    }
    key.push(state.current_player().index() as u8);
    key
}

/// Enumerate all reachable info states for a given board size (CDH).
///
/// Uses memoization on full game state to avoid re-traversing identical
/// subtrees. This makes 3x3 feasible in seconds instead of hours.
pub fn enumerate_game_tree(rows: usize, cols: usize) -> GameTreeStats {
    let mut info_states: [HashSet<String>; 2] = [HashSet::new(), HashSet::new()];
    let mut canonical_states: [HashSet<String>; 2] = [HashSet::new(), HashSet::new()];
    let mut visited: HashSet<Vec<u8>> = HashSet::new();
    let mut max_depth: usize = 0;

    let state = DarkHexState::new_cdh(rows, cols);
    let mut actions_buf = Vec::new();

    traverse(
        &state,
        0,
        &mut info_states,
        &mut canonical_states,
        &mut visited,
        &mut max_depth,
        &mut actions_buf,
    );

    let p0 = info_states[0].len();
    let p1 = info_states[1].len();
    let cp0 = canonical_states[0].len();
    let cp1 = canonical_states[1].len();
    GameTreeStats {
        total_info_states: p0 + p1,
        info_states_by_player: [p0, p1],
        canonical_info_states: cp0 + cp1,
        canonical_by_player: [cp0, cp1],
        game_states_visited: visited.len(),
        max_depth,
    }
}

fn traverse(
    state: &DarkHexState,
    depth: usize,
    info_states: &mut [HashSet<String>; 2],
    canonical_states: &mut [HashSet<String>; 2],
    visited: &mut HashSet<Vec<u8>>,
    max_depth: &mut usize,
    actions_buf: &mut Vec<usize>,
) {
    if state.is_terminal() {
        if depth > *max_depth {
            *max_depth = depth;
        }
        return;
    }

    // Memoize: if we've seen this exact game state, skip
    let key = state_key(state);
    if !visited.insert(key) {
        return;
    }

    let player = state.current_player();
    let pi = player.index();
    info_states[pi].insert(state.info_state_string(player));
    let (canon, _) = state.canonical_info_state(player);
    canonical_states[pi].insert(canon);

    state.legal_actions_buf(actions_buf);
    let actions: Vec<usize> = actions_buf.clone();

    for &action in &actions {
        let mut child = state.clone();
        child.apply_action_unchecked(action);
        traverse(
            &child,
            depth + 1,
            info_states,
            canonical_states,
            visited,
            max_depth,
            actions_buf,
        );
    }
}

#[cfg(test)]
#[path = "enumerate_tests.rs"]
mod tests;
