use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use darkhex_core::solver::mccfr::{MCCFRSolver, Sampling};

use super::tree_model::{EdgeOutcome, StrategyTree};
use crate::common::info_state;
use crate::hex_board::components::{CellState, HexCell};
use crate::hex_board::research_renderer::BoardSizeRequest;

/// Viewer configuration.
#[derive(Resource)]
pub struct ViewerConfig {
    pub rows: usize,
    pub cols: usize,
    pub player: usize,
    pub iterations: usize,
    pub load_requested: bool,
}

impl Default for ViewerConfig {
    fn default() -> Self {
        Self {
            rows: 2,
            cols: 2,
            player: 0,
            iterations: 50_000,
            load_requested: false,
        }
    }
}

/// Active viewer session.
#[derive(Resource)]
pub struct ViewerSession {
    pub tree: StrategyTree,
    pub selected_node: usize,
    pub strategy: HashMap<String, Vec<(usize, f32)>>,
    pub exploitability: Option<f64>,
    pub navigation_history: Vec<usize>,
    pub solve_time_ms: u128,
    pub num_info_states: usize,
}

/// Resource indicating a solve is in progress.
#[derive(Resource)]
pub struct SolveInProgress;

// -- Color helpers --

fn exploitability_color(eps: f64) -> egui::Color32 {
    if eps < 0.01 {
        egui::Color32::from_rgb(80, 200, 120) // green
    } else if eps < 0.1 {
        egui::Color32::from_rgb(240, 200, 50) // yellow
    } else {
        egui::Color32::from_rgb(220, 80, 60) // red
    }
}

fn probability_color(p: f32) -> egui::Color32 {
    // Interpolate from muted blue (low) to saturated blue (high)
    let base_r: f32 = 100.0;
    let base_g: f32 = 140.0;
    let base_b: f32 = 200.0;
    let hi_r: f32 = 50.0;
    let hi_g: f32 = 120.0;
    let hi_b: f32 = 255.0;
    let t = p.clamp(0.0, 1.0);
    egui::Color32::from_rgb(
        (base_r + (hi_r - base_r) * t) as u8,
        (base_g + (hi_g - base_g) * t) as u8,
        (base_b + (hi_b - base_b) * t) as u8,
    )
}

fn player_name(player: usize) -> &'static str {
    if player == 0 { "Black" } else { "White" }
}

fn cell_char_color(c: char) -> egui::Color32 {
    match c {
        'x' => egui::Color32::from_rgb(30, 30, 30),       // black stone
        'o' => egui::Color32::from_rgb(230, 224, 216),     // white stone
        _ => egui::Color32::from_rgb(100, 130, 160),       // empty
    }
}

// -- Breadcrumb builder --

fn build_breadcrumbs(tree: &StrategyTree, history: &[usize], current: usize) -> Vec<(String, usize)> {
    let mut crumbs: Vec<(String, usize)> = Vec::new();
    crumbs.push(("Root".to_string(), 0));
    for &node_id in history {
        if node_id == 0 {
            continue;
        }
        let node = tree.node(node_id);
        let label = node.edge_label.clone().unwrap_or_else(|| format!("#{node_id}"));
        crumbs.push((label, node_id));
    }
    if current != 0 && !history.contains(&current) {
        let node = tree.node(current);
        let label = node.edge_label.clone().unwrap_or_else(|| format!("#{current}"));
        crumbs.push((label, current));
    }
    crumbs
}

