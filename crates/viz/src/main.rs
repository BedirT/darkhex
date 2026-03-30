use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod app_state;
mod hex_board;
mod play;
mod theme;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "DarkHex".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins(hex_board::HexBoardPlugin)
        .add_plugins(play::PlayPlugin)
        .init_state::<app_state::AppScreen>()
        .init_state::<app_state::RenderMode>()
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
