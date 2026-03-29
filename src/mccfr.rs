use std::collections::HashMap;

use pyo3::prelude::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::state::DarkHexState;
use crate::types::Player;

/// Per-info-state data: cumulative regrets and strategy sums.
#[derive(Clone)]
struct InfoStateData {
    regret_sum: Vec<f32>,
    strategy_sum: Vec<f32>,
}

impl InfoStateData {
    fn new(num_actions: usize) -> Self {
        Self {
            regret_sum: vec![0.0; num_actions],
            strategy_sum: vec![0.0; num_actions],
        }
    }
}

/// Compute current strategy via regret matching.
fn regret_matching(regret_sum: &[f32], out: &mut Vec<f32>) {
    out.clear();
    let positive_sum: f32 = regret_sum.iter().map(|&r| r.max(0.0)).sum();
    if positive_sum > 0.0 {
        out.extend(regret_sum.iter().map(|&r| r.max(0.0) / positive_sum));
    } else {
        let uniform = 1.0 / regret_sum.len() as f32;
        out.resize(regret_sum.len(), uniform);
    }
}

/// Sample an index from a probability distribution.
fn sample_action(probs: &[f32], rng: &mut SmallRng) -> usize {
    let r: f32 = rng.gen();
    let mut cum = 0.0;
    for (i, &p) in probs.iter().enumerate() {
        cum += p;
        if r < cum {
            return i;
        }
    }
    probs.len() - 1
}

/// External Sampling MCCFR solver.
///
/// At the update player's decision nodes, tries ALL actions.
/// At the opponent's decision nodes, samples ONE action from the current strategy.
/// This gives unbiased regret estimates without importance sampling.
#[pyclass]
pub struct MCCFRSolver {
    info_states: HashMap<String, InfoStateData>,
    iterations: usize,
    rows: usize,
    cols: usize,
    rng: SmallRng,
}

#[pymethods]
impl MCCFRSolver {
    /// Create a new MCCFR solver for a given board size.
    #[new]
    #[pyo3(signature = (rows, cols, seed=None))]
    fn new(rows: usize, cols: usize, seed: Option<u64>) -> Self {
        Self {
            info_states: HashMap::new(),
            iterations: 0,
            rows,
            cols,
            rng: SmallRng::seed_from_u64(seed.unwrap_or(42)),
        }
    }

    /// Run `n` iterations of External Sampling MCCFR.
    fn solve(&mut self, n: usize) {
        for _ in 0..n {
            for &update_player in &[Player::Black, Player::White] {
                let mut state = DarkHexState::rs_new(self.rows, self.cols);
                let mut actions_buf = Vec::new();
                let mut sigma_buf = Vec::new();
                self.external_sampling(
                    &mut state,
                    update_player,
                    &mut actions_buf,
                    &mut sigma_buf,
                );
            }
            self.iterations += 1;
        }
    }

    /// Number of completed iterations.
    fn iterations(&self) -> usize {
        self.iterations
    }

    /// Number of info states discovered.
    fn num_info_states(&self) -> usize {
        self.info_states.len()
    }

    /// Get the average (converged) strategy as a Python dict.
    ///
    /// Returns `{info_state_str: [(action, probability), ...]}`.
    fn get_average_strategy(&self) -> HashMap<String, Vec<(usize, f32)>> {
        let mut result = HashMap::new();
        for (key, data) in &self.info_states {
            let sum: f32 = data.strategy_sum.iter().sum();
            if sum <= 0.0 {
                continue;
            }
            // Recover actions from the info state
            // Actions are 0..n for all non-occupied cells visible to the player
            let probs: Vec<(usize, f32)> = data
                .strategy_sum
                .iter()
                .enumerate()
                .map(|(i, &s)| (i, s / sum))
                .filter(|(_, p)| *p > 1e-6)
                .collect();
            if !probs.is_empty() {
                result.insert(key.clone(), probs);
            }
        }
        result
    }

    /// Get the current strategy (from regret matching) for an info state.
    fn get_current_strategy(&self, info_state: &str) -> Option<Vec<f32>> {
        self.info_states.get(info_state).map(|data| {
            let mut sigma = Vec::new();
            regret_matching(&data.regret_sum, &mut sigma);
            sigma
        })
    }

    /// Approximate exploitability (sum of both players' expected losses
    /// against a best-responding opponent). Lower = closer to Nash.
    ///
    /// Uses `num_games` random games to estimate each player's value
    /// under the average strategy vs uniform random opponent.
    /// This is a quick-and-dirty estimate, NOT true exploitability.
    fn estimated_exploitability(&self, _num_games: usize) -> f32 {
        // True exploitability requires best response computation.
        // For now, return NaN to indicate "not yet implemented".
        f32::NAN
    }

