use std::collections::HashMap;

use pyo3::prelude::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::game::state::DarkHexState;
use crate::game::types::Player;

/// MCCFR sampling variant.
#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sampling {
    /// Try ALL actions at update player nodes, sample ONE at opponent nodes.
    /// Unbiased, no importance weights. Expensive on large trees.
    External = 0,
    /// Sample ONE action at ALL nodes with epsilon-greedy exploration.
    /// O(depth) per iteration. Needs importance weights but scales to large games.
    Outcome = 1,
}

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

/// MCCFR solver supporting External and Outcome Sampling.
///
/// External Sampling: fast convergence on small games (2x2, 3x2).
/// Outcome Sampling: scales to large games (3x3+) with O(depth) per iteration.
#[pyclass]
pub struct MCCFRSolver {
    info_states: HashMap<String, InfoStateData>,
    iterations: usize,
    rows: usize,
    cols: usize,
    sampling: Sampling,
    epsilon: f32,
    rng: SmallRng,
}

#[pymethods]
impl MCCFRSolver {
    /// Create a new MCCFR solver.
    ///
    /// - `sampling`: `Sampling.External` or `Sampling.Outcome` (default: Outcome)
    /// - `epsilon`: exploration parameter for Outcome Sampling (default 0.6)
    /// - `seed`: RNG seed for reproducibility
    #[new]
    #[pyo3(signature = (rows, cols, sampling=None, epsilon=None, seed=None))]
    fn new(
        rows: usize,
        cols: usize,
        sampling: Option<Sampling>,
        epsilon: Option<f32>,
        seed: Option<u64>,
    ) -> Self {
        Self {
            info_states: HashMap::new(),
            iterations: 0,
            rows,
            cols,
            sampling: sampling.unwrap_or(Sampling::Outcome),
            epsilon: epsilon.unwrap_or(0.6),
            rng: SmallRng::seed_from_u64(seed.unwrap_or(42)),
        }
    }

    /// Run `n` iterations of MCCFR.
    fn solve(&mut self, n: usize) {
        let mut actions_buf = Vec::new();
        let mut sigma_buf = Vec::new();
        let mut q_buf = Vec::new();

        for _ in 0..n {
            for &update_player in &[Player::Black, Player::White] {
                let mut state = DarkHexState::rs_new(self.rows, self.cols);
                match self.sampling {
                    Sampling::External => {
                        self.external_sampling(
                            &mut state,
                            update_player,
                            &mut actions_buf,
                            &mut sigma_buf,
                        );
                    }
                    Sampling::Outcome => {
                        self.outcome_sampling(
                            &mut state,
                            update_player,
                            1.0,
                            1.0,
                            1.0,
                            &mut actions_buf,
                            &mut sigma_buf,
                            &mut q_buf,
                        );
                    }
                }
            }
            self.iterations += 1;
        }
    }

    fn iterations(&self) -> usize {
        self.iterations
    }

    fn num_info_states(&self) -> usize {
        self.info_states.len()
    }

    fn sampling(&self) -> Sampling {
        self.sampling
    }

    fn epsilon(&self) -> f32 {
        self.epsilon
    }

