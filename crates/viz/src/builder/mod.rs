pub mod engine;
pub mod ui;

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

pub struct BuilderPlugin;

impl Plugin for BuilderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ui::BuilderConfig::default())
            .add_systems(EguiPrimaryContextPass, ui::builder_ui)
            .add_systems(
                Update,
                (
                    ui::start_builder_system,
                    ui::handle_builder_click,
                    ui::update_builder_visuals,
                ),
            );
    }
}