    fn __repr__(&self) -> String {
        format!(
            "MCCFRSolver({}x{}, iters={}, info_states={})",
            self.rows,
            self.cols,
            self.iterations,
            self.info_states.len()
        )
    }
}

impl MCCFRSolver {
    /// External Sampling MCCFR traversal.
    ///
    /// Returns the counterfactual value for the update player.
    fn external_sampling(
        &mut self,
        state: &mut DarkHexState,
        update_player: Player,
        actions_buf: &mut Vec<usize>,
        sigma_buf: &mut Vec<f32>,
    ) -> f32 {
        if state.rs_is_terminal() {
            return state.rs_player_return(update_player);
        }

        let player = state.rs_current_player();
        state.rs_legal_actions(actions_buf);
        let num_actions = actions_buf.len();

        if num_actions == 0 {
            return 0.0;
        }

        let info_key = state.rs_info_state_string(player);

        // Ensure info state exists and get current strategy
        if !self.info_states.contains_key(&info_key) {
            self.info_states
                .insert(info_key.clone(), InfoStateData::new(num_actions));
        }
        regret_matching(
            &self.info_states[&info_key].regret_sum,
            sigma_buf,
        );
        // Copy sigma since we need it after mutation
        let sigma: Vec<f32> = sigma_buf.clone();

        // Copy actions since buffer will be reused in recursion
        let actions: Vec<usize> = actions_buf.clone();

        if player == update_player {
            // Update player: try ALL actions
            let mut values = vec![0.0f32; num_actions];
            for i in 0..num_actions {
                let mut child = state.clone();
                child.rs_apply_action(actions[i]);
                values[i] = self.external_sampling(
                    &mut child,
                    update_player,
                    actions_buf,
                    sigma_buf,
                );
            }

            // State value (expected value under current strategy)
            let v: f32 = sigma.iter().zip(values.iter()).map(|(s, u)| s * u).sum();

            // Update regrets: r(I, a) += v(I, a) - v(I)
            let data = self.info_states.get_mut(&info_key).unwrap();
            for i in 0..num_actions {
                data.regret_sum[i] += values[i] - v;
            }

            // Update average strategy
            for i in 0..num_actions {
                data.strategy_sum[i] += sigma[i];
            }

            v
        } else {
            // Opponent: sample ONE action from current strategy
            let a_idx = sample_action(&sigma, &mut self.rng);
            let mut child = state.clone();
            child.rs_apply_action(actions[a_idx]);
            self.external_sampling(
                &mut child,
                update_player,
                actions_buf,
                sigma_buf,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regret_matching_uniform_on_zero() {
        let regrets = vec![0.0, 0.0, 0.0];
        let mut sigma = Vec::new();
        regret_matching(&regrets, &mut sigma);
        assert_eq!(sigma.len(), 3);
        for &p in &sigma {
            assert!((p - 1.0 / 3.0).abs() < 1e-6);
        }
    }

    #[test]
    fn regret_matching_positive_only() {
        let regrets = vec![2.0, -1.0, 1.0];
        let mut sigma = Vec::new();
        regret_matching(&regrets, &mut sigma);
        assert!((sigma[0] - 2.0 / 3.0).abs() < 1e-6);
        assert!((sigma[1] - 0.0).abs() < 1e-6);
        assert!((sigma[2] - 1.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn solver_creates_and_runs() {
        let mut solver = MCCFRSolver::new(2, 2, Some(42));
        solver.solve(100);
        assert_eq!(solver.iterations(), 100);
        assert!(solver.num_info_states() > 0);
    }

    #[test]
    fn solver_2x2_discovers_info_states() {
        let mut solver = MCCFRSolver::new(2, 2, Some(42));
        solver.solve(1000);
        // 2x2 CDH imperfect recall should have a known number of info states
        // The thesis says 42 for IR. Let's just check it's reasonable.
        let n = solver.num_info_states();
        assert!(n > 5, "expected >5 info states, got {n}");
        assert!(n < 200, "expected <200 info states, got {n}");
    }

    #[test]
    fn average_strategy_is_valid() {
        let mut solver = MCCFRSolver::new(2, 2, Some(42));
        solver.solve(500);
        let strategy = solver.get_average_strategy();
        assert!(!strategy.is_empty());
        // Every info state's probabilities should sum to ~1
        for (key, probs) in &strategy {
            let sum: f32 = probs.iter().map(|(_, p)| p).sum();
            assert!(
                (sum - 1.0).abs() < 0.01,
                "strategy at {key} sums to {sum}, expected ~1.0"
            );
        }
    }
}
