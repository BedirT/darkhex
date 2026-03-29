mod board;
mod state;
mod types;

use pyo3::prelude::*;

use state::DarkHexState;
use types::{CollisionInfo, CollisionRule, Player};

/// DarkHex game engine implemented in Rust, exposed to Python via PyO3.
///
/// Supports all four thesis variants (CDH/ADH/NDH/FDH) via
/// `CollisionRule` × `CollisionInfo` configuration.
#[pymodule]
fn _engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<Player>()?;
    m.add_class::<CollisionRule>()?;
    m.add_class::<CollisionInfo>()?;
    m.add_class::<DarkHexState>()?;
    Ok(())
}
