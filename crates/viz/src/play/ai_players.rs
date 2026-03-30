use std::collections::HashMap;

use darkhex_core::game::types::Player;
use rand::Rng;

/// Strategy type: info_state_string → [(action, probability), ...]
pub type Strategy = HashMap<String, Vec<(usize, f32)>>;

/// AI player interface.
pub trait AIPlayer: Send + Sync {
    fn select_action(&self, info_state: &str, legal_actions: &[usize]) -> usize;
    fn display_name(&self) -> &str;
}

/// Plays uniformly at random.
pub struct RandomPlayer;

impl AIPlayer for RandomPlayer {
    fn select_action(&self, _info_state: &str, legal_actions: &[usize]) -> usize {
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..legal_actions.len());
        legal_actions[idx]
    }

    fn display_name(&self) -> &str {
        "Random"
    }
}

/// Plays according to a pre-computed strategy (from MCCFR or loaded file).
pub struct StrategyPlayer {
    strategy: Strategy,
    name: String,
}

impl StrategyPlayer {
    pub fn new(strategy: Strategy, name: impl Into<String>) -> Self {
        Self {
            strategy,
            name: name.into(),
        }
    }

    /// Build a StrategyPlayer by running MCCFR for `iterations` on the given board.
    pub fn from_mccfr(rows: usize, cols: usize, iterations: usize) -> Self {
        use darkhex_core::solver::mccfr::{MCCFRSolver, Sampling};
        let mut solver = MCCFRSolver::new(rows, cols, Some(Sampling::Outcome), Some(0.6), Some(42))
            .expect("valid solver params");
        solver.solve(iterations);
        let strategy = solver.get_average_strategy();
        Self::new(strategy, format!("MCCFR-{iterations}"))
    }
}

impl AIPlayer for StrategyPlayer {
    fn select_action(&self, info_state: &str, legal_actions: &[usize]) -> usize {
        if let Some(entries) = self.strategy.get(info_state) {
            // Build probability distribution over legal actions
            let mut probs = vec![0.0f32; legal_actions.len()];
            for &(action, prob) in entries {
                if let Some(idx) = legal_actions.iter().position(|&a| a == action) {
                    probs[idx] = prob;
                }
            }
            let total: f32 = probs.iter().sum();
            if total > 0.0 {
                // Normalize and sample
                let mut rng = rand::thread_rng();
                let r: f32 = rng.gen::<f32>() * total;
                let mut cum = 0.0;
                for (i, &p) in probs.iter().enumerate() {
                    cum += p;
                    if r < cum {
                        return legal_actions[i];
                    }
                }
                return *legal_actions.last().unwrap();
            }
        }
        // Fallback: uniform random
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..legal_actions.len());
        legal_actions[idx]
    }

    fn display_name(&self) -> &str {
        &self.name
    }
}

/// Solve on first use, then play from the strategy.
pub struct LiveSolverPlayer {
    rows: usize,
    cols: usize,
    iterations: usize,
    inner: Option<StrategyPlayer>,
}

impl LiveSolverPlayer {
    pub fn new(rows: usize, cols: usize, iterations: usize) -> Self {
        Self {
            rows,
            cols,
            iterations,
            inner: None,
        }
    }

    fn ensure_solved(&mut self) {
        if self.inner.is_none() {
            self.inner = Some(StrategyPlayer::from_mccfr(
                self.rows,
                self.cols,
                self.iterations,
            ));
        }
    }
}

impl AIPlayer for LiveSolverPlayer {
    fn select_action(&self, info_state: &str, legal_actions: &[usize]) -> usize {
        // Safety: ensure_solved must be called before this.
        // In practice we call it from the game controller.
        if let Some(ref inner) = self.inner {
            inner.select_action(info_state, legal_actions)
        } else {
            // Fallback: random
            let mut rng = rand::thread_rng();
            let idx = rng.gen_range(0..legal_actions.len());
            legal_actions[idx]
        }
    }

    fn display_name(&self) -> &str {
        "Live MCCFR"
    }
}

/// Which AI type to use.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AIType {
    Random,
    MCCFR,
    LiveSolver,
}

impl AIType {
    pub const ALL: &[AIType] = &[AIType::Random, AIType::MCCFR, AIType::LiveSolver];

    pub fn label(&self) -> &str {
        match self {
            AIType::Random => "Random",
            AIType::MCCFR => "MCCFR (pre-solved)",
            AIType::LiveSolver => "Live MCCFR",
        }
    }
}

/// Create an AI player from the type and board configuration.
pub fn create_ai(
    ai_type: AIType,
    rows: usize,
    cols: usize,
    _player: Player,
) -> Box<dyn AIPlayer> {
    match ai_type {
        AIType::Random => Box::new(RandomPlayer),
        AIType::MCCFR => {
            // Pre-solved: use high iteration count for small boards
            let iters = match (rows, cols) {
                (2, 2) => 100_000,
                (3, 2) | (2, 3) => 50_000,
                _ => 10_000,
            };
            Box::new(StrategyPlayer::from_mccfr(rows, cols, iters))
        }
        AIType::LiveSolver => {
            let iters = match (rows, cols) {
                (2, 2) => 50_000,
                (3, 2) | (2, 3) => 20_000,
                _ => 5_000,
            };
            let mut player = LiveSolverPlayer::new(rows, cols, iters);
            player.ensure_solved();
            Box::new(player)
        }
    }
}
