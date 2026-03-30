use std::collections::{HashMap, HashSet};

use pyo3::prelude::*;

use crate::game::board::HexBoard;
use crate::game::state::DarkHexState;
use crate::game::types::{Cell, Player};

/// Board key for minimax memoization: just cells + current player.
fn minimax_key(cells: &[Cell], current_player: Player) -> Vec<u8> {
    let mut key = Vec::with_capacity(cells.len() + 1);
    for &c in cells {
        key.push(c as u8);
    }
    key.push(current_player.index() as u8);
    key
}

/// Solve a regular Hex position via minimax with alpha-beta pruning.
/// Returns the guaranteed winner assuming both play optimally.
fn hex_minimax(
    cells: &[Cell],
    rows: usize,
    cols: usize,
    current_player: Player,
    memo: &mut HashMap<Vec<u8>, Player>,
) -> Player {
    let key = minimax_key(cells, current_player);
    if let Some(&winner) = memo.get(&key) {
        return winner;
    }

    // Check if there's already a winner on the board
    let mut board = HexBoard::new(rows, cols);
    for (i, &c) in cells.iter().enumerate() {
        if c != Cell::Empty {
            let player = match c {
                Cell::Black => Player::Black,
                Cell::White => Player::White,
                _ => unreachable!(),
            };
            board.place_stone(i, player);
        }
    }
    if let Some(winner) = board.winner() {
        memo.insert(key, winner);
        return winner;
    }

    // Find empty cells
    let empty: Vec<usize> = cells
        .iter()
        .enumerate()
        .filter(|(_, c)| **c == Cell::Empty)
        .map(|(i, _)| i)
        .collect();

    if empty.is_empty() {
        // Full board with no winner — shouldn't happen in Hex
        // but return opponent as pessimistic default
        let result = current_player.opponent();
        memo.insert(key, result);
        return result;
    }

    // Try each empty cell; if current player finds a win, prune
    let next = current_player.opponent();
    let mut result = current_player.opponent(); // pessimistic default
    for &pos in &empty {
        let mut child = cells.to_vec();
        child[pos] = Cell::from_player(current_player);
        let winner = hex_minimax(&child, rows, cols, next, memo);
        if winner == current_player {
            result = current_player;
            break; // alpha-beta prune
        }
    }

    memo.insert(key, result);
    result
}

/// Database of probability-one win states for Dark Hex.
///
/// A state is pONE for the current player if they can win with
/// probability 1 from their information state, accounting for ALL
/// possible placements of hidden opponent stones.
///
/// This is a belief-space check: for each possible configuration of
/// hidden stones consistent with the player's view, the player must
/// be able to force a win.
#[pyclass]
#[derive(Clone)]
pub struct PoneDb {
    /// Set of canonical info state strings that are pONE wins.
    states: HashSet<String>,
    rows: usize,
    cols: usize,
}

#[pymethods]
impl PoneDb {
    /// Build the pONE database by traversing the full game tree.
    #[new]
    fn new(rows: usize, cols: usize) -> Self {
        let mut states = HashSet::new();
        let mut minimax_memo: HashMap<Vec<u8>, Player> = HashMap::new();
        let mut visited: HashSet<Vec<u8>> = HashSet::new();

        let state = DarkHexState::rs_new(rows, cols);
        let mut actions_buf = Vec::new();

        pone_traverse(
            &state,
            rows,
            cols,
            &mut states,
            &mut minimax_memo,
            &mut visited,
            &mut actions_buf,
        );

        PoneDb { states, rows, cols }
    }

    /// Check if an info state is a pONE win.
    fn contains(&self, info_state: &str) -> bool {
        self.states.contains(info_state)
    }

    /// Number of pONE states in the database.
    fn len(&self) -> usize {
        self.states.len()
    }

    fn rows(&self) -> usize {
        self.rows
    }

    fn cols(&self) -> usize {
        self.cols
    }

    fn __repr__(&self) -> String {
        format!(
            "PoneDb({}x{}, {} pONE states)",
            self.rows,
            self.cols,
            self.states.len()
        )
    }
}

impl PoneDb {
    /// Rust-internal lookup.
    pub fn rs_contains(&self, info_state: &str) -> bool {
        self.states.contains(info_state)
    }
}

