pub mod ai_players;
pub mod game_logic;
pub mod ui;

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use crate::app_state::AppScreen;
use game_logic::GameConfig;

pub struct PlayPlugin;

impl Plugin for PlayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameConfig::default())
            .add_systems(
                EguiPrimaryContextPass,
                ui::play_ui.run_if(in_state(AppScreen::Play)),
            )
            .add_systems(
                Update,
                (ui::start_game_system, ui::handle_play_click)
                    .run_if(in_state(AppScreen::Play)),
            );
    }
}
