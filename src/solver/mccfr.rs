use std::collections::HashMap;

use pyo3::prelude::*;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::game::state::DarkHexState;
use crate::game::types::Player;
use crate::solver::pone::PoneDb;

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
    /// Actual cell indices corresponding to each slot (sorted ascending).
    actions: Vec<usize>,
}

impl InfoStateData {
    fn new(actions: &[usize]) -> Self {
        let n = actions.len();
        Self {
            regret_sum: vec![0.0; n],
            strategy_sum: vec![0.0; n],
            actions: actions.to_vec(),
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
/// Handles floating-point rounding by sampling in [0, sum) range.
fn sample_action(probs: &[f32], rng: &mut SmallRng) -> usize {
    let total: f32 = probs.iter().sum();
    let r: f32 = rng.gen::<f32>() * total;
    let mut cum = 0.0;
    for (i, &p) in probs.iter().enumerate() {
        cum += p;
        if r < cum {
            return i;
        }
    }
    // Fallback: return last nonzero-probability index
    for i in (0..probs.len()).rev() {
        if probs[i] > 0.0 {
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
    /// Optional pONE database for pruning determined subtrees.
    pone_db: Option<PoneDb>,
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
    ) -> PyResult<Self> {
        let sampling_mode = sampling.unwrap_or(Sampling::Outcome);
        let eps = epsilon.unwrap_or(0.6);
        if sampling_mode == Sampling::Outcome && !(0.0 < eps && eps <= 1.0) {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "epsilon must be in (0, 1] for Outcome Sampling",
            ));
        }
        Ok(Self {
            info_states: HashMap::new(),
            iterations: 0,
            rows,
            cols,
            sampling: sampling_mode,
            epsilon: eps,
            rng: SmallRng::seed_from_u64(seed.unwrap_or(42)),
            pone_db: None,
        })
    }

    /// Set an optional pONE database for pruning determined subtrees.
    ///
    /// When set, MCCFR will skip traversal into info states where the
    /// current player has a probability-1 win, returning the determined
    /// outcome immediately.
    fn set_pone_db(&mut self, db: PoneDb) {
        self.pone_db = Some(db);
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
                .actions
                .iter()
                .zip(data.strategy_sum.iter())
                .map(|(&action, &s)| (action, s / sum))
                .filter(|(_, p)| *p > 1e-6)
                .collect();
            if !probs.is_empty() {
                result.insert(key.clone(), probs);
            }
        }
        result
    }

    /// Get the current strategy at an info state.
    ///
    /// Accepts both canonical and non-canonical info state strings.
    fn get_current_strategy(&self, info_state: &str) -> Option<Vec<f32>> {
        // Canonicalize before lookup — solver stores canonical keys only.
        let canon = crate::solver::pone::canonicalize_info_state_str(
            info_state, self.rows, self.cols,
        );
        self.info_states.get(&canon).map(|data| {
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
    fn get_strategy(&mut self, info_key: &str, actions: &[usize], sigma_buf: &mut Vec<f32>) {
        if !self.info_states.contains_key(info_key) {
            self.info_states
                .insert(info_key.to_string(), InfoStateData::new(actions));
        }
        regret_matching(&self.info_states[info_key].regret_sum, sigma_buf);
    }

    /// Register a uniform strategy for a pONE info state.
    ///
    /// This ensures `get_average_strategy()` includes pONE states, so
    /// exploitability can evaluate them instead of falling back to an
    /// empty/default strategy. The uniform strategy is correct: at a
    /// pONE state the player wins regardless of action choice.
    fn register_pone_strategy(&mut self, info_key: &str, actions: &[usize]) {
        self.info_states
            .entry(info_key.to_string())
            .or_insert_with(|| {
                let n = actions.len();
                InfoStateData {
                    regret_sum: vec![0.0; n],
                    strategy_sum: vec![1.0; n], // uniform: all actions equally good
                    actions: actions.to_vec(),
                }
            });
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

        let actions: Vec<usize> = actions_buf.clone();
        let n = self.rows * self.cols;
        let (info_key, is_canonical) = state.rs_canonical_info_state(player);

        // Canonical actions for InfoStateData storage
        let canonical_actions: Vec<usize> = if is_canonical {
            actions.clone()
        } else {
            actions.iter().rev().map(|&a| n - 1 - a).collect()
        };

        self.get_strategy(&info_key, &canonical_actions, sigma_buf);

        // Map sigma from canonical to original action order
        let sigma: Vec<f32> = if is_canonical {
            sigma_buf.clone()
        } else {
            sigma_buf.iter().rev().cloned().collect()
        };

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
                let ci = if is_canonical { i } else { num_actions - 1 - i };
                data.regret_sum[ci] += values[i] - v;
                data.strategy_sum[ci] += sigma[i];
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

    /// Outcome Sampling MCCFR traversal (OpenSpiel formulation).
    ///
    /// Returns an estimate of the "tail value" at this node — the utility
    /// weighted by tail reach/sample ratios. Raw utility at terminals,
    /// importance-corrected value_estimate at decision nodes.
    ///
    /// Epsilon-greedy exploration is applied ONLY at the update player's
    /// nodes. Opponent nodes sample from the current strategy directly.
    ///
    /// Reference: OpenSpiel outcome_sampling_mccfr.cc (Lanctot et al.)
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
            // Return raw utility — weighting deferred to regret update
            return state.rs_player_return(update_player);
        }

        let player = state.rs_current_player();

        state.rs_legal_actions(actions_buf);
        let num_actions = actions_buf.len();
        if num_actions == 0 {
            return 0.0;
        }

        let actions: Vec<usize> = actions_buf.clone();
        let n = self.rows * self.cols;
        let (info_key, is_canonical) = state.rs_canonical_info_state(player);

        // Canonical actions for InfoStateData storage
        let canonical_actions: Vec<usize> = if is_canonical {
            actions.clone()
        } else {
            actions.iter().rev().map(|&a| n - 1 - a).collect()
        };

        self.get_strategy(&info_key, &canonical_actions, sigma_buf);

        // Map sigma from canonical to original action order
        let sigma: Vec<f32> = if is_canonical {
            sigma_buf.clone()
        } else {
            sigma_buf.iter().rev().cloned().collect()
        };

        // Sampling policy: epsilon-greedy at update player, sigma at opponent
        let is_update = player == update_player;
        q_buf.clear();
        if is_update {
            let eps = self.epsilon;
            let n = num_actions as f32;
            q_buf.extend(sigma.iter().map(|&s| eps / n + (1.0 - eps) * s));
        } else {
            q_buf.extend_from_slice(&sigma);
        }

        let a_idx = sample_action(q_buf, &mut self.rng);
        let q_a = q_buf[a_idx];

        // Thread reach probabilities
        let mut child = state.clone();
        child.rs_apply_action(actions[a_idx]);

        let (new_pi_i, new_pi_opp) = if is_update {
            (pi_i * sigma[a_idx], pi_opp)
        } else {
            (pi_i, pi_opp * sigma[a_idx])
        };
        let new_pi_sample = pi_sample * q_a;

        let child_value = self.outcome_sampling(
            &mut child,
            update_player,
            new_pi_i,
            new_pi_opp,
            new_pi_sample,
            actions_buf,
            sigma_buf,
            q_buf,
        );

        // Importance-corrected child value for the sampled action
        // For unsampled actions, child_values[a] = 0 (vanilla baseline)
        let child_value_corrected = child_value / q_a;

        // value_estimate = sum_a sigma[a] * child_values[a]
        // Only sampled action contributes (others have baseline 0)
        let value_estimate = sigma[a_idx] * child_value_corrected;

        if is_update {
            // Counterfactual value = value_estimate * opp_reach / sample_reach
            let cf_prefix = pi_opp / pi_sample;
            let cf_value = value_estimate * cf_prefix;
            let cf_action_value = child_value_corrected * cf_prefix;

            // Regret: r(I, a) += cf_action_value(a) - cf_value
            let data = self.info_states.get_mut(&info_key).unwrap();
            for i in 0..num_actions {
                let ci = if is_canonical { i } else { num_actions - 1 - i };
                if i == a_idx {
                    data.regret_sum[ci] += cf_action_value - cf_value;
                } else {
                    // Unsampled actions have cf_action_value = 0
                    data.regret_sum[ci] -= cf_value;
                }
            }

            // Average strategy: reach-weighted
            let strat_weight = pi_i / pi_sample;
            for i in 0..num_actions {
                let ci = if is_canonical { i } else { num_actions - 1 - i };
                data.strategy_sum[ci] += strat_weight * sigma[i];
            }
        }

        value_estimate
    }
}

#[cfg(test)]
#[path = "mccfr_tests.rs"]
mod tests;
