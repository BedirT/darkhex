use std::collections::{HashMap, HashSet};

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

    let empty: Vec<usize> = cells
        .iter()
        .enumerate()
        .filter(|(_, c)| **c == Cell::Empty)
        .map(|(i, _)| i)
        .collect();

    if empty.is_empty() {
        let result = current_player.opponent();
        memo.insert(key, result);
        return result;
    }

    let next = current_player.opponent();
    let mut result = current_player.opponent();
    for &pos in &empty {
        let mut child = cells.to_vec();
        child[pos] = Cell::from_player(current_player);
        let winner = hex_minimax(&child, rows, cols, next, memo);
        if winner == current_player {
            result = current_player;
            break;
        }
    }

    memo.insert(key, result);
    result
}

/// Database of probability-one win states for Dark Hex.
#[derive(Clone)]
pub struct PoneDb {
    /// Set of canonical info state strings that are pONE wins.
    states: HashSet<String>,
    rows: usize,
    cols: usize,
}

impl PoneDb {
    /// Build the pONE database by traversing the full game tree.
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut states = HashSet::new();
        let mut minimax_memo: HashMap<Vec<u8>, Player> = HashMap::new();
        let mut andor_memo: HashMap<Vec<u8>, bool> = HashMap::new();
        let mut visited: HashSet<Vec<u8>> = HashSet::new();

        let state = DarkHexState::new_cdh(rows, cols);
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
    pub fn contains(&self, info_state: &str) -> bool {
        let canon = canonicalize_info_state_str(info_state, self.rows, self.cols);
        self.states.contains(&canon)
    }

    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Rust-internal lookup (already-canonical key).
    pub fn contains_canonical(&self, info_state: &str) -> bool {
        self.states.contains(info_state)
    }
}

/// Canonicalize an info state string by comparing it with its 180° rotation.
pub fn canonicalize_info_state_str(info_state: &str, rows: usize, cols: usize) -> String {
    let n = rows * cols;
    if info_state.len() < 3 || !info_state.starts_with('P') {
        return info_state.to_string();
    }
    let grid = &info_state[3..];

    let cells: Vec<char> = grid.chars().filter(|&c| c != '\n').collect();
    if cells.len() != n {
        return info_state.to_string();
    }
    let reversed: Vec<char> = cells.iter().rev().cloned().collect();

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

/// Compact state key for memoization.
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
    if state.is_terminal() {
        return;
    }

    let key = state_key(state);
    if !visited.insert(key) {
        return;
    }

    let player = state.current_player();

    if is_pone(state, player, rows, cols, minimax_memo, andor_memo) {
        let (canon, _) = state.canonical_info_state(player);
        pone_states.insert(canon);
    }

    state.legal_actions_buf(actions_buf);
    let actions: Vec<usize> = actions_buf.clone();
    for &action in &actions {
        let mut child = state.clone();
        child.apply_action_unchecked(action);
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

fn is_pone(
    state: &DarkHexState,
    player: Player,
    rows: usize,
    cols: usize,
    minimax_memo: &mut HashMap<Vec<u8>, Player>,
    andor_memo: &mut HashMap<Vec<u8>, bool>,
) -> bool {
    let pi = player.index();
    let true_cells = state.board_cells();
    let view = &state.player_views()[pi];
    let n = true_cells.len();

    let opp_cell = Cell::from_player(player.opponent());
    let true_opp_count = true_cells.iter().filter(|&&c| c == opp_cell).count();
    let visible_opp_count = view.iter().filter(|v| **v == Some(opp_cell)).count();
    let hidden_count = true_opp_count - visible_opp_count;

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

fn pone_andor(
    view: &[Cell],
    player: Player,
    h: usize,
    rows: usize,
    cols: usize,
    minimax_memo: &mut HashMap<Vec<u8>, Player>,
    andor_memo: &mut HashMap<Vec<u8>, bool>,
) -> bool {
    let mut key = Vec::with_capacity(view.len() + 1);
    for &c in view {
        key.push(c as u8);
    }
    key.push(h as u8);

    if let Some(&result) = andor_memo.get(&key) {
        return result;
    }

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

    let player_cell = Cell::from_player(player);
    let opp_cell = Cell::from_player(player.opponent());
    let player_stones = view.iter().filter(|&&c| c == player_cell).count();
    let opp_visible = view.iter().filter(|&&c| c == opp_cell).count();
    let opp_total = opp_visible + h;

    let is_player_turn = if player == Player::Black {
        player_stones <= opp_total
    } else {
        let total_moves = player_stones + opp_total;
        total_moves % 2 == 1
    };

    if !is_player_turn {
        let empty_count = view.iter().filter(|&&c| c == Cell::Empty).count();
        if h >= empty_count {
            andor_memo.insert(key, false);
            return false;
        }
        let result = pone_andor(view, player, h + 1, rows, cols, minimax_memo, andor_memo);
        andor_memo.insert(key, result);
        return result;
    }

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
        hex_minimax(view, rows, cols, player, minimax_memo) == player
    } else {
        let mut found = false;
        for &y in &empty_cells {
            let mut collision_view = view.to_vec();
            collision_view[y] = opp_cell;
            let collision_ok =
                pone_andor(&collision_view, player, h - 1, rows, cols, minimax_memo, andor_memo);

            if !collision_ok {
                continue;
            }

            let mut success_view = view.to_vec();
            success_view[y] = player_cell;
            let success_ok =
                pone_andor(&success_view, player, h, rows, cols, minimax_memo, andor_memo);

            if success_ok {
                found = true;
                break;
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
