use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass};

mod app_state;
mod builder;
mod common;
mod hex_board;
mod play;
mod theme;
mod viewer;

use app_state::AppScreen;

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
        .add_plugins(builder::BuilderPlugin)
        .add_plugins(viewer::ViewerPlugin)
        .init_state::<AppScreen>()
        .init_state::<app_state::RenderMode>()
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, navigation_bar)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn navigation_bar(
    mut contexts: EguiContexts,
    current_screen: Res<State<AppScreen>>,
    mut next_screen: ResMut<NextState<AppScreen>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    bevy_egui::egui::TopBottomPanel::top("nav_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("DarkHex");
            ui.separator();
            let tabs = [
                (AppScreen::Play, "Play"),
                (AppScreen::StrategyBuilder, "Build Strategy"),
                (AppScreen::StrategyViewer, "View Strategy"),
            ];
            for (screen, label) in tabs {
                if ui
                    .selectable_label(*current_screen.get() == screen, label)
                    .clicked()
                {
                    next_screen.set(screen);
                }
            }
        });
    });
}
