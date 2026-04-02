use std::collections::HashMap;

use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;

use darkhex_core::error::CoreError;
use darkhex_core::game::enumerate as core_enumerate;
use darkhex_core::game::state::DarkHexState as CoreState;
use darkhex_core::game::types::{
    CollisionInfo as CoreCollisionInfo, CollisionRule as CoreCollisionRule, Player as CorePlayer,
};
use darkhex_core::solver::exploitability as core_exploitability;
use darkhex_core::solver::mccfr::{MCCFRSolver as CoreMCCFR, Sampling as CoreSampling};
use darkhex_core::solver::pone::PoneDb as CorePoneDb;
use darkhex_core::solver::sip as core_sip;

/// Convert CoreError into a Python exception.
fn core_err_to_py(e: CoreError) -> PyErr {
    match e {
        CoreError::InvalidArgument(msg) => PyValueError::new_err(msg),
        CoreError::Io(msg) => PyIOError::new_err(msg),
        CoreError::Serialization(msg) => PyIOError::new_err(msg),
    }
}

// ---------- Player ----------

#[pyclass(name = "Player", eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Player {
    Black = 0,
    White = 1,
}

impl Player {
    fn to_core(self) -> CorePlayer {
        match self {
            Player::Black => CorePlayer::Black,
            Player::White => CorePlayer::White,
        }
    }
    fn from_core(p: CorePlayer) -> Self {
        match p {
            CorePlayer::Black => Player::Black,
            CorePlayer::White => Player::White,
        }
    }
}

#[pymethods]
impl Player {
    fn opponent(&self) -> Player {
        Player::from_core(self.to_core().opponent())
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

// ---------- CollisionRule ----------

#[pyclass(name = "CollisionRule", eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum CollisionRule {
    Classic = 0,
    Abrupt = 1,
}

impl CollisionRule {
    fn to_core(self) -> CoreCollisionRule {
        match self {
            CollisionRule::Classic => CoreCollisionRule::Classic,
            CollisionRule::Abrupt => CoreCollisionRule::Abrupt,
        }
    }
}

// ---------- CollisionInfo ----------

#[pyclass(name = "CollisionInfo", eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum CollisionInfo {
    Silent = 0,
    Noisy = 1,
    Flash = 2,
}

impl CollisionInfo {
    fn to_core(self) -> CoreCollisionInfo {
        match self {
            CollisionInfo::Silent => CoreCollisionInfo::Silent,
            CollisionInfo::Noisy => CoreCollisionInfo::Noisy,
            CollisionInfo::Flash => CoreCollisionInfo::Flash,
        }
    }
}

// ---------- DarkHexState ----------

#[pyclass(name = "DarkHexState")]
pub struct DarkHexState {
    inner: CoreState,
}

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
        let cr = collision_rule.map(|r| r.to_core());
        let ci = collision_info.map(|i| i.to_core());
        let inner = CoreState::new(rows, cols, cr, ci).map_err(core_err_to_py)?;
        Ok(Self { inner })
    }

    #[getter]
    fn rows(&self) -> usize {
        self.inner.rows()
    }

    #[getter]
    fn cols(&self) -> usize {
        self.inner.cols()
    }

    fn num_players(&self) -> usize {
        2
    }

    fn current_player(&self) -> Player {
        Player::from_core(self.inner.current_player())
    }

    fn is_terminal(&self) -> bool {
        self.inner.is_terminal()
    }

    fn winner(&self) -> Option<Player> {
        self.inner.winner().map(Player::from_core)
    }

    fn returns(&self) -> [f64; 2] {
        self.inner.returns()
    }

    fn player_return(&self, player: Player) -> f64 {
        self.inner.player_return(player.to_core())
    }

    fn legal_actions(&self) -> Vec<usize> {
        self.inner.legal_actions()
    }

    fn num_legal_actions(&self) -> usize {
        self.inner.num_legal_actions()
    }

    fn apply_action(&mut self, action: usize) -> PyResult<bool> {
        self.inner.apply_action(action).map_err(core_err_to_py)
    }

    fn info_state_string(&self, player: Player) -> String {
        self.inner.info_state_string(player.to_core())
    }

    fn info_state_string_perfect_recall(&self, player: Player) -> String {
        self.inner
            .info_state_string_perfect_recall(player.to_core())
    }

    fn canonical_info_state(&self, player: Player) -> (String, bool) {
        self.inner.canonical_info_state(player.to_core())
    }

    fn copy(&self) -> Self {
        Self {
            inner: self.inner.copy(),
        }
    }

    fn stones_placed(&self) -> usize {
        self.inner.stones_placed()
    }

    fn num_stones(&self) -> [usize; 2] {
        self.inner.num_stones()
    }

    fn collision_rule(&self) -> CollisionRule {
        match self.inner.collision_rule() {
            CoreCollisionRule::Classic => CollisionRule::Classic,
            CoreCollisionRule::Abrupt => CollisionRule::Abrupt,
        }
    }

    fn collision_info(&self) -> CollisionInfo {
        match self.inner.collision_info() {
            CoreCollisionInfo::Silent => CollisionInfo::Silent,
            CoreCollisionInfo::Noisy => CollisionInfo::Noisy,
            CoreCollisionInfo::Flash => CollisionInfo::Flash,
        }
    }

    fn true_board_string(&self) -> String {
        self.inner.true_board_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "DarkHexState({}x{}, player={:?}, stones={}, terminal={})",
            self.inner.rows(),
            self.inner.cols(),
            self.inner.current_player(),
            self.inner.stones_placed(),
            self.inner.is_terminal()
        )
    }
}

