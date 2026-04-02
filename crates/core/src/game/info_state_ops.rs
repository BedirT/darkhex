//! Info-state-level operations for the strategy generator.
//!
//! **CDH-only**: These functions implement Classic Dark Hex (CDH) semantics
//! where the current player retries on collision and the opponent learns
//! nothing. They do NOT support Abrupt (ADH), Noisy (NDH), or Flash (FDH)
//! variants — those require turn-passing and opponent observation state that
//! this string-based API cannot encode. Callers must not use these functions
//! for non-CDH variants.
//!
//! These functions operate on info state *strings* (not full game states),
//! enabling the strategy generator to explore all reachable info states for
//! one player by branching on collision/no-collision outcomes.
//!
//! Info state format (matching `DarkHexState::info_state_string`):
//! - Imperfect recall: `"P{player}\n{row0}\n{row1}\n..."` using `.`=empty, `x`=black, `o`=white
//! - Perfect recall:   appends `"\n{player},{action} {player},{action} ..."`

use crate::error::CoreError;
use crate::game::board::HexBoard;
use crate::game::types::Player;

/// Parsed representation of an info state string.
#[derive(Debug, Clone)]
pub struct ParsedInfoState {
    /// Player index: 0 = Black, 1 = White.
    pub player: usize,
    /// Flat board view: '.', 'x', 'o'.
    pub board: Vec<char>,
    pub rows: usize,
    pub cols: usize,
    /// Action history (only populated for perfect-recall states).
    pub action_history: Vec<usize>,
}

/// Parse an info state string into its components.
///
/// Accepts both imperfect-recall (`"P0\n..x\n.o."`) and perfect-recall
/// (`"P0\n..x\n.o.\n0,1 0,3 "`) formats.
pub fn parse_info_state(
    info_state: &str,
    rows: usize,
    cols: usize,
) -> Result<ParsedInfoState, CoreError> {
    let lines: Vec<&str> = info_state.split('\n').collect();
    if lines.is_empty() {
        return Err(CoreError::InvalidArgument("empty info state".into()));
    }

    // Parse player from "P0" or "P1"
    let header = lines[0];
    if header.len() < 2 || !header.starts_with('P') {
        return Err(CoreError::InvalidArgument(format!(
            "invalid header: {header}"
        )));
    }
    let player = header[1..2]
        .parse::<usize>()
        .map_err(|_| CoreError::InvalidArgument(format!("invalid player in header: {header}")))?;
    if player > 1 {
        return Err(CoreError::InvalidArgument(format!(
            "player must be 0 or 1, got {player}"
        )));
    }

    let n = rows * cols;

    // Board rows are lines[1..1+rows]
    if lines.len() < 1 + rows {
        return Err(CoreError::InvalidArgument(format!(
            "expected {} board rows, got {}",
            rows,
            lines.len() - 1
        )));
    }

    let mut board = Vec::with_capacity(n);
    for r in 0..rows {
        let row_str = lines[1 + r];
        if row_str.len() != cols {
            return Err(CoreError::InvalidArgument(format!(
                "row {r} has {} chars, expected {cols}",
                row_str.len()
            )));
        }
        for ch in row_str.chars() {
            match ch {
                '.' | 'x' | 'o' => board.push(ch),
                _ => {
                    return Err(CoreError::InvalidArgument(format!(
                        "invalid board char: '{ch}'"
                    )))
                }
            }
        }
    }

    // Parse action history (perfect recall)
    let action_history = if lines.len() > 1 + rows {
        let hist_line = lines[1 + rows];
        parse_action_history(hist_line)
    } else {
        Vec::new()
    };

    Ok(ParsedInfoState {
        player,
        board,
        rows,
        cols,
        action_history,
    })
}

/// Parse action history from a line like `"0,1 0,3 "`.
fn parse_action_history(line: &str) -> Vec<usize> {
    line.split_whitespace()
        .filter_map(|pair| {
            let parts: Vec<&str> = pair.split(',').collect();
            if parts.len() == 2 {
                parts[1].parse::<usize>().ok()
            } else {
                None
            }
        })
        .collect()
}

/// Cells appearing as '.' in the player's view.
pub fn legal_actions(parsed: &ParsedInfoState) -> Vec<usize> {
    parsed
        .board
        .iter()
        .enumerate()
        .filter_map(|(i, &ch)| if ch == '.' { Some(i) } else { None })
        .collect()
}

/// Can a collision occur at this info state?
///
/// Collision is possible when the opponent could have hidden stones on the
/// board. This depends on piece counts:
/// - Black (player 0): collision possible if `white_count < black_count`
///   (Black moved first, so White has placed fewer stones and could have
///   hidden ones in empty-looking cells)
/// - White (player 1): collision possible if `black_count <= white_count`
///   (Black moved first, so Black has placed at least as many stones)
///
/// Mirrors `util.py:278 is_collusion_possible`.
pub fn is_collision_possible(parsed: &ParsedInfoState) -> bool {
    let mut black_count = 0usize;
    let mut white_count = 0usize;
    for &ch in &parsed.board {
        match ch {
            'x' => black_count += 1,
            'o' => white_count += 1,
            _ => {}
        }
    }
    if parsed.player == 0 {
        // Black's turn: opponent is White. Collision possible if White has
        // hidden stones, i.e., White has placed more stones than Black sees.
        // Since Black sees only discovered White stones, collision is possible
        // when visible_white < visible_black (opponent could have placed
        // stones we haven't seen).
        white_count < black_count
    } else {
        // White's turn: opponent is Black. Black moves first so has >=
        // White's count. Collision possible when visible_black <= visible_white.
        black_count <= white_count
    }
}

