pub mod ai_players;
pub mod game_logic;
pub mod ui;

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use game_logic::GameConfig;

pub struct PlayPlugin;

impl Plugin for PlayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameConfig::default())
            .add_systems(EguiPrimaryContextPass, ui::play_ui)
            .add_systems(
                Update,
                (
                    ui::start_game_system,
                    ui::handle_play_click,
                    ui::update_cell_colors,
                ),
            );
    }
}
