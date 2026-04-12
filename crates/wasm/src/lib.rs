use wasm_bindgen::prelude::*;

use darkhex_core::game::info_state_ops;
use darkhex_core::game::state::DarkHexState;
use darkhex_core::game::types::Player;

/// WASM wrapper for Dark Hex game state (DSaGe web app).
#[wasm_bindgen]
pub struct GameState {
    inner: DarkHexState,
}

#[wasm_bindgen]
impl GameState {
    /// Create a new CDH game state.
    /// Validates that rows and cols are at least 1.
    #[wasm_bindgen(constructor)]
    pub fn new(rows: usize, cols: usize) -> Result<GameState, JsError> {
        if rows == 0 || cols == 0 {
            return Err(JsError::new("rows and cols must be >= 1"));
        }
        Ok(Self {
            inner: DarkHexState::new_cdh(rows, cols),
        })
    }

    pub fn rows(&self) -> usize {
        self.inner.rows()
    }

    pub fn cols(&self) -> usize {
        self.inner.cols()
    }

    /// Current player: 0 = Black, 1 = White.
    pub fn current_player(&self) -> u8 {
        self.inner.current_player().index() as u8
    }

    pub fn is_terminal(&self) -> bool {
        self.inner.is_terminal()
    }

    /// Winner: -1 = none, 0 = Black, 1 = White.
    pub fn winner(&self) -> i8 {
        match self.inner.winner() {
            None => -1,
            Some(p) => p.index() as i8,
        }
    }

    /// Legal actions (cell indices that appear empty to the current player).
    pub fn legal_actions(&self) -> Vec<usize> {
        self.inner.legal_actions()
    }

    /// Apply an action with bounds and legality checks.
    /// Returns true if stone placed, false if collision.
    pub fn apply_action(&mut self, action: usize) -> Result<bool, JsError> {
        self.inner
            .apply_action(action)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Imperfect-recall info state string for the given player.
    pub fn info_state_string(&self, player: u8) -> Result<String, JsError> {
        let p = to_player(player)?;
        Ok(self.inner.info_state_string(p))
    }

    /// Perfect-recall info state string.
    pub fn info_state_string_perfect_recall(&self, player: u8) -> Result<String, JsError> {
        let p = to_player(player)?;
        Ok(self.inner.info_state_string_perfect_recall(p))
    }

    /// Clone the game state.
    pub fn copy(&self) -> GameState {
        GameState {
            inner: self.inner.copy(),
        }
    }

    /// Player's view of the board as flat array.
    /// 0 = empty/hidden, 1 = black, 2 = white.
    pub fn player_view(&self, player: u8) -> Result<Vec<i8>, JsError> {
        let p = to_player(player)?;
        Ok(self.inner.player_view_flat(p))
    }

    /// True board state as flat array.
    /// 0 = empty, 1 = black, 2 = white.
    pub fn true_board(&self) -> Vec<i8> {
        self.inner.true_board_flat()
    }

    /// Returns [black_payoff, white_payoff].
    pub fn returns(&self) -> Vec<f64> {
        let r = self.inner.returns();
        vec![r[0], r[1]]
    }
}

/// Convert a u8 player index from JS to Player, returning JsError on invalid input.
fn to_player(player: u8) -> Result<Player, JsError> {
    Player::try_from_index(player as usize)
        .ok_or_else(|| JsError::new(&format!("invalid player index: {player} (expected 0 or 1)")))
}

// ---------------------------------------------------------------------------
// Info-state-level operations for the strategy generator
// ---------------------------------------------------------------------------

/// Stateless info state operations for the strategy generator.
///
/// Operates on info state *strings* — each method parses the string on every
/// call. The struct only stores board dimensions.
#[wasm_bindgen]
pub struct InfoStateOps {
    rows: usize,
    cols: usize,
}

#[wasm_bindgen]
impl InfoStateOps {
    #[wasm_bindgen(constructor)]
    pub fn new(rows: usize, cols: usize) -> Result<InfoStateOps, JsError> {
        if rows == 0 || cols == 0 {
            return Err(JsError::new("rows and cols must be >= 1"));
        }
        Ok(Self { rows, cols })
    }

    /// Initial info state for the given player: `"P0\n..\n.."`.
    pub fn initial_info_state(&self, player: u8) -> Result<String, JsError> {
        if player > 1 {
            return Err(JsError::new("player must be 0 or 1"));
        }
        Ok(info_state_ops::initial_info_state(
            self.rows,
            self.cols,
            player as usize,
        ))
    }

    /// Legal actions (cell indices where view shows '.').
    pub fn legal_actions(&self, info_state: &str) -> Result<Vec<usize>, JsError> {
        let parsed = self.parse(info_state)?;
        Ok(info_state_ops::legal_actions(&parsed))
    }

    /// Can collision occur at this info state?
    pub fn is_collision_possible(&self, info_state: &str) -> Result<bool, JsError> {
        let parsed = self.parse(info_state)?;
        Ok(info_state_ops::is_collision_possible(&parsed))
    }

    /// Successor info state after action.
    ///
    /// `stone_player` controls outcome:
    /// - same as info state player → successful placement
    /// - opponent → collision (player discovers opponent's stone)
    pub fn info_state_after_action(
        &self,
        info_state: &str,
        action: usize,
        stone_player: u8,
        perfect_recall: bool,
    ) -> Result<String, JsError> {
        let parsed = self.parse(info_state)?;
        info_state_ops::info_state_after_action(
            &parsed,
            action,
            stone_player as usize,
            perfect_recall,
        )
        .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Is this info state terminal?
    pub fn is_info_state_terminal(&self, info_state: &str) -> Result<bool, JsError> {
        let parsed = self.parse(info_state)?;
        Ok(info_state_ops::is_info_state_terminal(&parsed))
    }

    /// Board view as flat i8 array for rendering (0=empty, 1=black, 2=white).
    pub fn board_view_flat(&self, info_state: &str) -> Result<Vec<i8>, JsError> {
        let parsed = self.parse(info_state)?;
        Ok(info_state_ops::board_view_flat(&parsed))
    }

    /// Player index from info state string (0 or 1).
    pub fn player(&self, info_state: &str) -> Result<u8, JsError> {
        let parsed = self.parse(info_state)?;
        Ok(parsed.player as u8)
    }
}

impl InfoStateOps {
    fn parse(
        &self,
        info_state: &str,
    ) -> Result<info_state_ops::ParsedInfoState, JsError> {
        info_state_ops::parse_info_state(info_state, self.rows, self.cols)
            .map_err(|e| JsError::new(&e.to_string()))
    }
}
