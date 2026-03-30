//! Strategy builder engine — Rust port of darkhex/gui/strategy_generator.py.
//!
//! Interactively builds a complete strategy by presenting info states one at a time.
//! The user specifies action-probability pairs for each. The engine branches on
//! success/collision outcomes and tracks completeness.

use std::collections::HashMap;

use crate::common::info_state;

/// A snapshot for undo/redo.
#[derive(Clone)]
struct BuilderSnapshot {
    info_states: HashMap<String, Vec<(usize, f32)>>,
    action_stack: Vec<String>,
    current_info_state: String,
}

/// Result of submitting actions.
pub enum SubmitResult {
    /// Strategy is now complete (no more branches).
    Complete,
    /// Moved to next info state.
    NextState,
}

/// Strategy builder state machine.
pub struct StrategyBuilder {
    pub rows: usize,
    pub cols: usize,
    pub player: usize,
    pub info_states: HashMap<String, Vec<(usize, f32)>>,
    pub action_stack: Vec<String>,
    pub current_info_state: String,
    history: Vec<BuilderSnapshot>,
}

impl StrategyBuilder {
    /// Create a new builder for the given board size and player.
    pub fn new(rows: usize, cols: usize, player: usize) -> Self {
        let initial = info_state::initial_info_state(player, rows, cols);
        let mut builder = Self {
            rows,
            cols,
            player,
            info_states: HashMap::new(),
            action_stack: Vec::new(),
            current_info_state: initial.clone(),
            history: Vec::new(),
        };
        builder.save_snapshot();
        builder
    }

    fn save_snapshot(&mut self) {
        self.history.push(BuilderSnapshot {
            info_states: self.info_states.clone(),
            action_stack: self.action_stack.clone(),
            current_info_state: self.current_info_state.clone(),
        });
    }

    fn restore_snapshot(&mut self, snap: BuilderSnapshot) {
        self.info_states = snap.info_states;
        self.action_stack = snap.action_stack;
        self.current_info_state = snap.current_info_state;
    }

    /// Submit action-probability pairs for the current info state.
    ///
    /// Input format:
    /// - `"a1 0.3 b1 0.7"` — action-probability pairs
    /// - `"= a1 b1"` — equiprobable
    /// - `"r"` — random single action (probability 1.0)
    /// - `"a1"` — single action with probability 1.0
    pub fn submit_actions(&mut self, input: &str) -> Result<SubmitResult, String> {
        let (actions, probs) = self.parse_input(input)?;

        // Validate probabilities sum to 1
        let sum: f32 = probs.iter().sum();
        if (sum - 1.0).abs() > 1e-5 {
            return Err(format!("Probabilities sum to {sum}, expected 1.0"));
        }

        // Record strategy for current info state
        let action_probs: Vec<(usize, f32)> =
            actions.iter().copied().zip(probs.iter().copied()).collect();
        self.info_states
            .insert(self.current_info_state.clone(), action_probs);

        // Compute successor info states for each action
        let collision_possible = info_state::is_collision_possible(&self.current_info_state);

        for &action in &actions {
            // Success case: the player's own stone is placed
            let success_state = info_state::info_state_after_action(
                &self.current_info_state,
                action,
                self.player,
                self.cols,
            );
            if !info_state::is_terminal(&success_state, self.rows, self.cols)
                && !self.info_states.contains_key(&success_state)
                && !self.action_stack.contains(&success_state)
            {
                self.action_stack.push(success_state);
            }

            // Collision case: opponent's stone is discovered
            if collision_possible {
                let collision_state = info_state::info_state_after_collision(
                    &self.current_info_state,
                    action,
                    self.player,
                    self.cols,
                );
                if !info_state::is_terminal(&collision_state, self.rows, self.cols)
                    && !self.info_states.contains_key(&collision_state)
                    && !self.action_stack.contains(&collision_state)
                {
                    self.action_stack.push(collision_state);
                }
            }
        }

        // Move to next info state
        if self.action_stack.is_empty() {
            self.save_snapshot();
            return Ok(SubmitResult::Complete);
        }

        self.current_info_state = self.action_stack.pop().unwrap();
        self.save_snapshot();
        Ok(SubmitResult::NextState)
    }

