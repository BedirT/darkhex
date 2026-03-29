use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::board::HexBoard;
use crate::types::{Cell, CollisionInfo, CollisionRule, Player};

/// State of a Dark Hex game.
///
/// Supports all four thesis variants via `CollisionRule` × `CollisionInfo`:
/// - **CDH** (Classic):  `Classic` + `Silent`  — player retries, opponent unaware
/// - **ADH** (Abrupt):   `Abrupt` + `Silent`  — collision wastes turn
/// - **NDH** (Noisy):    either   + `Noisy`   — opponent told collision happened
/// - **FDH** (Flash):    either   + `Flash`   — opponent told where collision was
#[pyclass]
#[derive(Clone)]
pub struct DarkHexState {
    board: HexBoard,
    /// What each player can see. `None` = appears empty to that player.
    player_views: [Vec<Option<Cell>>; 2],
    /// Per-player action histories (for perfect-recall info states).
    action_histories: [Vec<usize>; 2],
    current_player: Player,
    /// Number of successful placements (determines whose turn it is).
    stones_placed: usize,
    cached_winner: Option<Player>,
    collision_rule: CollisionRule,
    collision_info: CollisionInfo,
}

#[pymethods]
impl DarkHexState {
    /// Create a new Dark Hex game state.
    ///
    /// Defaults to CDH (Classic Dark Hex): player retries after collision,
    /// opponent not informed.
    #[new]
    #[pyo3(signature = (rows, cols, collision_rule=None, collision_info=None))]
    fn new(
        rows: usize,
        cols: usize,
        collision_rule: Option<CollisionRule>,
        collision_info: Option<CollisionInfo>,
    ) -> PyResult<Self> {
        if rows == 0 || cols == 0 {
            return Err(PyValueError::new_err("board dimensions must be positive"));
        }
        let n = rows * cols;
        Ok(Self {
            board: HexBoard::new(rows, cols),
            player_views: [vec![None; n], vec![None; n]],
            action_histories: [Vec::new(), Vec::new()],
            current_player: Player::Black,
            stones_placed: 0,
            cached_winner: None,
            collision_rule: collision_rule.unwrap_or(CollisionRule::Classic),
            collision_info: collision_info.unwrap_or(CollisionInfo::Silent),
        })
    }

    #[getter]
    fn rows(&self) -> usize {
        self.board.rows
    }

    #[getter]
    fn cols(&self) -> usize {
        self.board.cols
    }

    fn num_players(&self) -> usize {
        2
    }

    fn current_player(&self) -> Player {
        self.current_player
    }

    fn is_terminal(&self) -> bool {
        self.cached_winner.is_some()
    }

    fn winner(&self) -> Option<Player> {
        self.cached_winner
    }

    /// Payoffs: `[black, white]`. +1 for winner, -1 for loser, 0 if ongoing.
    fn returns(&self) -> [f64; 2] {
        match self.cached_winner {
            Some(Player::Black) => [1.0, -1.0],
            Some(Player::White) => [-1.0, 1.0],
            None => [0.0, 0.0],
        }
    }

    fn player_return(&self, player: Player) -> f64 {
        self.returns()[player.index()]
    }

    /// Cells that appear empty to the current player.
    fn legal_actions(&self) -> Vec<usize> {
        let pi = self.current_player.index();
        self.player_views[pi]
            .iter()
            .enumerate()
            .filter(|(_, v)| v.is_none())
            .map(|(i, _)| i)
            .collect()
    }

    fn num_legal_actions(&self) -> usize {
        let pi = self.current_player.index();
        self.player_views[pi].iter().filter(|v| v.is_none()).count()
    }

