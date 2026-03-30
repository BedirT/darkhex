use std::collections::HashMap;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use darkhex_core::game::enumerate as core_enumerate;
use darkhex_core::game::state as core_state;
use darkhex_core::game::types as core_types;
use darkhex_core::solver::exploitability as core_exploit;
use darkhex_core::solver::mccfr as core_mccfr;
use darkhex_core::solver::pone as core_pone;

// ── Player ────────────────────────────────────────────────────────────

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Player {
    Black = 0,
    White = 1,
}

#[pymethods]
impl Player {
    fn opponent(&self) -> Player {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }

    fn __repr__(&self) -> &'static str {
        match self {
            Player::Black => "Player.Black",
            Player::White => "Player.White",
        }
    }

    fn __int__(&self) -> i32 {
        *self as i32
    }
}

impl From<core_types::Player> for Player {
    fn from(p: core_types::Player) -> Self {
        match p {
            core_types::Player::Black => Player::Black,
            core_types::Player::White => Player::White,
        }
    }
}

impl From<Player> for core_types::Player {
    fn from(p: Player) -> Self {
        match p {
            Player::Black => core_types::Player::Black,
            Player::White => core_types::Player::White,
        }
    }
}

// ── CollisionRule ─────────────────────────────────────────────────────

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum CollisionRule {
    Classic = 0,
    Abrupt = 1,
}

impl From<CollisionRule> for core_types::CollisionRule {
    fn from(r: CollisionRule) -> Self {
        match r {
            CollisionRule::Classic => core_types::CollisionRule::Classic,
            CollisionRule::Abrupt => core_types::CollisionRule::Abrupt,
        }
    }
}

// ── CollisionInfo ─────────────────────────────────────────────────────

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum CollisionInfo {
    Silent = 0,
    Noisy = 1,
    Flash = 2,
}

impl From<CollisionInfo> for core_types::CollisionInfo {
    fn from(i: CollisionInfo) -> Self {
        match i {
            CollisionInfo::Silent => core_types::CollisionInfo::Silent,
            CollisionInfo::Noisy => core_types::CollisionInfo::Noisy,
            CollisionInfo::Flash => core_types::CollisionInfo::Flash,
        }
    }
}

// ── Sampling ──────────────────────────────────────────────────────────

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sampling {
    External = 0,
    Outcome = 1,
}

impl From<Sampling> for core_mccfr::Sampling {
    fn from(s: Sampling) -> Self {
        match s {
            Sampling::External => core_mccfr::Sampling::External,
            Sampling::Outcome => core_mccfr::Sampling::Outcome,
        }
    }
}

// ── DarkHexState ──────────────────────────────────────────────────────

#[pyclass]
#[derive(Clone)]
pub struct DarkHexState(core_state::DarkHexState);

#[pymethods]
impl DarkHexState {
    #[new]
    #[pyo3(signature = (rows, cols, collision_rule=None, collision_info=None))]
    fn new(
        rows: usize,
        cols: usize,
        collision_rule: Option<CollisionRule>,
        collision_info: Option<CollisionInfo>,
    ) -> PyResult<Self> {
        core_state::DarkHexState::new(
            rows,
            cols,
            collision_rule.map(Into::into),
            collision_info.map(Into::into),
        )
        .map(DarkHexState)
        .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[getter]
    fn rows(&self) -> usize {
        self.0.rows()
    }

    #[getter]
    fn cols(&self) -> usize {
        self.0.cols()
    }

    fn num_players(&self) -> usize {
        self.0.num_players()
    }

    fn current_player(&self) -> Player {
        self.0.current_player().into()
    }

    fn is_terminal(&self) -> bool {
        self.0.is_terminal()
    }

    fn winner(&self) -> Option<Player> {
        self.0.winner().map(Into::into)
    }

    fn returns(&self) -> [f64; 2] {
        self.0.returns()
    }

    fn player_return(&self, player: Player) -> f64 {
        self.0.player_return(player.into())
    }

    fn legal_actions(&self) -> Vec<usize> {
        self.0.legal_actions()
    }

    fn num_legal_actions(&self) -> usize {
        self.0.num_legal_actions()
    }

    fn apply_action(&mut self, action: usize) -> PyResult<bool> {
        self.0
            .apply_action(action)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn info_state_string(&self, player: Player) -> String {
        self.0.info_state_string(player.into())
    }

    fn info_state_string_perfect_recall(&self, player: Player) -> String {
        self.0.info_state_string_perfect_recall(player.into())
    }

    fn copy(&self) -> Self {
        DarkHexState(self.0.copy())
    }

    fn stones_placed(&self) -> usize {
        self.0.stones_placed()
    }

    fn num_stones(&self) -> [usize; 2] {
        self.0.num_stones()
    }

    fn collision_rule(&self) -> CollisionRule {
        match self.0.collision_rule() {
            core_types::CollisionRule::Classic => CollisionRule::Classic,
            core_types::CollisionRule::Abrupt => CollisionRule::Abrupt,
        }
    }

    fn collision_info(&self) -> CollisionInfo {
        match self.0.collision_info() {
            core_types::CollisionInfo::Silent => CollisionInfo::Silent,
            core_types::CollisionInfo::Noisy => CollisionInfo::Noisy,
            core_types::CollisionInfo::Flash => CollisionInfo::Flash,
        }
    }

    fn true_board_string(&self) -> String {
        self.0.true_board_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "DarkHexState({}x{}, player={:?}, stones={}, terminal={})",
            self.0.rows(),
            self.0.cols(),
            self.current_player(),
            self.0.stones_placed(),
            self.0.is_terminal()
        )
    }
}

// ── GameTreeStats ─────────────────────────────────────────────────────

#[pyclass]
pub struct GameTreeStats(core_enumerate::GameTreeStats);

#[pymethods]
impl GameTreeStats {
    #[getter]
    fn total_info_states(&self) -> usize {
        self.0.total_info_states
    }

