use std::collections::HashSet;

use pyo3::prelude::*;

use crate::game::state::DarkHexState;
use crate::game::types::Player;

/// Exhaustive game tree traversal to collect all reachable info states.
///
/// This is the ground truth for info state counts — no sampling, no
/// randomness, no approximation. Every reachable decision point in
/// the game tree is visited and its info state string recorded.
///
/// CDH collision retries make the tree deeper but the HashSet
/// deduplicates, so we only count unique board views.
#[pyclass]
pub struct GameTreeStats {
    /// All unique info states found (both players).
    #[pyo3(get)]
    pub total_info_states: usize,
    /// Info states per player.
    #[pyo3(get)]
    pub info_states_by_player: [usize; 2],
    /// Total terminal states (game-over leaves).
    #[pyo3(get)]
    pub terminal_states: usize,
    /// Maximum depth reached in the tree.
    #[pyo3(get)]
    pub max_depth: usize,
}

#[pymethods]
impl GameTreeStats {
    fn __repr__(&self) -> String {
        format!(
            "GameTreeStats(info_states={}, P0={}, P1={}, terminals={}, max_depth={})",
            self.total_info_states,
            self.info_states_by_player[0],
            self.info_states_by_player[1],
            self.terminal_states,
            self.max_depth,
        )
    }
}

/// Enumerate all reachable info states for a given board size (CDH).
///
/// Performs exhaustive depth-first traversal of the full game tree.
/// Returns exact counts — use this to verify MCCFR coverage.
#[pyfunction]
pub fn enumerate_game_tree(rows: usize, cols: usize) -> GameTreeStats {
    let mut info_states: [HashSet<String>; 2] = [HashSet::new(), HashSet::new()];
    let mut terminal_count: usize = 0;
    let mut max_depth: usize = 0;

    let state = DarkHexState::rs_new(rows, cols);
    let mut actions_buf = Vec::new();

    traverse(
        &state,
        0,
        &mut info_states,
        &mut terminal_count,
        &mut max_depth,
        &mut actions_buf,
    );

    let p0 = info_states[0].len();
    let p1 = info_states[1].len();
    GameTreeStats {
        total_info_states: p0 + p1,
        info_states_by_player: [p0, p1],
        terminal_states: terminal_count,
        max_depth,
    }
}

fn traverse(
    state: &DarkHexState,
    depth: usize,
    info_states: &mut [HashSet<String>; 2],
    terminal_count: &mut usize,
    max_depth: &mut usize,
    actions_buf: &mut Vec<usize>,
) {
    if state.rs_is_terminal() {
        *terminal_count += 1;
        if depth > *max_depth {
            *max_depth = depth;
        }
        return;
    }

    let player = state.rs_current_player();
    let pi = player.index();
    let info_key = state.rs_info_state_string(player);
    info_states[pi].insert(info_key);

    state.rs_legal_actions(actions_buf);
    let actions: Vec<usize> = actions_buf.clone();

    for &action in &actions {
        let mut child = state.clone();
        child.rs_apply_action(action);
        traverse(&child, depth + 1, info_states, terminal_count, max_depth, actions_buf);
    }
}

#[cfg(test)]
#[path = "enumerate_tests.rs"]
mod tests;