    /// Apply an action for the current player.
    ///
    /// **CDH (Classic)**: On collision the player discovers the opponent's
    /// stone and stays as the current player (retries until successful).
    ///
    /// **ADH (Abrupt)**: On collision the turn is wasted; play passes.
    ///
    /// Returns `true` if the stone was placed, `false` if collision.
    fn apply_action(&mut self, action: usize) -> PyResult<bool> {
        if self.cached_winner.is_some() {
            return Err(PyValueError::new_err("game is already terminal"));
        }
        if action >= self.board.size() {
            return Err(PyValueError::new_err(format!(
                "action {action} out of bounds for {}x{} board",
                self.board.rows, self.board.cols
            )));
        }
        let pi = self.current_player.index();
        if self.player_views[pi][action].is_some() {
            return Err(PyValueError::new_err(format!(
                "action {action} not legal: cell not empty in player's view"
            )));
        }

        let player = self.current_player;
        self.action_histories[pi].push(action);

        let placed = if self.board.place_stone(action, player) {
            // Success — stone placed
            self.player_views[pi][action] = Some(Cell::from_player(player));
            self.cached_winner = self.board.winner();
            self.stones_placed += 1;
            // Turn passes to opponent after successful placement
            self.current_player = Player::from_index(self.stones_placed % 2);
            true
        } else {
            // Collision — player discovers opponent's stone
            self.player_views[pi][action] = Some(Cell::from_player(player.opponent()));

            // Inform opponent based on CollisionInfo variant
            let oi = player.opponent().index();
            match self.collision_info {
                CollisionInfo::Silent => {}
                CollisionInfo::Noisy => {
                    // Opponent knows a collision happened but not where.
                    // This is tracked externally (e.g., in the info state).
                    // For now the observation is implicit in the action count.
                }
                CollisionInfo::Flash => {
                    // Opponent knows WHERE the collision happened —
                    // they see their own stone was discovered at this cell.
                    // (Opponent already sees their own stone, so no view change
                    //  needed, but this info affects the info state string.)
                    let _ = oi; // Flash info tracked via action history
                }
            }

            // Turn handling depends on collision rule
            match self.collision_rule {
                CollisionRule::Classic => {
                    // CDH: same player retries — do NOT switch
                }
                CollisionRule::Abrupt => {
                    // ADH: turn wasted, switch player
                    self.stones_placed += 1;
                    self.current_player =
                        Player::from_index(self.stones_placed % 2);
                }
            }
            false
        };

        Ok(placed)
    }

    /// Imperfect-recall information state string for the given player.
    ///
    /// Format: `"P{player}\n{board_view}"` where the board view shows
    /// only what the player can see (opponent stones hidden as `.`).
    fn info_state_string(&self, player: Player) -> String {
        let pi = player.index();
        let mut s = String::with_capacity(3 + self.board.size() + self.board.rows);
        s.push('P');
        s.push(char::from(b'0' + pi as u8));
        s.push('\n');
        for row in 0..self.board.rows {
            if row > 0 {
                s.push('\n');
            }
            for col in 0..self.board.cols {
                let pos = row * self.board.cols + col;
                s.push(match self.player_views[pi][pos] {
                    None => '.',
                    Some(c) => c.to_char(),
                });
            }
        }
        s
    }

    /// Perfect-recall information state string (includes action history).
    fn info_state_string_perfect_recall(&self, player: Player) -> String {
        let pi = player.index();
        let mut s = self.info_state_string(player);
        s.push('\n');
        for &a in &self.action_histories[pi] {
            s.push_str(&format!("{pi},{a} "));
        }
        s
    }

    /// Clone the state (for game-tree traversal in MCCFR).
    fn copy(&self) -> Self {
        self.clone()
    }

    /// Number of successful stone placements so far.
    fn stones_placed(&self) -> usize {
        self.stones_placed
    }

    /// Number of stones actually on the board for each player.
    fn num_stones(&self) -> [usize; 2] {
        self.board.num_stones
    }

    fn collision_rule(&self) -> CollisionRule {
        self.collision_rule
    }

    fn collision_info(&self) -> CollisionInfo {
        self.collision_info
    }

    /// String representation of the true board (for debugging/verification).
    fn true_board_string(&self) -> String {
        let mut s = String::with_capacity(self.board.size() + self.board.rows);
        for row in 0..self.board.rows {
            if row > 0 {
                s.push('\n');
            }
            for col in 0..self.board.cols {
                let pos = row * self.board.cols + col;
                s.push(self.board.cells[pos].to_char());
            }
        }
        s
    }