    /// Parse user input into (actions, probabilities).
    fn parse_input(&self, input: &str) -> Result<(Vec<usize>, Vec<f32>), String> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err("Empty input".to_string());
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();

        if parts[0] == "r" {
            // Random action
            let action = info_state::random_action(&self.current_info_state)
                .ok_or("No legal actions available")?;
            return Ok((vec![action], vec![1.0]));
        }

        if parts[0] == "=" {
            // Equiprobable: "= a1 b1 c1"
            if parts.len() < 2 {
                return Err("Equiprobable needs at least one action".to_string());
            }
            let mut actions = Vec::new();
            for &label in &parts[1..] {
                let pos = info_state::label_to_pos(label, self.cols)
                    .ok_or_else(|| format!("Invalid action label: {label}"))?;
                self.validate_action(pos)?;
                actions.push(pos);
            }
            let prob = 1.0 / actions.len() as f32;
            let probs = vec![prob; actions.len()];
            return Ok((actions, probs));
        }

        if parts.len() == 1 {
            // Single action: "a1" → probability 1.0
            let pos = info_state::label_to_pos(parts[0], self.cols)
                .ok_or_else(|| format!("Invalid action label: {}", parts[0]))?;
            self.validate_action(pos)?;
            return Ok((vec![pos], vec![1.0]));
        }

        // Action-probability pairs: "a1 0.3 b1 0.7"
        if parts.len() % 2 != 0 {
            return Err("Expected pairs of action probability".to_string());
        }
        let mut actions = Vec::new();
        let mut probs = Vec::new();
        for chunk in parts.chunks(2) {
            let pos = info_state::label_to_pos(chunk[0], self.cols)
                .ok_or_else(|| format!("Invalid action label: {}", chunk[0]))?;
            self.validate_action(pos)?;
            let prob: f32 = chunk[1]
                .parse()
                .map_err(|_| format!("Invalid probability: {}", chunk[1]))?;
            actions.push(pos);
            probs.push(prob);
        }
        Ok((actions, probs))
    }

    fn validate_action(&self, pos: usize) -> Result<(), String> {
        let legal = info_state::legal_actions(&self.current_info_state);
        if !legal.contains(&pos) {
            let label = info_state::pos_to_label(pos, self.cols);
            return Err(format!("Action {label} (pos {pos}) is not legal"));
        }
        Ok(())
    }

    /// Undo the last submit.
    pub fn rewind(&mut self) -> bool {
        if self.history.len() <= 1 {
            return false;
        }
        self.history.pop(); // remove current
        if let Some(snap) = self.history.last().cloned() {
            self.restore_snapshot(snap);
            true
        } else {
            false
        }
    }

    /// Restart from the beginning.
    pub fn restart(&mut self) {
        if let Some(snap) = self.history.first().cloned() {
            self.restore_snapshot(snap);
            self.history.truncate(1);
        }
    }

    /// Randomly complete all remaining branches.
    pub fn random_complete(&mut self) {
        while !self.is_complete() {
            if let Err(e) = self.submit_actions("r") {
                bevy::log::warn!("Random complete error: {e}");
                break;
            }
        }
    }

    /// Whether the strategy is complete (no more branches to specify).
    pub fn is_complete(&self) -> bool {
        self.action_stack.is_empty() && self.info_states.contains_key(&self.current_info_state)
    }

    /// Progress: (defined_count, remaining_count).
    pub fn progress(&self) -> (usize, usize) {
        (self.info_states.len(), self.action_stack.len())
    }

    /// Get the strategy as a map.
    pub fn strategy(&self) -> &HashMap<String, Vec<(usize, f32)>> {
        &self.info_states
    }

    /// Get the current info state grid for rendering.
    pub fn current_grid(&self) -> Vec<char> {
        let (_pid, grid) = info_state::parse_info_state(&self.current_info_state);
        grid
    }

    /// Get legal actions at the current info state.
    pub fn current_legal_actions(&self) -> Vec<usize> {
        info_state::legal_actions(&self.current_info_state)
    }
}
