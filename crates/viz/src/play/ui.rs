use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use darkhex_core::game::types::{CollisionRule, Player};

use super::ai_players::AIType;
use super::game_logic::{GameConfig, GameSession};
use crate::hex_board::components::{CellState, HexCell, HexClickEvent};
use crate::hex_board::research_renderer::BoardSizeRequest;

/// Format the game status as colored `RichText`.
fn status_rich_text(session: &GameSession) -> egui::RichText {
    if session.game_over {
        let label = match session.winner {
            Some(Player::Black) => "Black wins!",
            Some(Player::White) => "White wins!",
            None => "Draw",
        };
        egui::RichText::new(label)
            .size(18.0)
            .strong()
            .color(egui::Color32::from_rgb(255, 215, 0))
    } else if session.state.current_player() == session.human_player {
        egui::RichText::new("Your turn — click a cell")
            .size(16.0)
            .strong()
            .color(egui::Color32::from_rgb(80, 200, 80))
    } else {
        egui::RichText::new("AI is thinking...")
            .size(16.0)
            .strong()
            .color(egui::Color32::from_rgb(230, 200, 50))
    }
}

/// Short label for the active variant.
fn variant_label(rule: CollisionRule) -> &'static str {
    match rule {
        CollisionRule::Classic => "CDH",
        CollisionRule::Abrupt => "ADH",
    }
}

/// System: egui sidebar for the Play screen.
pub fn play_ui(
    mut contexts: EguiContexts,
    mut config: ResMut<GameConfig>,
    session: Option<Res<GameSession>>,
    mut board_req: ResMut<BoardSizeRequest>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::SidePanel::left("play_controls")
        .default_width(220.0)
        .show(ctx, |ui| {
            ui.heading("Play");
            ui.separator();

            // -- Game status (prominent, at the top) --
            if let Some(ref session) = session {
                ui.add_space(2.0);
                ui.label(status_rich_text(session));

                if let Some(collision_pos) = session.last_collision {
                    let label = pos_to_label_ui(collision_pos, session.cols);
                    ui.label(
                        egui::RichText::new(format!("Collision at {label}!"))
                            .color(egui::Color32::from_rgb(220, 60, 40)),
                    );
                }

                // Compact game info line
                ui.add_space(2.0);
                let info_line = format!(
                    "{}x{}  {}  vs {}",
                    session.rows,
                    session.cols,
                    variant_label(config.collision_rule),
                    session.ai.display_name(),
                );
                ui.label(egui::RichText::new(info_line).weak().italics());
                ui.add_space(4.0);
                ui.separator();
            }

            // -- Settings --
            ui.add_space(4.0);
            ui.label(egui::RichText::new("Settings").strong().size(14.0));
            ui.add_space(2.0);

            // Board size — horizontal button row
            ui.horizontal(|ui| {
                ui.label("Board:");
                let sizes: &[(usize, usize)] = &[(2, 2), (3, 2), (3, 3), (4, 3)];
                for &(r, c) in sizes {
                    let label = format!("{}x{}", r, c);
                    let selected = config.rows == r && config.cols == c;
                    if ui.selectable_label(selected, label).clicked() && !selected {
                        config.rows = r;
                        config.cols = c;
                    }
                }
            });

            // Play as — horizontal
            ui.horizontal(|ui| {
                ui.label("Play as:");
                if ui
                    .selectable_label(config.human_player == Player::Black, "Black")
                    .clicked()
                {
                    config.human_player = Player::Black;
                }
                if ui
                    .selectable_label(config.human_player == Player::White, "White")
                    .clicked()
                {
                    config.human_player = Player::White;
                }
            });

            // Variant — horizontal
            ui.horizontal(|ui| {
                ui.label("Variant:");
                if ui
                    .selectable_label(
                        config.collision_rule == CollisionRule::Classic,
                        "CDH",
                    )
                    .on_hover_text("Classic: retry on collision")
                    .clicked()
                {
                    config.collision_rule = CollisionRule::Classic;
                }
                if ui
                    .selectable_label(
                        config.collision_rule == CollisionRule::Abrupt,
                        "ADH",
                    )
                    .on_hover_text("Abrupt: turn wasted on collision")
                    .clicked()
                {
                    config.collision_rule = CollisionRule::Abrupt;
                }
            });
            ui.add_space(2.0);

            // Opponent
            ui.collapsing("Opponent", |ui| {
                for &ai in AIType::ALL {
                    if ui
                        .selectable_label(config.ai_type == ai, ai.label())
                        .clicked()
                    {
                        config.ai_type = ai;
                    }
                }
            });

            ui.add_space(6.0);

            // New Game button — prominent, full width
            let btn = egui::Button::new(
                egui::RichText::new("New Game").strong().size(15.0),
            );
            if ui
                .add_sized([ui.available_width(), 32.0], btn)
                .clicked()
            {
                config.start_requested = true;
                board_req.rows = config.rows;
                board_req.cols = config.cols;
                board_req.changed = true;
            }

            ui.add_space(4.0);
            ui.separator();

            // -- Move log --
            if let Some(ref session) = session {
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Move Log").strong().size(14.0));
                ui.add_space(2.0);

                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for (i, entry) in session.move_log.iter().enumerate() {
                            let num = i + 1;
                            let (marker, color) = match entry.player {
                                Player::Black => (
                                    "B",
                                    egui::Color32::from_rgb(160, 160, 160),
                                ),
                                Player::White => (
                                    "W",
                                    egui::Color32::from_rgb(230, 220, 200),
                                ),
                            };
                            let suffix =
                                if entry.placed { "" } else { " (collision)" };
                            ui.label(
                                egui::RichText::new(format!(
                                    "{num:>2}. [{marker}] {}{suffix}",
                                    entry.label,
                                ))
                                .color(color)
                                .monospace(),
                            );
                        }
                    });
            } else {
                ui.add_space(20.0);
                ui.label("Press 'New Game' to start.");
            }
        });
}

