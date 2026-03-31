pub mod components;
pub mod game_renderer;
pub mod research_renderer;

use bevy::prelude::*;
use hexx::Hex;

use crate::app_state::RenderMode;
use components::HexClickEvent;
use research_renderer::BoardSizeRequest;

/// Convert darkhex (row, col) to hexx axial Hex coordinate.
///
/// DarkHex uses an offset-row layout where each row shifts right.
/// The neighbor topology in board.rs:
///   E=(r,c+1), W=(r,c-1), SW=(r+1,c), SE=(r+1,c-1), NE=(r-1,c), NW=(r-1,c+1)
///
/// This maps to axial coordinates: q = col, r = row (with appropriate offset).
pub fn darkhex_to_hex(row: usize, col: usize) -> Hex {
    let q = col as i32 - (row as i32) / 2;
    let r = row as i32;
    Hex::new(q, r)
}

/// Convert linear position to alphanumeric label (a1, b2, etc.).
pub fn pos_to_label(pos: usize, cols: usize) -> String {
    let row = pos / cols;
    let col = pos % cols;
    let row_char = (b'a' + row as u8) as char;
    format!("{}{}", row_char, col + 1)
}

pub struct HexBoardPlugin;

impl Plugin for HexBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<HexClickEvent>()
            .insert_resource(BoardSizeRequest {
                rows: 2,
                cols: 2,
                changed: true,
            })
            // Research mode systems
            .add_systems(
                Update,
                (
                    research_renderer::handle_board_resize,
                    research_renderer::handle_hex_click,
                    crate::play::ui::update_cell_colors,
                )
                    .run_if(in_state(RenderMode::Research)),
            )
            // Game mode systems
            .add_systems(
                Update,
                (
                    game_renderer::handle_game_board_resize,
                    game_renderer::handle_game_hex_click,
                    game_renderer::update_game_cell_visuals,
                )
                    .run_if(in_state(RenderMode::Game)),
            )
            // Cleanup on mode exit
            .add_systems(
                OnExit(RenderMode::Research),
                research_renderer::despawn_research_board,
            )
            .add_systems(
                OnExit(RenderMode::Game),
                game_renderer::despawn_game_board,
            );
    }
}
