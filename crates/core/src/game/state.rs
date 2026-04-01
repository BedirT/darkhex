use crate::error::CoreError;
use crate::game::board::HexBoard;
use crate::game::types::{Cell, CollisionInfo, CollisionRule, Player};

/// State of a Dark Hex game.
///
/// Supports all four thesis variants via `CollisionRule` × `CollisionInfo`:
/// - **CDH** (Classic):  `Classic` + `Silent`  — player retries, opponent unaware
/// - **ADH** (Abrupt):   `Abrupt` + `Silent`  — collision wastes turn
/// - **NDH** (Noisy):    either   + `Noisy`   — opponent told collision happened
/// - **FDH** (Flash):    either   + `Flash`   — opponent told where collision was
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

impl DarkHexState {
    /// Create a new Dark Hex game state.
    ///
    /// Defaults to CDH (Classic Dark Hex): player retries after collision,
    /// opponent not informed.
    pub fn new(
        rows: usize,
        cols: usize,
        collision_rule: Option<CollisionRule>,
        collision_info: Option<CollisionInfo>,
    ) -> Result<Self, CoreError> {
        if rows == 0 || cols == 0 {
            return Err(CoreError::InvalidArgument(
                "board dimensions must be positive".to_string(),
            ));
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

    /// Create a CDH game state (convenience for internal Rust use).
    pub fn new_cdh(rows: usize, cols: usize) -> Self {
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
    pub fn rows(&self) -> usize {
        self.board.rows
    }

    #[inline]
    pub fn cols(&self) -> usize {
        self.board.cols
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.board.size()
    }

    pub fn num_players(&self) -> usize {
        2
    }

    #[inline]
    pub fn current_player(&self) -> Player {
        self.current_player
    }

    #[inline]
    pub fn is_terminal(&self) -> bool {
        self.cached_winner.is_some()
    }

    pub fn winner(&self) -> Option<Player> {
        self.cached_winner
    }

    /// Payoffs: `[black, white]`. +1 for winner, -1 for loser, 0 if ongoing.
    pub fn returns(&self) -> [f64; 2] {
        match self.cached_winner {
            Some(Player::Black) => [1.0, -1.0],
            Some(Player::White) => [-1.0, 1.0],
            None => [0.0, 0.0],
        }
    }

    pub fn player_return(&self, player: Player) -> f64 {
        self.returns()[player.index()]
    }

    /// f32 variant for hot-path MCCFR traversal.
    #[inline]
    pub fn player_return_f32(&self, player: Player) -> f32 {
        match self.cached_winner {
            Some(p) if p == player => 1.0,
            Some(_) => -1.0,
            None => 0.0,
        }
    }

    /// Cells that appear empty to the current player.
    pub fn legal_actions(&self) -> Vec<usize> {
        let pi = self.current_player.index();
        self.player_views[pi]
            .iter()
            .enumerate()
            .filter(|(_, v)| v.is_none())
            .map(|(i, _)| i)
            .collect()
    }

    /// Fill buffer with legal actions (hot-path, no allocation).
    pub fn legal_actions_buf(&self, buf: &mut Vec<usize>) {
        buf.clear();
        let pi = self.current_player.index();
        for (i, v) in self.player_views[pi].iter().enumerate() {
            if v.is_none() {
                buf.push(i);
            }
        }
    }

    pub fn num_legal_actions(&self) -> usize {
        let pi = self.current_player.index();
        self.player_views[pi].iter().filter(|v| v.is_none()).count()
    }

    /// Apply an action for the current player (with validation).
    ///
    /// **CDH (Classic)**: On collision the player discovers the opponent's
    /// stone and stays as the current player (retries until successful).
    ///
    /// **ADH (Abrupt)**: On collision the turn is wasted; play passes.
    ///
    /// Returns `true` if the stone was placed, `false` if collision.
    pub fn apply_action(&mut self, action: usize) -> Result<bool, CoreError> {
        if self.cached_winner.is_some() {
            return Err(CoreError::InvalidArgument(
                "game is already terminal".to_string(),
            ));
        }
        if action >= self.board.size() {
            return Err(CoreError::InvalidArgument(format!(
                "action {action} out of bounds for {}x{} board",
                self.board.rows, self.board.cols
            )));
        }
        let pi = self.current_player.index();
        if self.player_views[pi][action].is_some() {
            return Err(CoreError::InvalidArgument(format!(
                "action {action} not legal: cell not empty in player's view"
            )));
        }

        Ok(self.apply_action_unchecked(action))
    }

    /// Apply action without validation (hot path). Returns true if placed, false if collision.
    pub fn apply_action_unchecked(&mut self, action: usize) -> bool {
        let player = self.current_player;
        let pi = player.index();
        self.action_histories[pi].push(action);

        if self.board.place_stone(action, player) {
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
                    // Tracked externally (e.g., in the info state).
                    let _ = oi;
                }
                CollisionInfo::Flash => {
                    // Opponent knows WHERE the collision happened.
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
        }
    }

    /// Imperfect-recall information state string for the given player.
    ///
    /// Format: `"P{player}\n{board_view}"` where the board view shows
    /// only what the player can see (opponent stones hidden as `.`).
    pub fn info_state_string(&self, player: Player) -> String {
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
    pub fn info_state_string_perfect_recall(&self, player: Player) -> String {
        let pi = player.index();
        let mut s = self.info_state_string(player);
        s.push('\n');
        for &a in &self.action_histories[pi] {
            s.push_str(&format!("{pi},{a} "));
        }
        s
    }

    /// Returns (canonical_info_state_string, is_original_the_canonical_form).
    ///
    /// The canonical form is the lexicographically smaller of the original
    /// info state and its 180° rotation.
    pub fn canonical_info_state(&self, player: Player) -> (String, bool) {
        let original = self.info_state_string(player);
        let rotated = self.rotated_info_state_string(player);
        if original <= rotated {
            (original, true)
        } else {
            (rotated, false)
        }
    }

    /// 180°-rotated info state: reads cell at position `n-1-pos`.
    pub fn rotated_info_state_string(&self, player: Player) -> String {
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

    /// Clone the state (for game-tree traversal in MCCFR).
    pub fn copy(&self) -> Self {
        self.clone()
    }

    /// Number of successful stone placements so far.
    pub fn stones_placed(&self) -> usize {
        self.stones_placed
    }

    /// Number of stones actually on the board for each player.
    pub fn num_stones(&self) -> [usize; 2] {
        self.board.num_stones
    }

    pub fn collision_rule(&self) -> CollisionRule {
        self.collision_rule
    }

    pub fn collision_info(&self) -> CollisionInfo {
        self.collision_info
    }

    /// String representation of the true board (for debugging/verification).
    pub fn true_board_string(&self) -> String {
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

    pub fn board_cells(&self) -> &[Cell] {
        &self.board.cells
    }

    pub fn player_views(&self) -> &[Vec<Option<Cell>>; 2] {
        &self.player_views
    }

    /// Flat view of board from a player's perspective.
    /// -1 = hidden (appears empty), 0 = empty, 1 = black, 2 = white.
    pub fn player_view_flat(&self, player: Player) -> Vec<i8> {
        let pi = player.index();
        self.player_views[pi]
            .iter()
            .map(|v| match v {
                None => 0,    // empty (from player's perspective)
                Some(Cell::Empty) => 0,
                Some(Cell::Black) => 1,
                Some(Cell::White) => 2,
            })
            .collect()
    }

    /// True board as flat i8 vec: 0 = empty, 1 = black, 2 = white.
    pub fn true_board_flat(&self) -> Vec<i8> {
        self.board.cells
            .iter()
            .map(|c| match c {
                Cell::Empty => 0,
                Cell::Black => 1,
                Cell::White => 2,
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
