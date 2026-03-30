pub mod tree_model;
pub mod ui;

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

pub struct ViewerPlugin;

impl Plugin for ViewerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ui::ViewerConfig::default())
            .add_systems(EguiPrimaryContextPass, ui::viewer_ui)
            .add_systems(
                Update,
                (ui::load_strategy_system, ui::update_viewer_visuals),
            );
    }
}
