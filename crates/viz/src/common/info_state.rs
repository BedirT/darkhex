//! Info state string manipulation utilities.
//!
//! DarkHex info state format: "P{player_id}\n{grid_rows}"
//! Grid characters: 'x' (black), 'o' (white), '.' (empty from this player's view)

use darkhex_core::game::state::DarkHexState;
use darkhex_core::game::types::Player;

/// Parse an info state string into (player_id, grid).
/// Grid is a flat vector of characters ('x', 'o', '.').
pub fn parse_info_state(info: &str) -> (usize, Vec<char>) {
    let player_id = info.as_bytes()[1] - b'0';
    let grid: Vec<char> = info.lines().skip(1).flat_map(|l| l.chars()).collect();
    (player_id as usize, grid)
}

/// Build an info state string from player_id and grid characters.
pub fn build_info_state(player_id: usize, grid: &[char], cols: usize) -> String {
    let mut s = format!("P{player_id}\n");
    for (i, &c) in grid.iter().enumerate() {
        if i > 0 && i % cols == 0 {
            s.push('\n');
        }
        s.push(c);
    }
    s
}

/// Compute successor info state after an action.
///
/// `acting_player` is 0 (black) or 1 (white) — determines the stone character placed.
/// For the strategy builder, we need to compute successors for both outcomes:
/// - Success: the acting player's stone is placed at the action cell
/// - Collision: the opponent's stone is discovered at the action cell
pub fn info_state_after_action(
    info: &str,
    action: usize,
    acting_player: usize,
    cols: usize,
) -> String {
    let (player_id, mut grid) = parse_info_state(info);
    let stone = if acting_player == 0 { 'x' } else { 'o' };
    grid[action] = stone;
    build_info_state(player_id, &grid, cols)
}

/// Compute the info state after a collision at the given action.
/// The player discovers the opponent's stone at that cell.
pub fn info_state_after_collision(
    info: &str,
    action: usize,
    player_id: usize,
    cols: usize,
) -> String {
    let (pid, mut grid) = parse_info_state(info);
    debug_assert_eq!(pid, player_id);
    // Player discovers opponent's stone
    let opponent_stone = if player_id == 0 { 'o' } else { 'x' };
    grid[action] = opponent_stone;
    build_info_state(pid, &grid, cols)
}

/// Check if a collision is possible at this info state.
///
/// A collision is possible if the opponent could have hidden stones on the board.
/// In CDH Dark Hex, the player whose turn it is might have more or equal stones
/// compared to what they can see of the opponent's stones. If the opponent could
/// have placed stones that the player can't see, collision is possible.
///
/// Simple heuristic: collision is possible if there are empty-appearing cells
/// that could contain opponent stones. This is true when the total number of
/// visible opponent stones is less than what the opponent could have placed.
pub fn is_collision_possible(info: &str) -> bool {
    let (player_id, grid) = parse_info_state(info);
    let own_stone = if player_id == 0 { 'x' } else { 'o' };
    let opp_stone = if player_id == 0 { 'o' } else { 'x' };

    let own_count = grid.iter().filter(|&&c| c == own_stone).count();
    let visible_opp = grid.iter().filter(|&&c| c == opp_stone).count();
    let empty_count = grid.iter().filter(|&&c| c == '.').count();

    // In standard Dark Hex (CDH), players alternate successful placements.
    // Player 0 (Black) moves first. After k successful placements total:
    // - Black has ceil(k/2) stones, White has floor(k/2) stones.
    // The maximum opponent stones = own_count (if player is Black and placed first)
    // or own_count + 1 (if player is White).
    //
    // Collision is possible if there could be hidden opponent stones,
    // i.e., the opponent's true count could exceed visible_opp.
    // Conservative: if there are empty cells and opponent could have stones there.
    if empty_count == 0 {
        return false;
    }

    // If player 0 (Black, moves first): opponent (White) has placed at most own_count stones
    // If player 1 (White): opponent (Black) has placed at most own_count + 1 stones
    let max_opp_stones = if player_id == 0 {
        own_count
    } else {
        own_count + 1
    };

    // Hidden opponent stones = true_opp - visible_opp
    // Collision possible if max_opp > visible_opp
    max_opp_stones > visible_opp
}

/// Check if an info state is terminal using the game engine.
pub fn is_terminal(info: &str, rows: usize, cols: usize) -> bool {
    let (player_id, grid) = parse_info_state(info);
    let n = rows * cols;
    if grid.len() != n {
        return false;
    }

    // Check if all cells are filled (the grid has no empty cells).
    // Also check via engine if there's a winner.
    // Build a DarkHexState and check.
    // For simplicity: a state is terminal if there are no empty cells in view,
    // or if a win can be detected.

    // Use the engine: replay the visible stones onto a state
    let empty_count = grid.iter().filter(|&&c| c == '.').count();
    if empty_count == 0 {
        // Board is full from this player's view — must be terminal
        return true;
    }

    // Check for a win by constructing a board with the visible stones
    // and checking connectivity. We use HexBoard directly.
    use darkhex_core::game::board::HexBoard;
    let mut board = HexBoard::new(rows, cols);
    for (i, &c) in grid.iter().enumerate() {
        match c {
            'x' => {
                board.place_stone(i, Player::Black);
            }
            'o' => {
                board.place_stone(i, Player::White);
            }
            _ => {}
        }
    }
    board.winner().is_some()
}

/// Get all legal actions (empty cells) from an info state.
pub fn legal_actions(info: &str) -> Vec<usize> {
    let (_pid, grid) = parse_info_state(info);
    grid.iter()
        .enumerate()
        .filter(|(_, &c)| c == '.')
        .map(|(i, _)| i)
        .collect()
}

/// Get a random legal action from an info state.
pub fn random_action(info: &str) -> Option<usize> {
    let actions = legal_actions(info);
    if actions.is_empty() {
        return None;
    }
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..actions.len());
    Some(actions[idx])
}

/// Convert a position to an alphanumeric label (a1, b2, etc.).
pub fn pos_to_label(pos: usize, cols: usize) -> String {
    let row = pos / cols;
    let col = pos % cols;
    let row_char = (b'a' + row as u8) as char;
    format!("{}{}", row_char, col + 1)
}

/// Convert an alphanumeric label to a position.
pub fn label_to_pos(label: &str, cols: usize) -> Option<usize> {
    let bytes = label.as_bytes();
    if bytes.len() < 2 {
        return None;
    }
    let row = (bytes[0].to_ascii_lowercase() - b'a') as usize;
    let col_str = &label[1..];
    let col: usize = col_str.parse().ok()?;
    if col == 0 {
        return None;
    }
    Some(row * cols + (col - 1))
}

/// Build the initial info state for a given player and board size.
pub fn initial_info_state(player_id: usize, rows: usize, cols: usize) -> String {
    let grid = vec!['.'; rows * cols];
    build_info_state(player_id, &grid, cols)
}
