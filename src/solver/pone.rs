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
        let mut andor_memo: HashMap<Vec<u8>, bool> = HashMap::new();
        let mut visited: HashSet<Vec<u8>> = HashSet::new();

        let state = DarkHexState::rs_new(rows, cols);
        let mut actions_buf = Vec::new();

        pone_traverse(
            &state,
            rows,
            cols,
            &mut states,
            &mut minimax_memo,
            &mut andor_memo,
            &mut visited,
            &mut actions_buf,
        );

        PoneDb { states, rows, cols }
    }

    /// Check if an info state is a pONE win.
    ///
    /// Accepts both canonical and non-canonical info state strings —
    /// the input is canonicalized before lookup.
    fn contains(&self, info_state: &str) -> bool {
        // Canonicalize: reverse the grid characters, compare, pick smaller.
        let canon = canonicalize_info_state_str(info_state, self.rows, self.cols);
        self.states.contains(&canon)
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

/// Canonicalize an info state string by comparing it with its 180° rotation.
/// Returns the lexicographically smaller of (original, rotated).
///
/// If the input is malformed (wrong length, missing prefix), returns
/// the input unchanged rather than panicking.
pub fn canonicalize_info_state_str(info_state: &str, rows: usize, cols: usize) -> String {
    // Format: "P{d}\n{grid}" where grid has \n-separated rows
    let n = rows * cols;
    if info_state.len() < 3 || !info_state.starts_with('P') {
        return info_state.to_string();
    }
    let grid = &info_state[3..];

    // Extract cell characters (skip newlines), reverse, rebuild
    let cells: Vec<char> = grid.chars().filter(|&c| c != '\n').collect();
    if cells.len() != n {
        return info_state.to_string(); // wrong dimension — return as-is
    }
    let reversed: Vec<char> = cells.iter().rev().cloned().collect();

    // Rebuild rotated grid with row breaks
    let prefix = &info_state[..3];
    let mut rotated = String::with_capacity(info_state.len());
    rotated.push_str(prefix);
    for row in 0..rows {
        if row > 0 {
            rotated.push('\n');
        }
        for col in 0..cols {
            rotated.push(reversed[row * cols + col]);
        }
    }

    if info_state <= rotated.as_str() {
        info_state.to_string()
    } else {
        rotated
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
    andor_memo: &mut HashMap<Vec<u8>, bool>,
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
    if is_pone(state, player, rows, cols, minimax_memo, andor_memo) {
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
            andor_memo,
            visited,
            actions_buf,
        );
    }
}

/// Check if the current player has a probability-1 win from this state.
///
/// Extracts the player's view and hidden count from the game state,
/// then delegates to the AND-OR belief-space search.
fn is_pone(
    state: &DarkHexState,
    player: Player,
    rows: usize,
    cols: usize,
    minimax_memo: &mut HashMap<Vec<u8>, Player>,
    andor_memo: &mut HashMap<Vec<u8>, bool>,
) -> bool {
    let pi = player.index();
    let true_cells = state.rs_board_cells();
    let view = &state.rs_player_views()[pi];
    let n = true_cells.len();

    // Count opponent stones on true board vs visible to player
    let opp_cell = Cell::from_player(player.opponent());
    let true_opp_count = true_cells.iter().filter(|&&c| c == opp_cell).count();
    let visible_opp_count = view.iter().filter(|v| **v == Some(opp_cell)).count();
    let hidden_count = true_opp_count - visible_opp_count;

    // Build the player's view as Cell array
    let mut view_cells = vec![Cell::Empty; n];
    for i in 0..n {
        if let Some(c) = view[i] {
            view_cells[i] = c;
        }
    }

    pone_andor(
        &view_cells,
        player,
        hidden_count,
        rows,
        cols,
        minimax_memo,
        andor_memo,
    )
}

/// AND-OR belief-space search for pONE (probability-1 win detection).
///
/// Determines whether `player` can guarantee a win from the given view
/// with `h` hidden opponent stones, using a SINGLE strategy that works
/// against all possible hidden configurations.
///
/// Structure (per thesis §4.2, Bonnet 2018, Russell & Wolfe 2005):
/// - OR nodes: player picks which cell to play
/// - AND nodes: both outcomes (collision / success) must lead to wins
/// - When h=0: delegate to perfect-information hex_minimax
/// - Opponent turns: increment h (opponent places a hidden stone)
fn pone_andor(
    view: &[Cell],
    player: Player,
    h: usize,
    rows: usize,
    cols: usize,
    minimax_memo: &mut HashMap<Vec<u8>, Player>,
    andor_memo: &mut HashMap<Vec<u8>, bool>,
) -> bool {
    // Memoization key: view cells + hidden count
    let mut key = Vec::with_capacity(view.len() + 1);
    for &c in view {
        key.push(c as u8);
    }
    key.push(h as u8);

    if let Some(&result) = andor_memo.get(&key) {
        return result;
    }

    // Check if player already won on the visible board
    let mut board = HexBoard::new(rows, cols);
    for (i, &c) in view.iter().enumerate() {
        if c != Cell::Empty {
            let p = match c {
                Cell::Black => Player::Black,
                Cell::White => Player::White,
                _ => unreachable!(),
            };
            board.place_stone(i, p);
        }
    }
    if board.winner() == Some(player) {
        andor_memo.insert(key, true);
        return true;
    }
    if board.winner() == Some(player.opponent()) {
        andor_memo.insert(key, false);
        return false;
    }

    // Determine whose turn it is from stone counts.
    // Black moves first; turns alternate. Total Black moves = visible Black stones.
    // Total White moves = visible White stones + hidden White stones (if player is Black)
    //                   or visible Black stones + hidden Black stones (if player is White).
    let player_cell = Cell::from_player(player);
    let opp_cell = Cell::from_player(player.opponent());
    let player_stones = view.iter().filter(|&&c| c == player_cell).count();
    let opp_visible = view.iter().filter(|&&c| c == opp_cell).count();
    let opp_total = opp_visible + h;

    // In Hex: Black has count_b stones, White has count_w total (visible + hidden).
    // Black's turn when count_b <= count_w.
    let is_player_turn = if player == Player::Black {
        player_stones <= opp_total
    } else {
        // player is White: player_stones = White stones, opp_total = Black stones
        // Black's turn when Black_stones <= White_total → opp_total <= player_stones + h_for_white
        // Actually: from generic perspective, it's player's turn when their stone count
        // is behind or equal in the alternation sequence.
        // Black goes first. Total moves = player_stones + opp_total.
        // If total is even → Black's turn. If odd → White's turn.
        // Player's turn iff (player==Black && total even) || (player==White && total odd)
        let total_moves = player_stones + opp_total;
        total_moves % 2 == 1 // White's turn when total moves is odd
    };

    if !is_player_turn {
        // Opponent's turn: they place a hidden stone.
        // The view doesn't change (player can't see it), but h increases.
        let empty_count = view.iter().filter(|&&c| c == Cell::Empty).count();
        if h >= empty_count {
            // No room for another hidden stone — game should be terminal
            andor_memo.insert(key, false);
            return false;
        }
        let result = pone_andor(view, player, h + 1, rows, cols, minimax_memo, andor_memo);
        andor_memo.insert(key, result);
        return result;
    }

    // Player's turn — find empty-appearing cells
    let empty_cells: Vec<usize> = view
        .iter()
        .enumerate()
        .filter(|(_, &c)| c == Cell::Empty)
        .map(|(i, _)| i)
        .collect();

    if empty_cells.is_empty() {
        andor_memo.insert(key, false);
        return false;
    }

    let result = if h == 0 {
        // No hidden stones: perfect information. OR-search via minimax.
        hex_minimax(view, rows, cols, player, minimax_memo) == player
    } else {
        // AND-OR search: player picks a cell y (OR), both outcomes must
        // lead to a win (AND).
        let mut found = false;
        for &y in &empty_cells {
            // AND branch 1: collision at y — reveal hidden opponent stone
            let mut collision_view = view.to_vec();
            collision_view[y] = opp_cell;
            let collision_ok =
                pone_andor(&collision_view, player, h - 1, rows, cols, minimax_memo, andor_memo);

            if !collision_ok {
                continue; // This action fails in the collision case
            }

            // AND branch 2: success at y — place own stone
            let mut success_view = view.to_vec();
            success_view[y] = player_cell;
            let success_ok =
                pone_andor(&success_view, player, h, rows, cols, minimax_memo, andor_memo);

            if success_ok {
                found = true;
                break; // Found an action that works for both outcomes
            }
        }
        found
    };

    andor_memo.insert(key, result);
    result
}

#[cfg(test)]
#[path = "pone_tests.rs"]
mod tests;
