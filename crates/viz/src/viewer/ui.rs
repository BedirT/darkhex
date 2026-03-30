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
}

/// System: egui panel for strategy viewer.
pub fn viewer_ui(
    mut contexts: EguiContexts,
    mut config: ResMut<ViewerConfig>,
    session: Option<ResMut<ViewerSession>>,
    mut board_req: ResMut<BoardSizeRequest>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::SidePanel::left("viewer_controls")
        .default_width(280.0)
        .show(ctx, |ui| {
            ui.heading("Strategy Viewer");
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

            ui.label("View strategy for:");
            ui.horizontal(|ui| {
                if ui.selectable_label(config.player == 0, "Black").clicked() {
                    config.player = 0;
                }
                if ui.selectable_label(config.player == 1, "White").clicked() {
                    config.player = 1;
                }
            });
            ui.separator();

            ui.label("MCCFR Iterations:");
            ui.add(egui::Slider::new(&mut config.iterations, 1000..=200_000).logarithmic(true));
            ui.separator();

            if ui.button("Solve & View").clicked() {
                config.load_requested = true;
                board_req.rows = config.rows;
                board_req.cols = config.cols;
                board_req.changed = true;
            }
            ui.separator();

            // Tree navigation
            if let Some(mut session) = session {
                if let Some(eps) = session.exploitability {
                    ui.label(format!("Exploitability: {eps:.6}"));
                    ui.separator();
                }

                ui.label(format!("{} nodes in tree", session.tree.nodes.len()));
                ui.separator();

                // Selected node info
                let node = session.tree.node(session.selected_node).clone();
                ui.label("Selected node:");
                ui.code(&node.info_state);

                if node.is_terminal {
                    ui.colored_label(egui::Color32::YELLOW, "Terminal state");
                }

                // Board view as text grid
                ui.label("Board view:");
                let grid_str = format_grid(&node.grid, session.tree.cols);
                ui.code(&grid_str);

                ui.separator();

                // Actions at this node
                if !node.actions.is_empty() {
                    ui.label("Actions:");
                    let selected_before = session.selected_node;
                    for edge in &node.actions {
                        let outcome_str = match edge.outcome {
                            EdgeOutcome::Placed => "",
                            EdgeOutcome::Collision => " (collision)",
                        };
                        let text = format!(
                            "{}: {:.2}{}",
                            edge.label, edge.probability, outcome_str
                        );
                        if ui.button(&text).clicked() {
                            session.selected_node = edge.child_id;
                        }
                    }
                    // Back to parent button
                    if session.selected_node != 0 {
                        if ui.button("← Back to root").clicked() {
                            session.selected_node = 0;
                        }
                    }
                }

                ui.separator();

                // Strategy at this node
                if let Some(entries) = session.strategy.get(&node.info_state) {
                    ui.label("Strategy:");
                    for (action, prob) in entries {
                        let label = info_state::pos_to_label(*action, session.tree.cols);
                        ui.label(format!("  {label}: {prob:.4}"));
                    }
                }
            } else {
                ui.label("Press 'Solve & View' to generate a strategy");
            }
        });
}

fn format_grid(grid: &[char], cols: usize) -> String {
    let mut s = String::new();
    for (i, &c) in grid.iter().enumerate() {
        if i > 0 && i % cols == 0 {
            s.push('\n');
        }
        s.push(c);
        s.push(' ');
    }
    s
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

    // Solve
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

    // Compute exploitability
    let exploitability = darkhex_core::solver::exploitability::exploitability(
        config.rows,
        config.cols,
        strategy.clone(),
        None,
    );

    info!(
        "Solved: {} info states, exploitability={:.6}",
        strategy.len(),
        exploitability
    );

    // Build tree
    let tree = StrategyTree::build(&strategy, config.rows, config.cols, config.player);

    commands.insert_resource(ViewerSession {
        tree,
        selected_node: 0,
        strategy,
        exploitability: Some(exploitability),
    });
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
