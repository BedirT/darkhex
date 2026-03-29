use pyo3::prelude::*;

/// A player in the game. Black moves first and connects North-South.
/// White connects West-East.
#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Player {
    Black = 0,
    White = 1,
}

#[pymethods]
impl Player {
    /// Return the opposing player.
    #[pyo3(name = "opponent")]
    fn py_opponent(&self) -> Player {
        self.opponent()
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

impl Player {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Player::Black,
            1 => Player::White,
            _ => panic!("invalid player index: {i}"),
        }
    }

    pub fn opponent(self) -> Player {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
}

/// Dark Hex collision rule variant.
///
/// - **Classic (CDH)**: Player retries after collision until successful. Default.
/// - **Abrupt (ADH)**: Collision wastes the turn; play passes to opponent.
#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum CollisionRule {
    /// Player retries until a stone is placed (thesis default).
    Classic = 0,
    /// Collision wastes the turn.
    Abrupt = 1,
}

/// Whether the opponent is informed about collisions.
///
/// - **Silent**: Opponent learns nothing (CDH/ADH default).
/// - **Noisy (NDH)**: Opponent is told a collision occurred (but not where).
/// - **Flash (FDH)**: Opponent is told where the collision occurred.
#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum CollisionInfo {
    /// Opponent learns nothing.
    Silent = 0,
    /// Opponent learns a collision happened (not where).
    Noisy = 1,
    /// Opponent learns where the collision happened.
    Flash = 2,
}

/// Internal cell representation (not exposed to Python).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell {
    Empty,
    Black,
    White,
}

impl Cell {
    pub fn to_char(self) -> char {
        match self {
            Cell::Empty => '.',
            Cell::Black => 'x',
            Cell::White => 'o',
        }
    }

    pub fn from_player(player: Player) -> Self {
        match player {
            Player::Black => Cell::Black,
            Player::White => Cell::White,
        }
    }
}
