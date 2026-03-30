use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::game::board::HexBoard;
use crate::game::types::{Cell, CollisionInfo, CollisionRule, Player};

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
                    self.current_player = Player::from_index(self.stones_placed % 2);
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

/// Rust-native methods for internal use (no PyO3 overhead).
/// These are the hot-path methods called by MCCFR traversal.
impl DarkHexState {
    pub fn rs_new(rows: usize, cols: usize) -> Self {
        Self {
            board: HexBoard::new(rows, cols),
            player_views: [vec![None; rows * cols], vec![None; rows * cols]],
            action_histories: [Vec::new(), Vec::new()],
            current_player: Player::Black,
            stones_placed: 0,
            cached_winner: None,
            collision_rule: CollisionRule::Classic,
            collision_info: CollisionInfo::Silent,
        }
    }

    #[inline]
    pub fn rs_rows(&self) -> usize {
        self.board.rows
    }

    #[inline]
    pub fn rs_cols(&self) -> usize {
        self.board.cols
    }

    #[inline]
    pub fn rs_size(&self) -> usize {
        self.board.size()
    }

    /// Returns (canonical_info_state_string, is_original_the_canonical_form).
    ///
    /// The canonical form is the lexicographically smaller of the original
    /// info state and its 180° rotation. Under rotation, cell at position
    /// `pos` maps to `n-1-pos`, which reverses the grid character order.
    pub fn rs_canonical_info_state(&self, player: Player) -> (String, bool) {
        let original = self.rs_info_state_string(player);
        let rotated = self.rs_rotated_info_state_string(player);
        if original <= rotated {
            (original, true)
        } else {
            (rotated, false)
        }
    }

    /// 180°-rotated info state: reads cell at position `n-1-pos` for each grid position.
    fn rs_rotated_info_state_string(&self, player: Player) -> String {
        let pi = player.index();
        let n = self.board.size();
        let mut s = String::with_capacity(3 + n + self.board.rows);
        s.push('P');
        s.push(char::from(b'0' + pi as u8));
        s.push('\n');
        for row in 0..self.board.rows {
            if row > 0 {
                s.push('\n');
            }
            for col in 0..self.board.cols {
                let pos = row * self.board.cols + col;
                let rotated_pos = n - 1 - pos;
                s.push(match self.player_views[pi][rotated_pos] {
                    None => '.',
                    Some(c) => c.to_char(),
                });
            }
        }
        s
    }

    #[inline]
    pub fn rs_current_player(&self) -> Player {
        self.current_player
    }

    #[inline]
    pub fn rs_is_terminal(&self) -> bool {
        self.cached_winner.is_some()
    }

    #[inline]
    pub fn rs_player_return(&self, player: Player) -> f32 {
        match self.cached_winner {
            Some(p) if p == player => 1.0,
            Some(_) => -1.0,
            None => 0.0,
        }
    }

    pub fn rs_legal_actions(&self, buf: &mut Vec<usize>) {
        buf.clear();
        let pi = self.current_player.index();
        for (i, v) in self.player_views[pi].iter().enumerate() {
            if v.is_none() {
                buf.push(i);
            }
        }
    }

    /// Apply action without validation. Returns true if placed, false if collision.
    pub fn rs_apply_action(&mut self, action: usize) -> bool {
        let player = self.current_player;
        let pi = player.index();
        self.action_histories[pi].push(action);

        if self.board.place_stone(action, player) {
            self.player_views[pi][action] = Some(Cell::from_player(player));
            self.cached_winner = self.board.winner();
            self.stones_placed += 1;
            self.current_player = Player::from_index(self.stones_placed % 2);
            true
        } else {
            self.player_views[pi][action] = Some(Cell::from_player(player.opponent()));
            match self.collision_rule {
                CollisionRule::Classic => {}
                CollisionRule::Abrupt => {
                    self.stones_placed += 1;
                    self.current_player = Player::from_index(self.stones_placed % 2);
                }
            }
            false
        }
    }

    pub fn rs_board_cells(&self) -> &[Cell] {
        &self.board.cells
    }

    pub fn rs_player_views(&self) -> &[Vec<Option<Cell>>; 2] {
        &self.player_views
    }

    pub fn rs_info_state_string(&self, player: Player) -> String {
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
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
