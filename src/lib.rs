mod board;
mod mccfr;
mod state;
mod types;

use pyo3::prelude::*;

use mccfr::MCCFRSolver;
use state::DarkHexState;
use types::{CollisionInfo, CollisionRule, Player};

/// DarkHex game engine and solver, implemented in Rust.
#[pymodule]
fn _engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<Player>()?;
    m.add_class::<CollisionRule>()?;
    m.add_class::<CollisionInfo>()?;
    m.add_class::<DarkHexState>()?;
    m.add_class::<MCCFRSolver>()?;
    Ok(())
}