    /// Get the average (converged) strategy.
    ///
    /// Returns `{info_state_str: [(action_index, probability), ...]}`.
    fn get_average_strategy(&self) -> HashMap<String, Vec<(usize, f32)>> {
        let mut result = HashMap::new();
        for (key, data) in &self.info_states {
            let sum: f32 = data.strategy_sum.iter().sum();
            if sum <= 0.0 {
                continue;
            }
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

    fn get_current_strategy(&self, info_state: &str) -> Option<Vec<f32>> {
        self.info_states.get(info_state).map(|data| {
            let mut sigma = Vec::new();
            regret_matching(&data.regret_sum, &mut sigma);
            sigma
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "MCCFRSolver({}x{}, {:?}, eps={}, iters={}, info_states={})",
            self.rows,
            self.cols,
            self.sampling,
            self.epsilon,
            self.iterations,
            self.info_states.len()
        )
    }
}

// --- Traversal implementations (pure Rust, no PyO3) ---

impl MCCFRSolver {
    fn get_strategy(
        &mut self,
        info_key: &str,
        num_actions: usize,
        sigma_buf: &mut Vec<f32>,
    ) {
        if !self.info_states.contains_key(info_key) {
            self.info_states
                .insert(info_key.to_string(), InfoStateData::new(num_actions));
        }
        regret_matching(&self.info_states[info_key].regret_sum, sigma_buf);
    }

    // --- External Sampling ---

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
        self.get_strategy(&info_key, num_actions, sigma_buf);
        let sigma: Vec<f32> = sigma_buf.clone();
        let actions: Vec<usize> = actions_buf.clone();

        if player == update_player {
            let mut values = vec![0.0f32; num_actions];
            for i in 0..num_actions {
                let mut child = state.clone();
                child.rs_apply_action(actions[i]);
                values[i] =
                    self.external_sampling(&mut child, update_player, actions_buf, sigma_buf);
            }

            let v: f32 = sigma.iter().zip(values.iter()).map(|(s, u)| s * u).sum();

            let data = self.info_states.get_mut(&info_key).unwrap();
            for i in 0..num_actions {
                data.regret_sum[i] += values[i] - v;
                data.strategy_sum[i] += sigma[i];
            }
            v
        } else {
            let a_idx = sample_action(&sigma, &mut self.rng);
            let mut child = state.clone();
            child.rs_apply_action(actions[a_idx]);
            self.external_sampling(&mut child, update_player, actions_buf, sigma_buf)
        }
    }

    // --- Outcome Sampling ---

    /// Outcome Sampling MCCFR traversal.
    ///
    /// Samples ONE action at every node using epsilon-greedy exploration.
    /// Returns `u_i(z) / pi_sample(z)` for the sampled terminal z.
    #[allow(clippy::too_many_arguments)]
    fn outcome_sampling(
        &mut self,
        state: &mut DarkHexState,
        update_player: Player,
        pi_i: f32,
        pi_opp: f32,
        pi_sample: f32,
        actions_buf: &mut Vec<usize>,
        sigma_buf: &mut Vec<f32>,
        q_buf: &mut Vec<f32>,
    ) -> f32 {
        if state.rs_is_terminal() {
            return state.rs_player_return(update_player) / pi_sample;
        }

        let player = state.rs_current_player();
        state.rs_legal_actions(actions_buf);
        let num_actions = actions_buf.len();
        if num_actions == 0 {
            return 0.0;
        }

        let info_key = state.rs_info_state_string(player);
        self.get_strategy(&info_key, num_actions, sigma_buf);
        let sigma: Vec<f32> = sigma_buf.clone();
        let actions: Vec<usize> = actions_buf.clone();

        // Epsilon-on-policy sampling probabilities
        let eps = self.epsilon;
        let n = num_actions as f32;
        q_buf.clear();
        q_buf.extend(sigma.iter().map(|&s| eps / n + (1.0 - eps) * s));

        let a_idx = sample_action(q_buf, &mut self.rng);
        let q_a = q_buf[a_idx];

        let mut child = state.clone();
        child.rs_apply_action(actions[a_idx]);

        if player == update_player {
            let tail = self.outcome_sampling(
                &mut child,
                update_player,
                pi_i * sigma[a_idx],
                pi_opp,
                pi_sample * q_a,
                actions_buf,
                sigma_buf,
                q_buf,
            );

            // Counterfactual weight: pi_{-i} * u(z) / pi_s(z)
            let w = tail * pi_opp;

            let data = self.info_states.get_mut(&info_key).unwrap();
            for i in 0..num_actions {
                if i == a_idx {
                    data.regret_sum[i] += w * (1.0 - sigma[a_idx]);
                } else {
                    data.regret_sum[i] -= w * sigma[i];
                }
            }

            // Average strategy: reach-weighted
            let strat_weight = pi_i / pi_sample;
            for i in 0..num_actions {
                data.strategy_sum[i] += strat_weight * sigma[i];
            }

            tail * sigma[a_idx]
        } else {
            self.outcome_sampling(
                &mut child,
                update_player,
                pi_i,
                pi_opp * sigma[a_idx],
                pi_sample * q_a,
                actions_buf,
                sigma_buf,
                q_buf,
            )
        }
    }
}

#[cfg(test)]
#[path = "mccfr_tests.rs"]
mod tests;