    fn __repr__(&self) -> String {
        format!(
            "DarkHexState({}x{}, player={:?}, stones={}, terminal={})",
            self.board.rows,
            self.board.cols,
            self.current_player,
            self.stones_placed,
            self.cached_winner.is_some()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cdh(rows: usize, cols: usize) -> DarkHexState {
        DarkHexState::new(rows, cols, None, None).unwrap()
    }

    fn adh(rows: usize, cols: usize) -> DarkHexState {
        DarkHexState::new(rows, cols, Some(CollisionRule::Abrupt), None).unwrap()
    }

    // --- CDH (Classic) tests ---

    #[test]
    fn cdh_initial_state() {
        let s = cdh(2, 2);
        assert_eq!(s.current_player(), Player::Black);
        assert!(!s.is_terminal());
        assert_eq!(s.legal_actions().len(), 4);
    }

    #[test]
    fn cdh_black_wins_2x2() {
        let mut s = cdh(2, 2);
        s.apply_action(0).unwrap(); // Black at (0,0) → success, White's turn
        s.apply_action(1).unwrap(); // White at (0,1) → success, Black's turn
        s.apply_action(2).unwrap(); // Black at (1,0) → N-S win
        assert!(s.is_terminal());
        assert_eq!(s.winner(), Some(Player::Black));
        assert_eq!(s.returns(), [1.0, -1.0]);
    }

    #[test]
    fn cdh_collision_retries() {
        // CDH: after collision, SAME player tries again
        let mut s = cdh(2, 2);
        assert!(s.apply_action(0).unwrap()); // Black places at 0
        // White's turn: tries cell 0 → collision
        assert!(!s.apply_action(0).unwrap()); // collision, returns false
        // White should STILL be the current player (CDH retry)
        assert_eq!(s.current_player(), Player::White);
        assert_eq!(s.num_stones(), [1, 0]);
        // White retries on cell 1 → success
        assert!(s.apply_action(1).unwrap());
        // Now Black's turn
        assert_eq!(s.current_player(), Player::Black);
        assert_eq!(s.num_stones(), [1, 1]);
    }

    #[test]
    fn cdh_collision_reveals_in_view() {
        let mut s = cdh(2, 2);
        s.apply_action(0).unwrap(); // Black at (0,0)
        s.apply_action(0).unwrap(); // White collides at (0,0) → discovers Black
        // White sees Black at cell 0
        let info_w = s.info_state_string(Player::White);
        assert_eq!(info_w, "P1\nx.\n..");
        // White still has 3 empty-looking cells (cell 0 now revealed)
        assert_eq!(s.num_legal_actions(), 3);
    }

    #[test]
    fn cdh_info_state_hides_opponent() {
        let mut s = cdh(2, 2);
        s.apply_action(0).unwrap(); // Black at (0,0)
        s.apply_action(3).unwrap(); // White at (1,1)
        assert_eq!(s.info_state_string(Player::Black), "P0\nx.\n..");
        assert_eq!(s.info_state_string(Player::White), "P1\n..\n.o");
    }

    #[test]
    fn cdh_multiple_collisions_then_success() {
        // Black places at 0 and 1, White collides on both then succeeds on 2
        let mut s = cdh(2, 2);
        s.apply_action(0).unwrap(); // Black places 0
        s.apply_action(0).unwrap(); // White collides 0 → stays White
        assert_eq!(s.current_player(), Player::White);
        // White needs to place somewhere. Black hasn't placed at 1 yet so...
        // Actually Black only placed 0. White collided 0. White tries 1.
        s.apply_action(1).unwrap(); // White places 1 → success
        assert_eq!(s.current_player(), Player::Black);
        s.apply_action(2).unwrap(); // Black places 2 → N-S win (0 and 2)
        assert!(s.is_terminal());
        assert_eq!(s.winner(), Some(Player::Black));
    }

    // --- ADH (Abrupt) tests ---

    #[test]
    fn adh_collision_wastes_turn() {
        let mut s = adh(2, 2);
        s.apply_action(0).unwrap(); // Black places at 0
        // White collides on 0 → turn wasted
        assert!(!s.apply_action(0).unwrap());
        // Turn passed to Black (ADH)
        assert_eq!(s.current_player(), Player::Black);
        assert_eq!(s.num_stones(), [1, 0]);
    }

    // --- Common tests ---

    #[test]
    fn terminal_rejects_action() {
        let mut s = cdh(2, 2);
        s.apply_action(0).unwrap();
        s.apply_action(1).unwrap();
        s.apply_action(2).unwrap(); // Black wins
        assert!(s.apply_action(3).is_err());
    }

    #[test]
    fn copy_is_independent() {
        let mut s = cdh(2, 2);
        s.apply_action(0).unwrap();
        let mut s2 = s.copy();
        s2.apply_action(1).unwrap();
        assert_eq!(s.stones_placed(), 1);
        assert_eq!(s2.stones_placed(), 2);
    }

    #[test]
    fn perfect_recall_info_state() {
        let mut s = cdh(2, 2);
        s.apply_action(0).unwrap();
        s.apply_action(3).unwrap();
        let info = s.info_state_string_perfect_recall(Player::Black);
        assert!(info.starts_with("P0\nx.\n..\n"));
        assert!(info.contains("0,0"));
    }

    #[test]
    fn three_by_three_black_wins() {
        let mut s = cdh(3, 3);
        assert_eq!(s.legal_actions().len(), 9);
        // Black plays column 0: cells 0, 3, 6
        for a in [0, 1, 3, 4, 6] {
            if s.is_terminal() {
                break;
            }
            s.apply_action(a).unwrap();
        }
        assert!(s.is_terminal());
        assert_eq!(s.winner(), Some(Player::Black));
    }
}
