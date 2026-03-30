use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, FontId, RichText, Stroke, TextStyle};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass};

mod app_state;
mod builder;
mod common;
mod hex_board;
mod play;
mod theme;
mod viewer;

use app_state::AppScreen;
use hex_board::research_renderer::BoardSizeRequest;
use play::game_logic::GameSession;

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
        .insert_resource(ClearColor(theme::CLEAR_COLOR))
        .add_systems(Startup, setup)
        .add_systems(
            EguiPrimaryContextPass,
            (
                configure_egui_style,
                navigation_bar,
                status_bar,
            )
                .chain(),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Apply the DarkHex egui theme on every frame (idempotent).
fn configure_egui_style(mut contexts: EguiContexts) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let mut style = (*ctx.style()).clone();

    // Slightly larger default fonts
    style.text_styles.insert(
        TextStyle::Body,
        FontId::proportional(15.0),
    );
    style.text_styles.insert(
        TextStyle::Button,
        FontId::proportional(15.0),
    );
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::proportional(22.0),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::proportional(12.0),
    );

    // Panel and window fills
    style.visuals.panel_fill = theme::PANEL_BG;
    style.visuals.window_fill = theme::WINDOW_BG;
    style.visuals.window_stroke = Stroke::new(1.0, theme::ACCENT_DIM);
    style.visuals.window_shadow = egui::Shadow::NONE;
    style.visuals.extreme_bg_color = Color32::from_rgb(18, 20, 28);
    style.visuals.faint_bg_color = Color32::from_rgb(35, 38, 50);
    style.visuals.code_bg_color = Color32::from_rgb(35, 38, 50);

    // Selection accent
    style.visuals.selection.bg_fill = theme::ACCENT_DIM;
    style.visuals.selection.stroke = Stroke::new(1.0, theme::ACCENT);

    // Hyperlink color
    style.visuals.hyperlink_color = theme::ACCENT;

    // Widget styling — rounded corners, accent-tinted
    let corner_radius = egui::CornerRadius::same(6);

    // Noninteractive (labels, panel outlines)
    style.visuals.widgets.noninteractive.bg_fill = theme::PANEL_BG;
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(0.5, Color32::from_rgb(50, 55, 70));
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, theme::TEXT_PRIMARY);
    style.visuals.widgets.noninteractive.corner_radius = corner_radius;

    // Inactive (buttons at rest)
    style.visuals.widgets.inactive.bg_fill = theme::WIDGET_BG;
    style.visuals.widgets.inactive.weak_bg_fill = theme::WIDGET_BG;
    style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, theme::TEXT_PRIMARY);
    style.visuals.widgets.inactive.corner_radius = corner_radius;

    // Hovered
    style.visuals.widgets.hovered.bg_fill = theme::WIDGET_BG_HOVER;
    style.visuals.widgets.hovered.weak_bg_fill = theme::WIDGET_BG_HOVER;
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, theme::ACCENT);
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    style.visuals.widgets.hovered.corner_radius = corner_radius;

    // Active (pressed)
    style.visuals.widgets.active.bg_fill = theme::WIDGET_BG_ACTIVE;
    style.visuals.widgets.active.weak_bg_fill = theme::WIDGET_BG_ACTIVE;
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, theme::ACCENT_HOVER);
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    style.visuals.widgets.active.corner_radius = corner_radius;

    // Open (combo boxes, menus)
    style.visuals.widgets.open.bg_fill = theme::WIDGET_BG_ACTIVE;
    style.visuals.widgets.open.weak_bg_fill = theme::WIDGET_BG_ACTIVE;
    style.visuals.widgets.open.bg_stroke = Stroke::new(1.0, theme::ACCENT);
    style.visuals.widgets.open.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    style.visuals.widgets.open.corner_radius = corner_radius;

    // Spacing
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 4.0);

    ctx.set_style(style);
}

fn navigation_bar(
    mut contexts: EguiContexts,
    current_screen: Res<State<AppScreen>>,
    mut next_screen: ResMut<NextState<AppScreen>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::TopBottomPanel::top("nav_bar")
        .frame(egui::Frame::new().fill(theme::PANEL_BG).inner_margin(egui::Margin::symmetric(12, 8)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // App title
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.label(
                        RichText::new("DarkHex")
                            .size(24.0)
                            .strong()
                            .color(theme::ACCENT_HOVER),
                    );
                    ui.label(
                        RichText::new("Research Toolkit")
                            .size(11.0)
                            .color(theme::TEXT_DIM),
                    );
                });

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(12.0);

                // Tab bar
                let tabs = [
                    (AppScreen::Play, "Play"),
                    (AppScreen::StrategyBuilder, "Build Strategy"),
                    (AppScreen::StrategyViewer, "View Strategy"),
                ];
                let active = current_screen.get();

                for (screen, label) in tabs {
                    let is_active = *active == screen;

                    // Draw the tab as a styled selectable label
                    let text = if is_active {
                        RichText::new(label).strong().color(Color32::WHITE)
                    } else {
                        RichText::new(label).color(theme::TEXT_DIM)
                    };

                    let response = ui.selectable_label(is_active, text);

                    // Draw underline for active tab
                    if is_active {
                        let rect = response.rect;
                        let painter = ui.painter();
                        painter.line_segment(
                            [
                                egui::pos2(rect.left() + 2.0, rect.bottom() + 1.0),
                                egui::pos2(rect.right() - 2.0, rect.bottom() + 1.0),
                            ],
                            Stroke::new(2.0, theme::ACCENT),
                        );
                    }

                    if response.clicked() {
                        next_screen.set(screen);
                    }

                    ui.add_space(4.0);
                }
            });
        });
}

fn status_bar(
    mut contexts: EguiContexts,
    board_req: Res<BoardSizeRequest>,
    current_screen: Res<State<AppScreen>>,
    session: Option<Res<GameSession>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::TopBottomPanel::bottom("status_bar")
        .frame(
            egui::Frame::new()
                .fill(theme::PANEL_BG)
                .inner_margin(egui::Margin::symmetric(12, 4))
                .stroke(Stroke::new(0.5, Color32::from_rgb(50, 55, 70))),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Board size
                ui.label(
                    RichText::new(format!("Board: {}x{}", board_req.rows, board_req.cols))
                        .small()
                        .color(theme::TEXT_DIM),
                );

                ui.separator();

                // Session info
                let session_text = match (current_screen.get(), &session) {
                    (AppScreen::Play, Some(s)) => {
                        if s.game_over {
                            match s.winner {
                                Some(w) => format!("Game over -- {:?} wins", w),
                                None => "Game over -- draw".to_string(),
                            }
                        } else {
                            format!("Playing as {:?} | Move {}", s.human_player, s.move_log.len())
                        }
                    }
                    (AppScreen::Play, None) => "No active game".to_string(),
                    (AppScreen::StrategyBuilder, _) => "Strategy Builder".to_string(),
                    (AppScreen::StrategyViewer, _) => "Strategy Viewer".to_string(),
                };
                ui.label(RichText::new(session_text).small().color(theme::TEXT_DIM));

                // Right-aligned hint
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("Tab: switch screens")
                            .small()
                            .color(Color32::from_rgb(80, 88, 110)),
                    );
                });
            });
        });
}