/// Compute successor info state after an action.
///
/// `stone_player` determines which stone character is placed:
/// - Same as `parsed.player` → successful placement (player's own stone)
/// - Opponent → collision reveal (opponent's stone discovered)
///
/// Mirrors `util.py:525 info_state_after_action`.
pub fn info_state_after_action(
    parsed: &ParsedInfoState,
    action: usize,
    stone_player: usize,
    perfect_recall: bool,
) -> Result<String, CoreError> {
    if action >= parsed.board.len() {
        return Err(CoreError::InvalidArgument(format!(
            "action {action} out of bounds (board size {})",
            parsed.board.len()
        )));
    }
    if parsed.board[action] != '.' {
        return Err(CoreError::InvalidArgument(format!(
            "cell {action} is not empty ('{}')",
            parsed.board[action]
        )));
    }

    let stone_char = if stone_player == 0 { 'x' } else { 'o' };

    let mut new_board = parsed.board.clone();
    new_board[action] = stone_char;

    let mut new_history = parsed.action_history.clone();
    new_history.push(action);

    let new_parsed = ParsedInfoState {
        player: parsed.player,
        board: new_board,
        rows: parsed.rows,
        cols: parsed.cols,
        action_history: new_history,
    };

    Ok(to_info_state_string(&new_parsed, perfect_recall))
}

/// Terminal detection from info state view.
///
/// Two conditions (either triggers terminal):
/// 1. **Win by connection**: Build a temporary `HexBoard`, place all visible
///    stones, check `winner()` via union-find.
/// 2. **Win by piece count**: The board is effectively full from this player's
///    perspective. Mirrors `util.py:303 is_board_terminal`:
///    - Player 0 (Black): terminal if `white_count + empty_count == black_count`
///    - Player 1 (White): terminal if `black_count + empty_count == white_count + 1`
pub fn is_info_state_terminal(parsed: &ParsedInfoState) -> bool {
    // Check win by connection using HexBoard
    let mut board = HexBoard::new(parsed.rows, parsed.cols);
    for (pos, &ch) in parsed.board.iter().enumerate() {
        match ch {
            'x' => {
                board.place_stone(pos, Player::Black);
            }
            'o' => {
                board.place_stone(pos, Player::White);
            }
            _ => {}
        }
    }
    if board.winner().is_some() {
        return true;
    }

    // Check piece-count terminal condition
    let mut black_count = 0usize;
    let mut white_count = 0usize;
    let mut empty_count = 0usize;
    for &ch in &parsed.board {
        match ch {
            'x' => black_count += 1,
            'o' => white_count += 1,
            '.' => empty_count += 1,
            _ => {}
        }
    }
    if parsed.player == 0 {
        white_count + empty_count == black_count
    } else {
        black_count + empty_count == white_count + 1
    }
}

/// Board view as flat i8 array (0=empty, 1=black, 2=white) for rendering.
pub fn board_view_flat(parsed: &ParsedInfoState) -> Vec<i8> {
    parsed
        .board
        .iter()
        .map(|&ch| match ch {
            'x' => 1,
            'o' => 2,
            _ => 0,
        })
        .collect()
}

/// Reconstruct an info state string from a `ParsedInfoState`.
pub fn to_info_state_string(parsed: &ParsedInfoState, perfect_recall: bool) -> String {
    let board_size = parsed.rows * parsed.cols + parsed.rows; // chars + newlines
    let mut s = String::with_capacity(3 + board_size + 64);

    // Header
    s.push('P');
    s.push(char::from(b'0' + parsed.player as u8));
    s.push('\n');

    // Board rows
    for row in 0..parsed.rows {
        if row > 0 {
            s.push('\n');
        }
        let start = row * parsed.cols;
        for col in 0..parsed.cols {
            s.push(parsed.board[start + col]);
        }
    }

    // Action history (perfect recall)
    if perfect_recall {
        s.push('\n');
        for &action in &parsed.action_history {
            s.push_str(&format!("{},{} ", parsed.player, action));
        }
    }

    s
}

/// Create the initial info state string for a given board size and player.
pub fn initial_info_state(rows: usize, cols: usize, player: usize) -> String {
    let n = rows * cols;
    let parsed = ParsedInfoState {
        player,
        board: vec!['.'; n],
        rows,
        cols,
        action_history: Vec::new(),
    };
    to_info_state_string(&parsed, false)
}

#[cfg(test)]
#[path = "info_state_ops_tests.rs"]
mod tests;
