mod game;
mod solver;

use pyo3::prelude::*;

use game::enumerate::{enumerate_game_tree, GameTreeStats};
use game::state::DarkHexState;
use game::types::{CollisionInfo, CollisionRule, Player};
use solver::exploitability::{best_response_values, exploitability};
use solver::mccfr::{MCCFRSolver, Sampling};
use solver::pone::PoneDb;

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
    Ok(())
}