/// System: egui panel for strategy viewer.
pub fn viewer_ui(
    mut contexts: EguiContexts,
    mut config: ResMut<ViewerConfig>,
    session: Option<ResMut<ViewerSession>>,
    solving: Option<Res<SolveInProgress>>,
    mut board_req: ResMut<BoardSizeRequest>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::SidePanel::left("viewer_controls")
        .default_width(320.0)
        .show(ctx, |ui| {
            ui.heading("Strategy Viewer");
            ui.separator();

            // -- Solver config section --
            ui.label("Board Size:");
            let sizes: &[(usize, usize)] = &[(2, 2), (3, 2), (3, 3)];
            ui.horizontal(|ui| {
                for &(r, c) in sizes {
                    let label = format!("{r}x{c}");
                    let selected = config.rows == r && config.cols == c;
                    if ui.selectable_label(selected, label).clicked() && !selected {
                        config.rows = r;
                        config.cols = c;
                    }
                }
            });
            ui.add_space(4.0);

            ui.label("View strategy for:");
            ui.horizontal(|ui| {
                if ui.selectable_label(config.player == 0, "Black").clicked() {
                    config.player = 0;
                }
                if ui.selectable_label(config.player == 1, "White").clicked() {
                    config.player = 1;
                }
            });
            ui.add_space(4.0);

            ui.label("MCCFR Iterations:");
            ui.add(egui::Slider::new(&mut config.iterations, 1000..=200_000).logarithmic(true));
            ui.add_space(4.0);

            // Solve button / indicator
            if solving.is_some() {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Solving...");
                });
            } else if ui.button("Solve & View").clicked() {
                config.load_requested = true;
                board_req.rows = config.rows;
                board_req.cols = config.cols;
                board_req.changed = true;
            }
            ui.separator();

            // -- Session display --
            if let Some(mut session) = session {
                // Solve stats
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "{} info states | solved in {}ms",
                        session.num_info_states, session.solve_time_ms,
                    ));
                });

                // Exploitability badge
                if let Some(eps) = session.exploitability {
                    let color = exploitability_color(eps);
                    ui.horizontal(|ui| {
                        ui.label("Exploitability:");
                        let response = ui.label(
                            egui::RichText::new(format!(" {eps:.6} "))
                                .color(egui::Color32::WHITE)
                                .strong()
                                .background_color(color),
                        );
                        response.on_hover_text(
                            "Sum of best-response improvements for both players.\n\
                             Lower = closer to Nash equilibrium.\n\
                             < 0.01 = near-optimal, < 0.1 = reasonable, > 0.1 = unconverged",
                        );
                    });
                }
                ui.separator();

                // Breadcrumb navigation
                let crumbs = build_breadcrumbs(
                    &session.tree,
                    &session.navigation_history,
                    session.selected_node,
                );
                if crumbs.len() > 1 {
                    ui.horizontal_wrapped(|ui| {
                        for (i, (label, node_id)) in crumbs.iter().enumerate() {
                            if i > 0 {
                                ui.label(">");
                            }
                            if *node_id == session.selected_node {
                                ui.strong(label);
                            } else if ui.link(label).clicked() {
                                // Navigate to this breadcrumb: truncate history
                                let target = *node_id;
                                if let Some(pos) = session.navigation_history.iter().position(|&n| n == target) {
                                    session.navigation_history.truncate(pos);
                                }
                                session.selected_node = target;
                            }
                        }
                    });
                    ui.add_space(4.0);
                }

                // Back button
                if !session.navigation_history.is_empty() {
                    if ui.button("< Back").clicked() {
                        if let Some(prev) = session.navigation_history.pop() {
                            session.selected_node = prev;
                        }
                    }
                    ui.add_space(4.0);
                }

                // Node info header
                let node = session.tree.node(session.selected_node).clone();
                ui.horizontal(|ui| {
                    let turn_label = player_name(node.player);
                    let turn_color = if node.player == 0 {
                        egui::Color32::from_rgb(60, 60, 60)
                    } else {
                        egui::Color32::from_rgb(210, 205, 195)
                    };
                    ui.label(
                        egui::RichText::new(format!("{turn_label}'s turn"))
                            .color(turn_color)
                            .strong(),
                    );
                    ui.label(format!("| depth {}", node.depth));
                    ui.label(format!("| {} nodes total", session.tree.nodes.len()));
                });

                if node.is_terminal {
                    ui.colored_label(egui::Color32::YELLOW, "Terminal state");
                }
                ui.separator();

                // Board grid in monospace with colored characters
                ui.label("Board view:");
                draw_board_grid(ui, &node.grid, session.tree.cols);
                ui.separator();

                // Action edges as styled buttons
                if !node.actions.is_empty() {
                    ui.label("Actions:");
                    ui.add_space(2.0);
                    for edge in &node.actions {
                        let pct = edge.probability * 100.0;
                        let is_collision = edge.outcome == EdgeOutcome::Collision;

                        // Build button label
                        let icon = if is_collision { "x " } else { "# " };
                        let outcome_suffix = if is_collision { " (collision)" } else { "" };
                        let btn_text = format!("{icon}{}: {pct:.1}%{outcome_suffix}", edge.label);

                        let text_color = if is_collision {
                            egui::Color32::from_rgb(220, 80, 60)
                        } else {
                            probability_color(edge.probability)
                        };

                        // Probability bar + button
                        let bar_width = 200.0;
                        let filled = bar_width * edge.probability;

                        ui.horizontal(|ui| {
                            let response = ui.add(
                                egui::Button::new(
                                    egui::RichText::new(&btn_text).color(text_color).monospace(),
                                )
                                .min_size(egui::vec2(bar_width + 20.0, 22.0)),
                            );
                            if response.clicked() {
                                let prev = session.selected_node;
                                session.navigation_history.push(prev);
                                session.selected_node = edge.child_id;
                            }

                            // Draw a small probability bar beneath/beside
                            let bar_rect = egui::Rect::from_min_size(
                                response.rect.left_bottom() + egui::vec2(0.0, -3.0),
                                egui::vec2(filled, 2.0),
                            );
                            let bar_color = if is_collision {
                                egui::Color32::from_rgb(220, 80, 60)
                            } else {
                                probability_color(edge.probability)
                            };
                            ui.painter()
                                .rect_filled(bar_rect, 0.0, bar_color);
                        });
                    }
                }
                ui.separator();

                // Strategy at this info state (probability bars)
                if let Some(entries) = session.strategy.get(&node.info_state) {
                    ui.label("Strategy probabilities:");
                    ui.add_space(2.0);
                    let bar_max_width = 180.0;
                    for (action, prob) in entries {
                        let label = info_state::pos_to_label(*action, session.tree.cols);
                        let pct = prob * 100.0;
                        ui.horizontal(|ui| {
                            ui.monospace(format!("{label}: {pct:5.1}%"));
                            // Horizontal bar
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(bar_max_width, 14.0),
                                egui::Sense::hover(),
                            );
                            let bg = egui::Color32::from_rgb(50, 50, 70);
                            ui.painter().rect_filled(rect, 2.0, bg);
                            let filled_rect = egui::Rect::from_min_size(
                                rect.min,
                                egui::vec2(bar_max_width * prob, 14.0),
                            );
                            ui.painter()
                                .rect_filled(filled_rect, 2.0, probability_color(*prob));
                        });
                    }
                }
            } else {
                ui.label("Press 'Solve & View' to generate a strategy.");
            }
        });
}

