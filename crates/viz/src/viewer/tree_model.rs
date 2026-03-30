//! Strategy tree data model — builds a navigable tree from a strategy dict.

use std::collections::HashMap;

use crate::common::info_state;

/// A node in the strategy tree.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TreeNode {
    pub id: usize,
    pub info_state: String,
    pub player: usize,
    pub grid: Vec<char>,
    pub actions: Vec<TreeEdge>,
    pub is_terminal: bool,
}

/// An edge from a tree node to a child.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TreeEdge {
    pub action: usize,
    pub label: String,
    pub probability: f32,
    pub outcome: EdgeOutcome,
    pub child_id: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EdgeOutcome {
    Placed,
    Collision,
}

/// The complete strategy tree.
#[allow(dead_code)]
pub struct StrategyTree {
    pub nodes: Vec<TreeNode>,
    pub root: usize,
    pub rows: usize,
    pub cols: usize,
    pub player: usize,
}

impl StrategyTree {
    /// Build a tree from a strategy dict, starting from the initial info state.
    pub fn build(
        strategy: &HashMap<String, Vec<(usize, f32)>>,
        rows: usize,
        cols: usize,
        player: usize,
    ) -> Self {
        let mut tree = StrategyTree {
            nodes: Vec::new(),
            root: 0,
            rows,
            cols,
            player,
        };

        let initial = info_state::initial_info_state(player, rows, cols);
        let mut visited: HashMap<String, usize> = HashMap::new();
        tree.build_recursive(&initial, strategy, &mut visited);

        tree
    }

    fn build_recursive(
        &mut self,
        info: &str,
        strategy: &HashMap<String, Vec<(usize, f32)>>,
        visited: &mut HashMap<String, usize>,
    ) -> usize {
        // If already visited, return existing node id
        if let Some(&id) = visited.get(info) {
            return id;
        }

        let (player_id, grid) = info_state::parse_info_state(info);
        let is_terminal = info_state::is_terminal(info, self.rows, self.cols);
        let id = self.nodes.len();

        // Reserve the slot first (for cycle safety)
        self.nodes.push(TreeNode {
            id,
            info_state: info.to_string(),
            player: player_id,
            grid: grid.clone(),
            actions: Vec::new(),
            is_terminal,
        });
        visited.insert(info.to_string(), id);

        if is_terminal {
            return id;
        }

        // Get actions from strategy
        if let Some(action_probs) = strategy.get(info) {
            let collision_possible = info_state::is_collision_possible(info);
            let mut edges = Vec::new();

            for &(action, prob) in action_probs {
                let label = info_state::pos_to_label(action, self.cols);

                // Success case
                let success_info = info_state::info_state_after_action(
                    info,
                    action,
                    self.player,
                    self.cols,
                );
                let child_id = self.build_recursive(&success_info, strategy, visited);
                edges.push(TreeEdge {
                    action,
                    label: label.clone(),
                    probability: prob,
                    outcome: EdgeOutcome::Placed,
                    child_id,
                });

                // Collision case
                if collision_possible {
                    let collision_info = info_state::info_state_after_collision(
                        info,
                        action,
                        self.player,
                        self.cols,
                    );
                    let child_id = self.build_recursive(&collision_info, strategy, visited);
                    edges.push(TreeEdge {
                        action,
                        label: format!("{label}!"),
                        probability: prob,
                        outcome: EdgeOutcome::Collision,
                        child_id,
                    });
                }
            }

            self.nodes[id].actions = edges;
        }

        id
    }

    pub fn node(&self, id: usize) -> &TreeNode {
        &self.nodes[id]
    }
}
