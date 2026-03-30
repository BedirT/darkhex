use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

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
        .default_width(280.0)
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
                let step = defined + 1;

                // Progress section
                if session.complete {
                    render_completion(ui, defined);
                } else {
                    render_progress(ui, defined, total, step);
                    ui.separator();
                    render_current_state(ui, &session.builder);
                    ui.separator();
                    render_action_input(ui, &mut *session);
                    ui.separator();
                    render_controls(ui, &mut *session);
                }

                ui.separator();
                render_strategy_log(ui, &session.builder);
            } else {
                ui.label("Press 'Start New Builder' to begin");
            }
        });
}

// --- Section renderers ---

/// Render the progress bar with step counter and colored bar.
fn render_progress(ui: &mut egui::Ui, defined: usize, total: usize, step: usize) {
    ui.label(
        egui::RichText::new(format!("Step {step} of ~{total}"))
            .strong(),
    );

    let frac = defined as f32 / total.max(1) as f32;
    let bar_color = if frac > 0.8 {
        egui::Color32::from_rgb(60, 180, 75)
    } else if frac > 0.4 {
        egui::Color32::from_rgb(200, 180, 50)
    } else {
        egui::Color32::from_rgb(100, 140, 180)
    };

    ui.add(
        egui::ProgressBar::new(frac)
            .text(format!("{defined}/{total}"))
            .fill(bar_color),
    );
    ui.label(format!("{remaining} remaining", remaining = total - defined));
}

/// Render the current info state as a visual monospace grid with colored chars.
fn render_current_state(ui: &mut egui::Ui, builder: &StrategyBuilder) {
    ui.label(egui::RichText::new("Current info state:").strong());

    let (_pid, grid) = info_state::parse_info_state(&builder.current_info_state);

    // Render the grid as a colored monospace display
    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(25, 25, 40))
        .corner_radius(4.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            let cols = builder.cols;
            for row_start in (0..grid.len()).step_by(cols) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    for i in row_start..(row_start + cols).min(grid.len()) {
                        let c = grid[i];
                        let (text, color) = match c {
                            'x' => ("x", egui::Color32::from_rgb(200, 60, 60)),
                            'o' => ("o", egui::Color32::from_rgb(220, 210, 190)),
                            _ => (".", egui::Color32::from_rgb(100, 100, 120)),
                        };
                        ui.label(
                            egui::RichText::new(text)
                                .monospace()
                                .size(18.0)
                                .color(color),
                        );
                    }
                });
            }
        });

    // Legal actions
    let legal = builder.current_legal_actions();
    let labels: Vec<String> = legal
        .iter()
        .map(|&a| info_state::pos_to_label(a, builder.cols))
        .collect();
    ui.label(format!("Legal: {}", labels.join(", ")));
}

/// Render the action input field with placeholder and format tooltip.
fn render_action_input(ui: &mut egui::Ui, session: &mut BuilderSession) {
    let label_response = ui.label("Actions:");
    // Show tooltip on the label
    label_response.on_hover_text(
        "Input formats:\n\
         \n\
         a1 0.5 b1 0.5  action-prob pairs\n\
         = a1 b1         equal probability\n\
         a1              single action (p=1)\n\
         r               random action",
    );

    let hint = "e.g. a1 0.5 b1 0.5";
    let response = ui.add(
        egui::TextEdit::singleline(&mut session.action_input)
            .hint_text(hint)
            .desired_width(f32::INFINITY),
    );
    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        let input = session.action_input.clone();
        submit_action(session, &input);
        // Re-focus the input field after submission
        response.request_focus();
    }
    if ui.button("Submit").clicked() {
        let input = session.action_input.clone();
        submit_action(session, &input);
    }

    // Error display
    if let Some(ref err) = session.error_msg {
        ui.colored_label(egui::Color32::RED, err);
    }
}

/// Render the control buttons in a grouped horizontal bar.
fn render_controls(ui: &mut egui::Ui, session: &mut BuilderSession) {
    ui.horizontal(|ui| {
        if ui.button("\u{23EA} Rewind").clicked() {
            session.builder.rewind();
            session.error_msg = None;
        }
        if ui.button("\u{1F504} Restart").clicked() {
            session.builder.restart();
            session.error_msg = None;
            session.complete = false;
        }
        if ui.button("\u{1F3B2} Random").clicked() {
            let input = "r".to_string();
            submit_action(session, &input);
        }
    });

    ui.add_space(4.0);

    // Complete Randomly as a distinct colored button
    let btn = egui::Button::new(
        egui::RichText::new("Complete Randomly")
            .color(egui::Color32::WHITE),
    )
    .fill(egui::Color32::from_rgb(140, 60, 180));
    if ui.add(btn).clicked() {
        session.builder.random_complete();
        session.complete = session.builder.is_complete();
        session.error_msg = None;
    }
}

/// Render the strategy log as a compact table.
fn render_strategy_log(ui: &mut egui::Ui, builder: &StrategyBuilder) {
    ui.label(egui::RichText::new("Defined info states:").strong());

    let strategy = builder.strategy();
    if strategy.is_empty() {
        ui.label("(none yet)");
        return;
    }

    egui::ScrollArea::vertical()
        .max_height(200.0)
        .show(ui, |ui| {
            egui::Grid::new("strategy_log_grid")
                .striped(true)
                .min_col_width(40.0)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    // Header
                    ui.label(egui::RichText::new("State").strong().small());
                    ui.label(egui::RichText::new("Actions").strong().small());
                    ui.end_row();

                    for (state, actions) in strategy {
                        // State preview: extract grid chars and show inline
                        let (_pid, grid) = info_state::parse_info_state(state);
                        let preview: String = grid.iter().collect();

                        ui.label(
                            egui::RichText::new(&preview).monospace().small(),
                        );

                        // Action distribution
                        let dist: Vec<String> = actions
                            .iter()
                            .map(|(a, p)| {
                                format!(
                                    "{}:{:.0}%",
                                    info_state::pos_to_label(*a, builder.cols),
                                    p * 100.0,
                                )
                            })
                            .collect();
                        ui.label(
                            egui::RichText::new(dist.join(" ")).small(),
                        );
                        ui.end_row();
                    }
                });
        });
}

/// Render completion celebration with total count and export option.
fn render_completion(ui: &mut egui::Ui, defined: usize) {
    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(20, 60, 20))
        .corner_radius(6.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("Strategy Complete!")
                    .heading()
                    .color(egui::Color32::from_rgb(100, 255, 100)),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(format!("{defined} info states defined"))
                    .color(egui::Color32::from_rgb(180, 220, 180)),
            );
            ui.add_space(8.0);
            if ui.button("Export Strategy (JSON)").clicked() {
                bevy::log::info!("Strategy export requested ({defined} info states)");
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
        Ok(super::engine::SubmitResult::NextState) => {
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