fn pos_to_label_ui(pos: usize, cols: usize) -> String {
    let row = pos / cols;
    let col = pos % cols;
    let row_char = (b'a' + row as u8) as char;
    format!("{}{}", row_char, col + 1)
}

/// System: start a new game when requested.
pub fn start_game_system(
    mut commands: Commands,
    mut config: ResMut<GameConfig>,
) {
    if !config.start_requested {
        return;
    }
    config.start_requested = false;

    info!(
        "Starting new game: {}x{}, human={:?}, ai={:?}, variant={:?}",
        config.rows, config.cols, config.human_player, config.ai_type, config.collision_rule
    );

    let mut session = GameSession::new(&config);

    // If human is White (moves second), AI goes first
    if session.human_player == Player::White && !session.game_over {
        // AI needs to play first
        let info = session.state.info_state_string(session.state.current_player());
        let legal = session.state.legal_actions();
        if !legal.is_empty() {
            let action = session.ai.select_action(&info, &legal);
            let _ = session.state.apply_action(action);
            session.move_log.push(super::game_logic::MoveEntry {
                player: Player::Black,
                action,
                placed: true,
                label: pos_to_label_ui(action, session.cols),
            });
            if session.state.is_terminal() {
                session.game_over = true;
                session.winner = session.state.winner();
            }
        }
    }

    commands.insert_resource(session);
}

/// System: handle clicks on hex cells during gameplay.
pub fn handle_play_click(
    mut click_events: MessageReader<HexClickEvent>,
    mut session: Option<ResMut<GameSession>>,
    mut cell_query: Query<(&HexCell, &mut CellState)>,
) {
    let Some(ref mut session) = session else { return };

    for event in click_events.read() {
        if session.game_over {
            continue;
        }
        if session.state.current_player() != session.human_player {
            continue;
        }

        let accepted = session.human_move(event.pos);
        if accepted {
            // Update cell visuals based on human's view
            update_cell_visuals(session.as_ref(), &mut cell_query);
        }
    }
}

/// Update all cell visual states from the game session's human view.
fn update_cell_visuals(
    session: &GameSession,
    cell_query: &mut Query<(&HexCell, &mut CellState)>,
) {
    let view = session.human_view();
    for (cell, mut state) in cell_query.iter_mut() {
        if cell.pos < view.len() {
            *state = match view[cell.pos] {
                'x' => CellState::Black,
                'o' => CellState::White,
                _ => CellState::Empty,
            };
        }
    }
}

/// System: update cell material colors based on CellState changes.
pub fn update_cell_colors(
    mut materials: ResMut<Assets<ColorMaterial>>,
    changed_cells: Query<(&CellState, &MeshMaterial2d<ColorMaterial>), Changed<CellState>>,
) {
    for (state, mat_handle) in changed_cells.iter() {
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = match state {
                CellState::Empty => crate::theme::EMPTY_CELL,
                CellState::Black => crate::theme::BLACK_STONE,
                CellState::White => crate::theme::WHITE_STONE,
            };
        }
    }
}