// ---------- Sampling ----------

#[pyclass(name = "Sampling", eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sampling {
    External = 0,
    Outcome = 1,
}

impl Sampling {
    fn to_core(self) -> CoreSampling {
        match self {
            Sampling::External => CoreSampling::External,
            Sampling::Outcome => CoreSampling::Outcome,
        }
    }
    fn from_core(s: CoreSampling) -> Self {
        match s {
            CoreSampling::External => Sampling::External,
            CoreSampling::Outcome => Sampling::Outcome,
        }
    }
}

// ---------- MCCFRSolver ----------

#[pyclass(name = "MCCFRSolver")]
pub struct MCCFRSolver {
    inner: CoreMCCFR,
}

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
        let s = sampling.map(|s| s.to_core());
        let inner = CoreMCCFR::new(rows, cols, s, epsilon, seed).map_err(core_err_to_py)?;
        Ok(Self { inner })
    }

    fn set_pone_db(&mut self, db: &PoneDb) {
        self.inner.set_pone_db(db.inner.clone());
    }

    fn solve(&mut self, n: usize) {
        self.inner.solve(n);
    }

    fn iterations(&self) -> usize {
        self.inner.iterations()
    }

    fn num_info_states(&self) -> usize {
        self.inner.num_info_states()
    }

    fn sampling(&self) -> Sampling {
        Sampling::from_core(self.inner.sampling())
    }

    fn epsilon(&self) -> f32 {
        self.inner.epsilon()
    }

    fn get_average_strategy(&self) -> HashMap<String, Vec<(usize, f32)>> {
        self.inner.get_average_strategy()
    }

    fn get_current_strategy(&self, info_state: &str) -> Option<Vec<f32>> {
        self.inner.get_current_strategy(info_state)
    }

    fn save(&self, path: &str) -> PyResult<()> {
        self.inner.save(path).map_err(core_err_to_py)
    }

    #[staticmethod]
    fn load(path: &str) -> PyResult<Self> {
        let inner = CoreMCCFR::load(path).map_err(core_err_to_py)?;
        Ok(Self { inner })
    }

    fn __repr__(&self) -> String {
        format!(
            "MCCFRSolver({:?}, eps={}, iters={}, info_states={})",
            self.inner.sampling(),
            self.inner.epsilon(),
            self.inner.iterations(),
            self.inner.num_info_states()
        )
    }
}

// ---------- GameTreeStats ----------

#[pyclass(name = "GameTreeStats")]
pub struct GameTreeStats {
    inner: core_enumerate::GameTreeStats,
}

#[pymethods]
impl GameTreeStats {
    #[getter]
    fn total_info_states(&self) -> usize {
        self.inner.total_info_states
    }