/// Compact state key for memoization (same as in enumerate.rs).
fn state_key(state: &DarkHexState) -> Vec<u8> {
    let n = state.rs_board_cells().len();
    let mut key = Vec::with_capacity(3 * n + 1);
    for &c in state.rs_board_cells() {
        key.push(c as u8);
    }
    for view in state.rs_player_views() {
        for v in view {
            key.push(match v {
                None => 0,
                Some(c) => *c as u8 + 1,
            });
        }
    }
    key.push(state.rs_current_player().index() as u8);
    key
}

/// Traverse the game tree, checking pONE at each node.
fn pone_traverse(
    state: &DarkHexState,
    rows: usize,
    cols: usize,
    pone_states: &mut HashSet<String>,
    minimax_memo: &mut HashMap<Vec<u8>, Player>,
    visited: &mut HashSet<Vec<u8>>,
    actions_buf: &mut Vec<usize>,
) {
    if state.rs_is_terminal() {
        return;
    }

    let key = state_key(state);
    if !visited.insert(key) {
        return;
    }

    let player = state.rs_current_player();

    // Check pONE for current player
    if is_pone(state, player, rows, cols, minimax_memo) {
        let (canon, _) = state.rs_canonical_info_state(player);
        pone_states.insert(canon);
    }

    // Recurse into children
    state.rs_legal_actions(actions_buf);
    let actions: Vec<usize> = actions_buf.clone();
    for &action in &actions {
        let mut child = state.clone();
        child.rs_apply_action(action);
        pone_traverse(
            &child,
            rows,
            cols,
            pone_states,
            minimax_memo,
            visited,
            actions_buf,
        );
    }
}

/// Check if the current player has a probability-1 win from this state.
///
/// Algorithm:
/// 1. Determine hidden_count = opponent's true stones - opponent's visible stones
/// 2. Find empty-appearing cells (cells that look empty to the player)
/// 3. For all C(empty, hidden) placements of hidden stones:
///    a. Construct the hypothetical true board
///    b. Check if the player can force a win (minimax on true board)
/// 4. If player wins in ALL placements -> pONE
fn is_pone(
    state: &DarkHexState,
    player: Player,
    rows: usize,
    cols: usize,
    minimax_memo: &mut HashMap<Vec<u8>, Player>,
) -> bool {
    let pi = player.index();
    let true_cells = state.rs_board_cells();
    let view = &state.rs_player_views()[pi];
    let n = true_cells.len();

    // Count opponent stones on true board
    let opp_cell = Cell::from_player(player.opponent());
    let true_opp_count = true_cells.iter().filter(|&&c| c == opp_cell).count();

    // Count opponent stones visible to this player
    let visible_opp_count = view.iter().filter(|v| **v == Some(opp_cell)).count();

    let hidden_count = true_opp_count - visible_opp_count;

    // Find cells that appear empty to this player
    let empty_appearing: Vec<usize> = (0..n).filter(|&i| view[i].is_none()).collect();

    if hidden_count == 0 {
        // Player sees full picture — minimax check on the true board
        return hex_minimax(true_cells, rows, cols, player, minimax_memo) == player;
    }

    // Enumerate all placements of hidden_count opponent stones
    // among the empty-appearing cells.
    // For each placement, check if the player can still force a win.
    let combos = combinations(&empty_appearing, hidden_count);
    for combo in &combos {
        // Construct hypothetical true board with hidden stones placed
        let mut hypo_cells = true_cells.to_vec();
        for &pos in combo {
            hypo_cells[pos] = opp_cell;
        }
        // Check if player can force a win on this hypothetical board
        if hex_minimax(&hypo_cells, rows, cols, player, minimax_memo) != player {
            return false; // Found a placement where player can't win
        }
    }

    true // Player wins in all placements
}

/// Generate all combinations of `k` elements from `items`.
fn combinations(items: &[usize], k: usize) -> Vec<Vec<usize>> {
    if k == 0 {
        return vec![vec![]];
    }
    if items.len() < k {
        return vec![];
    }
    let mut result = Vec::new();
    combine_helper(items, k, 0, &mut Vec::with_capacity(k), &mut result);
    result
}

fn combine_helper(
    items: &[usize],
    k: usize,
    start: usize,
    current: &mut Vec<usize>,
    result: &mut Vec<Vec<usize>>,
) {
    if current.len() == k {
        result.push(current.clone());
        return;
    }
    let remaining = k - current.len();
    for i in start..=(items.len() - remaining) {
        current.push(items[i]);
        combine_helper(items, k, i + 1, current, result);
        current.pop();
    }
}

#[cfg(test)]
#[path = "pone_tests.rs"]
mod tests;
