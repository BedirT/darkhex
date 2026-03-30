use bevy::prelude::*;

use darkhex_core::game::state::DarkHexState;
use darkhex_core::game::types::{CollisionInfo, CollisionRule, Player};

use super::ai_players::{self, AIPlayer, AIType};

/// A move entry in the game log.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct MoveEntry {
    pub player: Player,
    pub action: usize,
    pub placed: bool,
    pub label: String,
}

/// Active game session.
#[allow(dead_code)]
#[derive(Resource)]
pub struct GameSession {
    pub state: DarkHexState,
    pub human_player: Player,
    pub ai: Box<dyn AIPlayer>,
    pub move_log: Vec<MoveEntry>,
    pub game_over: bool,
    pub winner: Option<Player>,
    pub last_collision: Option<usize>,
    pub rows: usize,
    pub cols: usize,
}

/// Configuration for starting a new game.
#[derive(Resource)]
pub struct GameConfig {
    pub rows: usize,
    pub cols: usize,
    pub human_player: Player,
    pub ai_type: AIType,
    pub collision_rule: CollisionRule,
    pub collision_info: CollisionInfo,
    pub start_requested: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            rows: 2,
            cols: 2,
            human_player: Player::Black,
            ai_type: AIType::Random,
            collision_rule: CollisionRule::Classic,
            collision_info: CollisionInfo::Silent,
            start_requested: false,
        }
    }
}

fn pos_to_label(pos: usize, cols: usize) -> String {
    let row = pos / cols;
    let col = pos % cols;
    let row_char = (b'a' + row as u8) as char;
    format!("{}{}", row_char, col + 1)
}

impl GameSession {
    pub fn new(config: &GameConfig) -> Self {
        let state = DarkHexState::new(
            config.rows,
            config.cols,
            Some(config.collision_rule),
            Some(config.collision_info),
        )
        .expect("valid board dimensions");

        let ai = ai_players::create_ai(
            config.ai_type,
            config.rows,
            config.cols,
            config.human_player.opponent(),
        );

        Self {
            state,
            human_player: config.human_player,
            ai,
            move_log: Vec::new(),
            game_over: false,
            winner: None,
            last_collision: None,
            rows: config.rows,
            cols: config.cols,
        }
    }

    /// Process a human move. Returns true if the move was accepted.
    pub fn human_move(&mut self, action: usize) -> bool {
        if self.game_over {
            return false;
        }
        if self.state.current_player() != self.human_player {
            return false;
        }

        self.last_collision = None;

        match self.state.apply_action(action) {
            Ok(placed) => {
                let label = pos_to_label(action, self.cols);
                self.move_log.push(MoveEntry {
                    player: self.human_player,
                    action,
                    placed,
                    label: label.clone(),
                });

                if !placed {
                    self.last_collision = Some(action);
                    info!("Collision at {label}!");
                }

                if self.state.is_terminal() {
                    self.game_over = true;
                    self.winner = self.state.winner();
                    return true;
                }

                // If turn passed to AI (placed in CDH, or always in ADH), let AI play
                if self.state.current_player() != self.human_player {
                    self.ai_move();
                }

                true
            }
            Err(_) => false,
        }
    }

    /// Let the AI take its turn(s).
    fn ai_move(&mut self) {
        // AI may need multiple moves if collision happens in CDH
        while !self.game_over && self.state.current_player() != self.human_player {
            let info = self
                .state
                .info_state_string(self.state.current_player());
            let legal = self.state.legal_actions();
            if legal.is_empty() {
                break;
            }

            let action = self.ai.select_action(&info, &legal);
            match self.state.apply_action(action) {
                Ok(placed) => {
                    let label = pos_to_label(action, self.cols);
                    self.move_log.push(MoveEntry {
                        player: self.human_player.opponent(),
                        action,
                        placed,
                        label,
                    });

                    if self.state.is_terminal() {
                        self.game_over = true;
                        self.winner = self.state.winner();
                    }
                }
                Err(e) => {
                    warn!("AI made invalid action {action}: {e}");
                    break;
                }
            }
        }
    }

    /// Get what the human player can see (for board rendering).
    pub fn human_view(&self) -> Vec<char> {
        let info = self.state.info_state_string(self.human_player);
        // Format: "P{id}\n{rows...}" — extract the grid characters
        info.lines()
            .skip(1)
            .flat_map(|line| line.chars())
            .collect()
    }

    pub fn status_text(&self) -> String {
        if self.game_over {
            match self.winner {
                Some(Player::Black) => "Black wins!".to_string(),
                Some(Player::White) => "White wins!".to_string(),
                None => "Draw (shouldn't happen in Hex)".to_string(),
            }
        } else if self.state.current_player() == self.human_player {
            "Your turn — click a cell".to_string()
        } else {
            "AI is thinking...".to_string()
        }
    }
}