/// Draw the hex board grid using colored monospace characters.
fn draw_board_grid(ui: &mut egui::Ui, grid: &[char], cols: usize) {
    let rows = (grid.len() + cols - 1) / cols;
    for r in 0..rows {
        ui.horizontal(|ui| {
            // Hex offset: indent each row by its index
            let indent = r as f32 * 10.0;
            ui.add_space(indent);
            for c in 0..cols {
                let idx = r * cols + c;
                if idx < grid.len() {
                    let ch = grid[idx];
                    let display = match ch {
                        'x' => "X",
                        'o' => "O",
                        _ => ".",
                    };
                    ui.label(
                        egui::RichText::new(display)
                            .monospace()
                            .strong()
                            .color(cell_char_color(ch))
                            .size(16.0),
                    );
                }
            }
        });
    }
}

/// System: solve and build tree when requested.
pub fn load_strategy_system(
    mut commands: Commands,
    mut config: ResMut<ViewerConfig>,
) {
    if !config.load_requested {
        return;
    }
    config.load_requested = false;

    info!(
        "Solving {}x{} with {} MCCFR iterations for player {}...",
        config.rows, config.cols, config.iterations, config.player
    );

    commands.insert_resource(SolveInProgress);

    // Solve (synchronous for now)
    let start = std::time::Instant::now();
    let mut solver = MCCFRSolver::new(
        config.rows,
        config.cols,
        Some(Sampling::Outcome),
        Some(0.6),
        Some(42),
    )
    .expect("valid solver params");
    solver.solve(config.iterations);
    let strategy = solver.get_average_strategy();
    let solve_time_ms = start.elapsed().as_millis();
    let num_info_states = strategy.len();

    // Compute exploitability
    let exploitability = darkhex_core::solver::exploitability::exploitability(
        config.rows,
        config.cols,
        strategy.clone(),
        None,
    );

    info!(
        "Solved: {} info states, exploitability={:.6}, time={}ms",
        num_info_states, exploitability, solve_time_ms,
    );

    // Build tree
    let tree = StrategyTree::build(&strategy, config.rows, config.cols, config.player);

    commands.insert_resource(ViewerSession {
        tree,
        selected_node: 0,
        strategy,
        exploitability: Some(exploitability),
        navigation_history: Vec::new(),
        solve_time_ms,
        num_info_states,
    });
    commands.remove_resource::<SolveInProgress>();
}

/// System: update cell visuals from viewer's selected node.
pub fn update_viewer_visuals(
    session: Option<Res<ViewerSession>>,
    mut cell_query: Query<(&HexCell, &mut CellState)>,
) {
    let Some(ref session) = session else { return };
    let node = session.tree.node(session.selected_node);

    for (cell, mut state) in cell_query.iter_mut() {
        if cell.pos < node.grid.len() {
            *state = match node.grid[cell.pos] {
                'x' => CellState::Black,
                'o' => CellState::White,
                _ => CellState::Empty,
            };
        }
    }
}
