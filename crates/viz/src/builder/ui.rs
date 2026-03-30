use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use darkhex_core::game::types::Player;

use super::engine::StrategyBuilder;
use crate::common::info_state;
use crate::hex_board::components::{CellState, HexCell, HexClickEvent};
use crate::hex_board::research_renderer::BoardSizeRequest;

/// Builder configuration.
#[derive(Resource)]
pub struct BuilderConfig {
    pub rows: usize,
    pub cols: usize,
    pub player: usize,
    pub start_requested: bool,
}

impl Default for BuilderConfig {
    fn default() -> Self {
        Self {
            rows: 2,
            cols: 2,
            player: 0,
            start_requested: false,
        }
    }
}

/// Active builder session.
#[derive(Resource)]
pub struct BuilderSession {
    pub builder: StrategyBuilder,
    pub action_input: String,
    pub error_msg: Option<String>,
    pub complete: bool,
}

/// System: egui sidebar for strategy builder.
pub fn builder_ui(
    mut contexts: EguiContexts,
    mut config: ResMut<BuilderConfig>,
    session: Option<ResMut<BuilderSession>>,
    mut board_req: ResMut<BoardSizeRequest>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::SidePanel::left("builder_controls")
        .default_width(240.0)
        .show(ctx, |ui| {
            ui.heading("Strategy Builder");
            ui.separator();

            // Config
            ui.label("Board Size:");
            let sizes: &[(usize, usize)] = &[(2, 2), (3, 2), (3, 3)];
            for &(r, c) in sizes {
                let label = format!("{}x{}", r, c);
                let selected = config.rows == r && config.cols == c;
                if ui.selectable_label(selected, label).clicked() && !selected {
                    config.rows = r;
                    config.cols = c;
                }
            }
            ui.separator();

            ui.label("Build strategy for:");
            ui.horizontal(|ui| {
                if ui.selectable_label(config.player == 0, "Black").clicked() {
                    config.player = 0;
                }
                if ui.selectable_label(config.player == 1, "White").clicked() {
                    config.player = 1;
                }
            });
            ui.separator();

            if ui.button("Start New Builder").clicked() {
                config.start_requested = true;
                board_req.rows = config.rows;
                board_req.cols = config.cols;
                board_req.changed = true;
            }

            ui.separator();

            if let Some(mut session) = session {
                let (defined, remaining) = session.builder.progress();
                let total = defined + remaining + if session.complete { 0 } else { 1 };

                // Progress
                if session.complete {
                    ui.colored_label(
                        egui::Color32::from_rgb(0, 200, 0),
                        format!("Strategy complete! ({defined} info states)"),
                    );
                } else {
                    ui.label(format!(
                        "Progress: {defined}/{total} ({remaining} remaining)"
                    ));
                    let frac = defined as f32 / total.max(1) as f32;
                    ui.add(egui::ProgressBar::new(frac));
                }
                ui.separator();

                if !session.complete {
                    // Current info state
                    ui.label("Current info state:");
                    ui.code(&session.builder.current_info_state);

                    // Legal actions
                    let legal = session.builder.current_legal_actions();
                    let labels: Vec<String> = legal
                        .iter()
                        .map(|&a| info_state::pos_to_label(a, session.builder.cols))
                        .collect();
                    ui.label(format!("Legal: {}", labels.join(", ")));
                    ui.separator();

                    // Action input
                    ui.label("Actions (e.g. 'a1 0.5 b1 0.5' or '= a1 b1' or 'r'):");
                    let response = ui.text_edit_singleline(&mut session.action_input);
                    if response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        let input = session.action_input.clone();
                        submit_action(&mut session, &input);
                    }
                    if ui.button("Submit").clicked() {
                        let input = session.action_input.clone();
                        submit_action(&mut session, &input);
                    }

                    // Error
                    if let Some(ref err) = session.error_msg {
                        ui.colored_label(egui::Color32::RED, err);
                    }

                    ui.separator();

                    // Controls
                    ui.horizontal(|ui| {
                        if ui.button("Rewind").clicked() {
                            session.builder.rewind();
                            session.error_msg = None;
                        }
                        if ui.button("Restart").clicked() {
                            session.builder.restart();
                            session.error_msg = None;
                            session.complete = false;
                        }
                        if ui.button("Random").clicked() {
                            let input = "r".to_string();
                            submit_action(&mut session, &input);
                        }
                    });
                    if ui.button("Complete Randomly").clicked() {
                        session.builder.random_complete();
                        session.complete = session.builder.is_complete();
                        session.error_msg = None;
                    }
                }

                ui.separator();

                // Strategy log
                ui.label("Defined info states:");
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for (state, actions) in session.builder.strategy() {
                            let actions_str: Vec<String> = actions
                                .iter()
                                .map(|(a, p)| {
                                    format!(
                                        "{}:{:.2}",
                                        info_state::pos_to_label(*a, session.builder.cols),
                                        p
                                    )
                                })
                                .collect();
                            // Show abbreviated info state
                            let abbrev: String = state.chars().take(20).collect();
                            ui.label(format!("{abbrev}.. → {}", actions_str.join(" ")));
                        }
                    });
            } else {
                ui.label("Press 'Start New Builder' to begin");
            }
        });
}

fn submit_action(session: &mut BuilderSession, input: &str) {
    match session.builder.submit_actions(input) {
        Ok(super::engine::SubmitResult::Complete) => {
            session.complete = true;
            session.error_msg = None;
            session.action_input.clear();
        }
        Ok(super::engine::SubmitResult::NextState(_)) => {
            session.error_msg = None;
            session.action_input.clear();
        }
        Err(e) => {
            session.error_msg = Some(e);
        }
    }
}

/// System: start a new builder when requested.
pub fn start_builder_system(
    mut commands: Commands,
    mut config: ResMut<BuilderConfig>,
) {
    if !config.start_requested {
        return;
    }
    config.start_requested = false;

    let builder = StrategyBuilder::new(config.rows, config.cols, config.player);
    commands.insert_resource(BuilderSession {
        builder,
        action_input: String::new(),
        error_msg: None,
        complete: false,
    });
}

/// System: handle clicks on hex cells to auto-fill builder input.
pub fn handle_builder_click(
    mut click_events: MessageReader<HexClickEvent>,
    mut session: Option<ResMut<BuilderSession>>,
) {
    let Some(ref mut session) = session else { return };
    if session.complete {
        return;
    }

    for event in click_events.read() {
        let label = info_state::pos_to_label(event.pos, session.builder.cols);
        // Append to input (user can build multi-action input by clicking)
        if session.action_input.is_empty() {
            session.action_input = label;
        } else {
            session.action_input.push(' ');
            session.action_input.push_str(&label);
        }
    }
}

/// System: update cell visuals from builder's current info state.
pub fn update_builder_visuals(
    session: Option<Res<BuilderSession>>,
    mut cell_query: Query<(&HexCell, &mut CellState)>,
) {
    let Some(ref session) = session else { return };
    let grid = session.builder.current_grid();

    for (cell, mut state) in cell_query.iter_mut() {
        if cell.pos < grid.len() {
            *state = match grid[cell.pos] {
                'x' => CellState::Black,
                'o' => CellState::White,
                _ => CellState::Empty,
            };
        }
    }
}