    #[getter]
    fn info_states_by_player(&self) -> [usize; 2] {
        self.0.info_states_by_player
    }

    #[getter]
    fn canonical_info_states(&self) -> usize {
        self.0.canonical_info_states
    }

    #[getter]
    fn canonical_by_player(&self) -> [usize; 2] {
        self.0.canonical_by_player
    }

    #[getter]
    fn game_states_visited(&self) -> usize {
        self.0.game_states_visited
    }

    #[getter]
    fn max_depth(&self) -> usize {
        self.0.max_depth
    }

    fn __repr__(&self) -> String {
        format!(
            "GameTreeStats(info_states={}, canonical={}, P0={}, P1={}, game_states={}, max_depth={})",
            self.0.total_info_states,
            self.0.canonical_info_states,
            self.0.info_states_by_player[0],
            self.0.info_states_by_player[1],
            self.0.game_states_visited,
            self.0.max_depth,
        )
    }
}

// ── MCCFRSolver ───────────────────────────────────────────────────────

#[pyclass]
pub struct MCCFRSolver(core_mccfr::MCCFRSolver);

#[pymethods]
impl MCCFRSolver {
    #[new]
    #[pyo3(signature = (rows, cols, sampling=None, epsilon=None, seed=None))]
    fn new(
        rows: usize,
        cols: usize,
        sampling: Option<Sampling>,
        epsilon: Option<f32>,
        seed: Option<u64>,
    ) -> PyResult<Self> {
        core_mccfr::MCCFRSolver::new(
            rows,
            cols,
            sampling.map(Into::into),
            epsilon,
            seed,
        )
        .map(MCCFRSolver)
        .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn set_pone_db(&mut self, db: &PoneDb) {
        self.0.set_pone_db(db.0.clone());
    }

    fn solve(&mut self, n: usize) {
        self.0.solve(n);
    }

    fn iterations(&self) -> usize {
        self.0.iterations()
    }

    fn num_info_states(&self) -> usize {
        self.0.num_info_states()
    }

    fn sampling(&self) -> Sampling {
        match self.0.sampling() {
            core_mccfr::Sampling::External => Sampling::External,
            core_mccfr::Sampling::Outcome => Sampling::Outcome,
        }
    }

    fn epsilon(&self) -> f32 {
        self.0.epsilon()
    }

    fn get_average_strategy(&self) -> HashMap<String, Vec<(usize, f32)>> {
        self.0.get_average_strategy()
    }

    fn get_current_strategy(&self, info_state: &str) -> Option<Vec<f32>> {
        self.0.get_current_strategy(info_state)
    }

    fn __repr__(&self) -> String {
        format!(
            "MCCFRSolver({}x{}, {:?}, eps={}, iters={}, info_states={})",
            self.0.rows(),
            self.0.cols(),
            self.sampling(),
            self.0.epsilon(),
            self.0.iterations(),
            self.0.num_info_states(),
        )
    }
}

// ── PoneDb ────────────────────────────────────────────────────────────

#[pyclass]
#[derive(Clone)]
pub struct PoneDb(core_pone::PoneDb);

#[pymethods]
impl PoneDb {
    #[new]
    fn new(rows: usize, cols: usize) -> Self {
        PoneDb(core_pone::PoneDb::new(rows, cols))
    }

    fn contains(&self, info_state: &str) -> bool {
        self.0.contains(info_state)
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn rows(&self) -> usize {
        self.0.rows()
    }

    fn cols(&self) -> usize {
        self.0.cols()
    }

    fn __repr__(&self) -> String {
        format!(
            "PoneDb({}x{}, {} pONE states)",
            self.0.rows(),
            self.0.cols(),
            self.0.len()
        )
    }
}

// ── Free functions ────────────────────────────────────────────────────

#[pyfunction]
fn enumerate_game_tree(rows: usize, cols: usize) -> GameTreeStats {
    GameTreeStats(core_enumerate::enumerate_game_tree(rows, cols))
}

#[pyfunction]
#[pyo3(signature = (rows, cols, strategy, pone_db=None))]
fn exploitability(
    rows: usize,
    cols: usize,
    strategy: HashMap<String, Vec<(usize, f32)>>,
    pone_db: Option<&PoneDb>,
) -> f64 {
    core_exploit::exploitability(rows, cols, strategy, pone_db.map(|db| db.0.clone()))
}

#[pyfunction]
#[pyo3(signature = (rows, cols, strategy, pone_db=None))]
fn best_response_values(
    rows: usize,
    cols: usize,
    strategy: HashMap<String, Vec<(usize, f32)>>,
    pone_db: Option<&PoneDb>,
) -> (f64, f64, f64) {
    core_exploit::best_response_values(rows, cols, strategy, pone_db.map(|db| db.0.clone()))
}

// ── Module ────────────────────────────────────────────────────────────

#[pymodule]
fn _engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<Player>()?;
    m.add_class::<CollisionRule>()?;
    m.add_class::<CollisionInfo>()?;
    m.add_class::<DarkHexState>()?;
    m.add_class::<Sampling>()?;
    m.add_class::<MCCFRSolver>()?;
    m.add_class::<GameTreeStats>()?;
    m.add_class::<PoneDb>()?;
    m.add_function(wrap_pyfunction!(enumerate_game_tree, m)?)?;
    m.add_function(wrap_pyfunction!(exploitability, m)?)?;
    m.add_function(wrap_pyfunction!(best_response_values, m)?)?;
    Ok(())
}
