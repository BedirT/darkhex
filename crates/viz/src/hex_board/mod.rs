pub mod components;
pub mod research_renderer;

use bevy::prelude::*;

use components::HexClickEvent;
use research_renderer::{handle_board_resize, handle_hex_click, BoardSizeRequest};

pub struct HexBoardPlugin;

impl Plugin for HexBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<HexClickEvent>()
            .insert_resource(BoardSizeRequest {
                rows: 2,
                cols: 2,
                changed: true,
            })
            .add_systems(
                Update,
                (
                    handle_board_resize,
                    handle_hex_click,
                    crate::play::ui::update_cell_colors,
                ),
            );
    }
}