    #[getter]
    fn info_states_by_player(&self) -> [usize; 2] {
        self.inner.info_states_by_player
    }

    #[getter]
    fn canonical_info_states(&self) -> usize {
        self.inner.canonical_info_states
    }

    #[getter]
    fn canonical_by_player(&self) -> [usize; 2] {
        self.inner.canonical_by_player
    }

    #[getter]
    fn game_states_visited(&self) -> usize {
        self.inner.game_states_visited
    }

    #[getter]
    fn max_depth(&self) -> usize {
        self.inner.max_depth
    }

    fn __repr__(&self) -> String {
        format!(
            "GameTreeStats(info_states={}, canonical={}, P0={}, P1={}, game_states={}, max_depth={})",
            self.inner.total_info_states,
            self.inner.canonical_info_states,
            self.inner.info_states_by_player[0],
            self.inner.info_states_by_player[1],
            self.inner.game_states_visited,
            self.inner.max_depth,
        )
    }
}

// ---------- PoneDb ----------

#[pyclass(name = "PoneDb")]
pub struct PoneDb {
    inner: CorePoneDb,
}

#[pymethods]
impl PoneDb {
    #[new]
    fn new(rows: usize, cols: usize) -> Self {
        Self {
            inner: CorePoneDb::new(rows, cols),
        }
    }

    fn contains(&self, info_state: &str) -> bool {
        self.inner.contains(info_state)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn rows(&self) -> usize {
        self.inner.rows()
    }

    fn cols(&self) -> usize {
        self.inner.cols()
    }

    fn __repr__(&self) -> String {
        format!(
            "PoneDb({}x{}, {} pONE states)",
            self.inner.rows(),
            self.inner.cols(),
            self.inner.len()
        )
    }
}

// ---------- Free functions ----------

#[pyfunction]
fn enumerate_game_tree(rows: usize, cols: usize) -> GameTreeStats {
    GameTreeStats {
        inner: core_enumerate::enumerate_game_tree(rows, cols),
    }
}

#[pyfunction]
#[pyo3(signature = (rows, cols, strategy, pone_db=None))]
fn exploitability(
    rows: usize,
    cols: usize,
    strategy: HashMap<String, Vec<(usize, f32)>>,
    pone_db: Option<&PoneDb>,
) -> f64 {
    core_exploitability::exploitability(rows, cols, strategy, pone_db.map(|db| db.inner.clone()))
}

#[pyfunction]
#[pyo3(signature = (rows, cols, strategy, pone_db=None))]
fn best_response_values(
    rows: usize,
    cols: usize,
    strategy: HashMap<String, Vec<(usize, f32)>>,
    pone_db: Option<&PoneDb>,
) -> (f64, f64, f64) {
    core_exploitability::best_response_values(
        rows,
        cols,
        strategy,
        pone_db.map(|db| db.inner.clone()),
    )
}

#[pyfunction]
#[pyo3(signature = (strategy, epsilon, action_cap))]
fn simplify_policy(
    strategy: HashMap<String, Vec<(usize, f32)>>,
    epsilon: f32,
    action_cap: usize,
) -> PyResult<HashMap<String, Vec<(usize, f32)>>> {
    core_sip::simplify_policy(strategy, epsilon, action_cap).map_err(core_err_to_py)
}

#[pyfunction]
#[pyo3(signature = (strategy, epsilon, action_cap, frac_limit, eta))]
fn simplify_policy_plus(
    strategy: HashMap<String, Vec<(usize, f32)>>,
    epsilon: f32,
    action_cap: usize,
    frac_limit: usize,
    eta: f32,
) -> PyResult<HashMap<String, Vec<(usize, f32)>>> {
    core_sip::simplify_policy_plus(strategy, epsilon, action_cap, frac_limit, eta)
        .map_err(core_err_to_py)
}

// ---------- Module ----------

/// DarkHex game engine and solver, implemented in Rust.
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
    m.add_function(wrap_pyfunction!(simplify_policy, m)?)?;
    m.add_function(wrap_pyfunction!(simplify_policy_plus, m)?)?;
    Ok(())
}
