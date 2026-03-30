use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass};

mod app_state;
mod hex_board;
mod theme;

use hex_board::research_renderer::BoardSizeRequest;

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
        .init_state::<app_state::AppScreen>()
        .init_state::<app_state::RenderMode>()
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, ui_sidebar)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn ui_sidebar(mut contexts: EguiContexts, mut board_req: ResMut<BoardSizeRequest>) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    bevy_egui::egui::SidePanel::left("controls")
        .default_width(160.0)
        .show(ctx, |ui| {
            ui.heading("DarkHex");
            ui.separator();
            ui.label("Board Size:");
            let sizes = [(2, 2), (3, 2), (3, 3), (4, 3)];
            for (r, c) in sizes {
                let label = format!("{}x{}", r, c);
                let selected = board_req.rows == r && board_req.cols == c;
                if ui.selectable_label(selected, label).clicked() && !selected {
                    board_req.rows = r;
                    board_req.cols = c;
                    board_req.changed = true;
                }
            }
            ui.separator();
            ui.label("Click hex cells to test");
        });
}
